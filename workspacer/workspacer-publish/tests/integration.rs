// File: workspacer-publish/tests/integration_mock_local.rs

use workspacer_3p::*;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use portpicker::pick_unused_port;
use rocket::figment::Figment;
use rocket::figment::providers::Format;
use rocket::{catchers, routes, Config as RocketConfig};
use rocket::tokio;
use tempfile::tempdir;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info, trace, warn};

use workspacer_cratesio_mock::{AppStateBuilder, MockCratesDb, not_found, publish_new};
use workspacer_errors::WorkspaceError;
use workspacer_publish::*; // pulls in our publish_topo and try_publish_crate modules

#[tokio::test]
#[disable]
async fn test_publish_against_local_git_index_with_api() -> Result<(), WorkspaceError> {
    trace!("Starting test_publish_against_local_git_index_with_api");

    // STEP 1. Create a local bare git repository to serve as the index.
    let bare_repo_dir = tempdir().expect("Failed to create temp directory for bare repo");
    let bare_repo_path = bare_repo_dir.path().join("mock-index.git");

    let output = Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(&bare_repo_path)
        .output()
        .expect("Failed to initialize bare repository");
    if !output.status.success() {
        panic!(
            "git init --bare failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // STEP 2. Clone that bare repo so we can commit a config.json.
    let clone_dir = tempdir().expect("Failed to create temp directory for clone");
    let clone_path = clone_dir.path().join("mock-index-clone");
    let output = Command::new("git")
        .arg("clone")
        .arg(&bare_repo_path)
        .arg(&clone_path)
        .output()
        .expect("Failed to clone bare repository");
    if !output.status.success() {
        panic!(
            "git clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Write a minimal config.json so Cargo recognizes the index.
    fs::write(clone_path.join("config.json"), r#"{"dl": "http://127.0.0.1/download"}"#)
        .expect("Failed to write config.json to index clone");

    // Commit & push the new file.
    {
        let output = Command::new("git")
            .current_dir(&clone_path)
            .arg("add")
            .arg("config.json")
            .output()
            .expect("Failed to run git add");
        if !output.status.success() {
            panic!(
                "git add config.json failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let output = Command::new("git")
            .current_dir(&clone_path)
            .arg("commit")
            .arg("-m")
            .arg("Add config.json for cargo registry index")
            .output()
            .expect("Failed to commit config.json");
        if !output.status.success() {
            panic!(
                "git commit failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let output = Command::new("git")
            .current_dir(&clone_path)
            .arg("push")
            .arg("origin")
            .arg("master")
            .output()
            .expect("Failed to push to bare repository");
        if !output.status.success() {
            panic!(
                "git push failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // STEP 3. Launch a Rocket instance for the mock crates.io server.
    let port = pick_unused_port().expect("No free port available");
    info!("Mock crates.io server will use port: {}", port);

    let rocket_config = RocketConfig {
        port,
        address: Ipv4Addr::LOCALHOST.into(),
        ..RocketConfig::debug_default()
    };

    let state = AppStateBuilder::default()
        .db(Arc::new(AsyncMutex::new(MockCratesDb::default())))
        .build()
        .expect("Failed to build AppState");

    let rocket_instance = rocket::custom(Figment::from(rocket_config))
        .manage(state)
        .mount("/", routes![publish_new])
        .register("/", catchers![not_found]);

    let rocket_handle = tokio::spawn(async move {
        if let Err(e) = rocket_instance.launch().await {
            error!("Rocket failed to launch: {:?}", e);
        }
    });

    // Wait briefly for Rocket to launch.
    tokio::time::sleep(Duration::from_millis(250)).await;

    // STEP 4. Prepare a minimal test crate.
    let crate_tmp = tempdir().expect("Failed to create test crate directory");
    let crate_path = crate_tmp.path();
    fs::write(
        crate_path.join("Cargo.toml"),
        r#"[package]
name = "test_integration_against_mock"
version = "0.1.0"
edition = "2021"
authors = ["IntegrationTest"]
license = "MIT"
"#,
    )
    .expect("Failed to write Cargo.toml for test crate");

    fs::create_dir_all(crate_path.join("src")).unwrap();
    fs::write(crate_path.join("src").join("lib.rs"), "// test lib")
        .expect("Failed to write src/lib.rs for test crate");

    // STEP 5. Create a .cargo/config.toml inside the test crate directory.
    // This file tells Cargo that for registry "mock", the index is at our bare repo (file://) and the API is at our Rocket server.
    let cargo_config_dir = crate_path.join(".cargo");
    fs::create_dir_all(&cargo_config_dir)
        .expect("Failed to create .cargo directory in test crate");
    let config_toml = format!(
r#"[registries.mock]
index = "file://{index_path}"
api = "http://127.0.0.1:{port}/api/v1/crates"
"#,
        index_path = bare_repo_path.display(),
        port = port
    );
    fs::write(cargo_config_dir.join("config.toml"), &config_toml)
        .expect("Failed to write .cargo/config.toml");

    // STEP 6. Set environment variables to override any defaults.
    let index_url = format!("file://{}", bare_repo_path.display());
    unsafe {
        env::set_var("CARGO_REGISTRIES_MOCK_INDEX", &index_url);
        env::set_var("CARGO_REGISTRIES_MOCK_API", format!("http://127.0.0.1:{}/api/v1/crates", port));
        env::set_var("CARGO_REGISTRIES_MOCK_TOKEN", "myTestToken123");
        env::set_var("USE_MOCK_REGISTRY", "1");
    }

    // STEP 7. Run "cargo publish" in the test crate directory.
    let output = Command::new("cargo")
        .current_dir(&crate_path)
        .arg("publish")
        .arg("--registry=mock")
        .arg("--allow-dirty")
        .env_remove("CARGO_HOME")
        .output()
        .expect("Failed to spawn cargo publish");
    if !output.status.success() {
        eprintln!("cargo publish stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("cargo publish stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("First cargo publish failed unexpectedly!");
    } else {
        info!("First cargo publish succeeded!");
    }

    // STEP 8. Attempt a second publish of the same version; it should fail because the crate already exists.
    let output2 = Command::new("cargo")
        .current_dir(&crate_path)
        .arg("publish")
        .arg("--registry=mock")
        .arg("--allow-dirty")
        .env_remove("CARGO_HOME")
        .output()
        .expect("Failed to run cargo publish second time");
    if output2.status.success() {
        panic!("Second publish unexpectedly succeeded, but it should have failed (already published)!");
    } else {
        info!("Second publish failed as expected (crate version already published).");
    }

    // STEP 9. Teardown: abort Rocket and remove environment variables.
    rocket_handle.abort();
    unsafe {
        env::remove_var("CARGO_REGISTRIES_MOCK_INDEX");
        env::remove_var("CARGO_REGISTRIES_MOCK_API");
        env::remove_var("CARGO_REGISTRIES_MOCK_TOKEN");
        env::remove_var("USE_MOCK_REGISTRY");
    }

    trace!("test_publish_against_local_git_index_with_api passed successfully");
    Ok(())
}
