// workspacer-bump/tests/cascade_bump.rs

use std::{fs, io::Write, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

use workspacer_bump::*;
use workspacer_crate::*;
use workspacer_workspace::*;
use workspacer_3p::*;
use semver::Version;

// ------------------------ Test helpers ------------------------

fn unique_tmp_dir(prefix: &str) -> PathBuf {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let mut p = std::env::temp_dir();
    p.push(format!("{prefix}_{ts}"));
    p
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).unwrap(); }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
}

fn mk_root_workspace(root: &Path, crates: &[&str]) {
    fs::create_dir_all(root).unwrap();
    let mut members = String::new();
    for c in crates {
        members.push_str(&format!(r#"    "{c}",
"#));
    }
    let ws = format!(
        "[workspace]\n\
         members = [\n{members}]\
        \n"
    );
    write_file(&root.join("Cargo.toml"), &ws);
}

#[derive(Clone, Copy)]
enum DepKind { Normal, Dev, Build }

#[derive(Clone)]
struct DepSpec<'a> {
    name: &'a str,
    // if Some -> include a version key with this value; otherwise omit the version key (path-only dep)
    version: Option<&'a str>,
    // if Some -> include this path value; otherwise omit the path key
    path_rel: Option<&'a str>,
    kind: DepKind,
}

fn mk_crate(
    root: &Path,
    name: &str,
    version: &str,
    deps: &[DepSpec<'_>],
) -> PathBuf {
    let base = root.join(name);
    fs::create_dir_all(&base).unwrap();
    write_file(&base.join("README.md"), &format!("# {name}\n"));
    fs::create_dir_all(&base.join("src")).unwrap();
    write_file(&base.join("src/lib.rs"), "pub fn ping() {}\n");

    // Group deps into sections
    let mut dep_lines = String::new();
    let mut dev_dep_lines = String::new();
    let mut build_dep_lines = String::new();

    for d in deps {
        let line = match (d.version, d.path_rel) {
            (Some(v), Some(p)) => format!(r#"{k} = {{ path = "{p}", version = "{v}" }}"#, k = d.name),
            (Some(v), None)    => format!(r#"{k} = "{v}""#,                       k = d.name),
            (None,    Some(p)) => format!(r#"{k} = {{ path = "{p}" }}"#,          k = d.name),
            (None,    None)    => format!(r#"{k} = "*""#,                          k = d.name),
        };
        match d.kind {
            DepKind::Normal => { dep_lines.push_str(&line); dep_lines.push('\n'); }
            DepKind::Dev    => { dev_dep_lines.push_str(&line); dev_dep_lines.push('\n'); }
            DepKind::Build  => { build_dep_lines.push_str(&line); build_dep_lines.push('\n'); }
        }
    }

    let mut toml = format!(
        r#"[package]
name = "{name}"
version = "{version}"
edition = "2021"
license = "MIT"
authors = ["tester@example.com"]

"#
    );
    if !dep_lines.is_empty() {
        toml.push_str("[dependencies]\n"); toml.push_str(&dep_lines); toml.push('\n');
    }
    if !dev_dep_lines.is_empty() {
        toml.push_str("[dev-dependencies]\n"); toml.push_str(&dev_dep_lines); toml.push('\n');
    }
    if !build_dep_lines.is_empty() {
        toml.push_str("[build-dependencies]\n"); toml.push_str(&build_dep_lines); toml.push('\n');
    }

    write_file(&base.join("Cargo.toml"), &toml);
    base
}

fn read_pkg_version(crate_dir: &Path) -> Version {
    let s = fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    let v: toml::Value = toml::from_str(&s).unwrap();
    let ver = v["package"]["version"].as_str().unwrap();
    Version::parse(ver).unwrap()
}

fn read_dep_version(crate_dir: &Path, section: &str, dep: &str) -> Option<String> {
    let s = fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    let v: toml::Value = toml::from_str(&s).unwrap();
    let sec = v.get(section)?.as_table()?;
    let item = sec.get(dep)?;
    if let Some(s) = item.as_str() {
        return Some(s.to_string());
    }
    if let Some(t) = item.as_table() {
        if let Some(s) = t.get("version").and_then(|x| x.as_str()) {
            return Some(s.to_string());
        } else {
            // path-only dep: no version key
            return None;
        }
    }
    None
}

fn has_dep(crate_dir: &Path, section: &str, dep: &str) -> bool {
    let s = fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    let v: toml::Value = toml::from_str(&s).unwrap();
    v.get(section)
        .and_then(|sec| sec.as_table())
        .map_or(false, |t| t.contains_key(dep))
}

//
// ------------------------ Tests ------------------------
//

#[traced_test]
async fn cascade_patch_updates_basic_chain_and_path_only() {
    // Workspace: A -> B -> C
    // B depends on A (with version+path)
    // C depends on B (path-only => no version key)
    let root = unique_tmp_dir("ws_cascade_patch_chain");
    mk_root_workspace(&root, &["A", "B", "C"]);

    let a = mk_crate(&root, "A", "0.1.0", &[]);
    let b = mk_crate(
        &root, "B", "0.1.5",
        &[DepSpec { name: "A", version: Some("0.1.0"), path_rel: Some("../A"), kind: DepKind::Normal }]
    );
    let c = mk_crate(
        &root, "C", "0.1.0",
        &[DepSpec { name: "B", version: None, path_rel: Some("../B"), kind: DepKind::Normal }] // path-only
    );

    let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
    let mut a_handle = CrateHandle::new_sync(&a).unwrap();
    ws.bump_crate_and_downstreams(&mut a_handle, ReleaseType::Patch).await.unwrap();

    // A bumped: 0.1.0 -> 0.1.1
    assert_eq!(read_pkg_version(&a).to_string(), "0.1.1");

    // B bumped: 0.1.5 -> 0.1.6, and its dep on A updated to 0.1.1
    assert_eq!(read_pkg_version(&b).to_string(), "0.1.6");
    assert_eq!(read_dep_version(&b, "dependencies", "A").as_deref(), Some("0.1.1"));

    // C bumped even though its dependency on B was path-only
    // 0.1.0 -> 0.1.1. Its dep on B remains path-only (no version key), but version bump still occurred.
    assert_eq!(read_pkg_version(&c).to_string(), "0.1.1");
    assert!(has_dep(&c, "dependencies", "B")); // present
    assert!(read_dep_version(&c, "dependencies", "B").is_none()); // still path-only, as desired
}

#[traced_test]
async fn cascade_handles_diamond_and_single_bump_per_node() {
    // Diamond:
    //       A
    //     /   \
    //    B     D
    //     \   /
    //       C
    //
    // B depends on A (with version+path)
    // D depends on A (path-only)
    // C depends on both B (with version+path) and D (with version+path)
    //
    // Ensure that:
    // - B and D each bump once
    // - C bumps once (even though both B and D change)
    // - All versioned edges update to new versions; path-only remain path-only.

    let root = unique_tmp_dir("ws_cascade_diamond");
    mk_root_workspace(&root, &["A", "B", "C", "D"]);

    let a = mk_crate(&root, "A", "0.1.0", &[]);
    let b = mk_crate(&root, "B", "0.1.0", &[
        DepSpec { name: "A", version: Some("0.1.0"), path_rel: Some("../A"), kind: DepKind::Normal },
    ]);
    let d = mk_crate(&root, "D", "0.1.2", &[
        DepSpec { name: "A", version: None, path_rel: Some("../A"), kind: DepKind::Normal }, // path-only
    ]);
    let c = mk_crate(&root, "C", "0.2.3", &[
        DepSpec { name: "B", version: Some("0.1.0"), path_rel: Some("../B"), kind: DepKind::Normal },
        DepSpec { name: "D", version: Some("0.1.2"), path_rel: Some("../D"), kind: DepKind::Normal },
    ]);

    let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
    let mut a_handle = CrateHandle::new_sync(&a).unwrap();
    ws.bump_crate_and_downstreams(&mut a_handle, ReleaseType::Patch).await.unwrap();

    // A: 0.1.1
    assert_eq!(read_pkg_version(&a).to_string(), "0.1.1");

    // B: 0.1.1 and dep A -> 0.1.1
    assert_eq!(read_pkg_version(&b).to_string(), "0.1.1");
    assert_eq!(read_dep_version(&b, "dependencies", "A").as_deref(), Some("0.1.1"));

    // D: 0.1.3 (bumped) and its dep on A remains path-only (no version key)
    assert_eq!(read_pkg_version(&d).to_string(), "0.1.3");
    assert!(read_dep_version(&d, "dependencies", "A").is_none());

    // C: bumped once: 0.2.3 -> 0.2.4
    //    dependencies reflect B=0.1.1 and D=0.1.3
    assert_eq!(read_pkg_version(&c).to_string(), "0.2.4");
    assert_eq!(read_dep_version(&c, "dependencies", "B").as_deref(), Some("0.1.1"));
    assert_eq!(read_dep_version(&c, "dependencies", "D").as_deref(), Some("0.1.3"));
}

#[traced_test]
async fn cascade_updates_dev_and_build_dependencies() {

    // E has:
    //   - [dev-dependencies] on A with version+path
    //   - [build-dependencies] on A path-only
    // After bump A(Patch), E's own version should bump and:
    //   - dev-dependency version updates,
    //   - build-dependency remains path-only.

    let root = unique_tmp_dir("ws_cascade_dev_build");
    mk_root_workspace(&root, &["A", "E"]);

    let a = mk_crate(&root, "A", "1.2.3", &[]);
    let e = mk_crate(&root, "E", "0.9.9", &[
        DepSpec { name: "A", version: Some("1.2.3"), path_rel: Some("../A"), kind: DepKind::Dev   },
        DepSpec { name: "A", version: None,          path_rel: Some("../A"), kind: DepKind::Build },
    ]);

    let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
    let mut a_handle = CrateHandle::new_sync(&a).unwrap();
    ws.bump_crate_and_downstreams(&mut a_handle, ReleaseType::Patch).await.unwrap();

    // A: 1.2.4
    assert_eq!(read_pkg_version(&a).to_string(), "1.2.4");

    // E: 0.9.10
    assert_eq!(read_pkg_version(&e).to_string(), "0.9.10");

    // dev-dep updated to 1.2.4
    assert_eq!(read_dep_version(&e, "dev-dependencies", "A").as_deref(), Some("1.2.4"));

    // build-dep remains path-only (no version key)
    assert!(has_dep(&e, "build-dependencies", "A"));
    assert!(read_dep_version(&e, "build-dependencies", "A").is_none());
}

#[traced_test]
async fn release_type_minor_and_major_propagate() {
    // F -> G (version+path)
    let root = unique_tmp_dir("ws_minor_major");
    mk_root_workspace(&root, &["F", "G"]);

    let f = mk_crate(&root, "F", "0.4.9", &[]);
    let g = mk_crate(&root, "G", "2.3.7", &[
        DepSpec { name: "F", version: Some("0.4.9"), path_rel: Some("../F"), kind: DepKind::Normal },
    ]);

    // Minor
    {
        let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
        let mut f_handle = CrateHandle::new_sync(&f).unwrap();
        ws.bump_crate_and_downstreams(&mut f_handle, ReleaseType::Minor).await.unwrap();
    }
    assert_eq!(read_pkg_version(&f).to_string(), "0.5.0");
    assert_eq!(read_pkg_version(&g).to_string(), "2.4.0");
    assert_eq!(read_dep_version(&g, "dependencies", "F").as_deref(), Some("0.5.0"));

    // Major
    {
        let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
        let mut f_handle = CrateHandle::new_sync(&f).unwrap();
        ws.bump_crate_and_downstreams(&mut f_handle, ReleaseType::Major).await.unwrap();
    }
    assert_eq!(read_pkg_version(&f).to_string(), "1.0.0");
    assert_eq!(read_pkg_version(&g).to_string(), "3.0.0");
    assert_eq!(read_dep_version(&g, "dependencies", "F").as_deref(), Some("1.0.0"));

}

#[traced_test]
async fn release_type_alpha_preserves_build_and_sets_prerelease_everywhere() {
    // H -> I (both have build metadata, I depends on H (version+path))
    let root = unique_tmp_dir("ws_alpha");
    mk_root_workspace(&root, &["H", "I"]);

    // versions with build metadata (+build.7)
    let h = mk_crate(&root, "H", "0.1.5+build.7", &[]);
    let i = mk_crate(&root, "I", "2.0.0+meta", &[
        DepSpec { name: "H", version: Some("0.1.5+build.7"), path_rel: Some("../H"), kind: DepKind::Normal },
    ]);

    let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
    let mut h_handle = CrateHandle::new_sync(&h).unwrap();
    ws.bump_crate_and_downstreams(&mut h_handle, ReleaseType::Alpha(Some(3))).await.unwrap();

    // H: pre-release set, numeric parts unchanged, build metadata preserved
    assert_eq!(read_pkg_version(&h).to_string(), "0.1.5-alpha3+build.7");

    // I: also alpha3 with its own build metadata preserved
    assert_eq!(read_pkg_version(&i).to_string(), "2.0.0-alpha3+meta");

    // dep on H updated to new pre-release (build of dependency is not carried over into the version field; the version field equals H's new string)
    assert_eq!(
        read_dep_version(&i, "dependencies", "H").as_deref(),
        Some("0.1.5-alpha3+build.7")
    );

}

#[traced_test]
async fn crates_unrelated_to_source_remain_unchanged() {
    // A -> B   and   X (no deps)
    let root = unique_tmp_dir("ws_unrelated");
    mk_root_workspace(&root, &["A", "B", "X"]);

    let a = mk_crate(&root, "A", "0.1.0", &[]);
    let b = mk_crate(&root, "B", "0.1.0", &[
        DepSpec { name: "A", version: Some("0.1.0"), path_rel: Some("../A"), kind: DepKind::Normal },
    ]);
    let x = mk_crate(&root, "X", "9.9.9", &[]);

    let orig_x_ver = read_pkg_version(&x);

    let mut ws: Workspace<PathBuf, CrateHandle> = Workspace::new(&root).await.unwrap();
    let mut a_handle = CrateHandle::new_sync(&a).unwrap();
    ws.bump_crate_and_downstreams(&mut a_handle, ReleaseType::Patch).await.unwrap();

    assert_eq!(read_pkg_version(&a).to_string(), "0.1.1");
    assert_eq!(read_pkg_version(&b).to_string(), "0.1.1");
    // X untouched
    assert_eq!(read_pkg_version(&x), orig_x_ver);
}
