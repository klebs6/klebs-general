use rck::*;
use std::io::Write;
use tempfile::NamedTempFile;
use std::path::PathBuf;

#[test]
fn test_load_manifest() {
    // Create a temporary file to hold a mock manifest
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
        {{
            "repos": [
                {{
                    "name": "test-repo",
                    "url": "https://example.com/repo.git",
                    "path": "repo/test-repo",
                    "branch": "main",
                    "remote": "origin"
                }}
            ]
        }}
        "#
    )
    .unwrap();

    let path = file.path().to_str().unwrap();

    // Load the manifest and check the result
    let manifest = Manifest::load(path).unwrap();

    assert_eq!(manifest.n_repos(), 1);

    let repo = manifest.repo(0).unwrap();
    assert_eq!(repo.name(), "test-repo");
    assert_eq!(repo.url(), "https://example.com/repo.git");
    assert_eq!(repo.path(), PathBuf::from("repo/test-repo"));
    assert_eq!(repo.branch(), "main");
    assert_eq!(repo.remote(), "origin");
}

