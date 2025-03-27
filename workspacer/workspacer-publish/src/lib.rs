// ---------------- [ File: workspacer-publish/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{publish_topo}
x!{try_publish_crate}

#[cfg(test)]
mod integration_with_mock_cratesio {
    use super::*;
    use portpicker::pick_unused_port;
    use std::time::Duration;
    use std::thread;
    use rocket::Config;
    use rocket::figment::Figment;
    use rocket::figment::providers::Format;
    use rocket::tokio::task::JoinHandle;
    use tracing::{trace, debug, info, warn, error};
    use std::process::Command;
    use tempfile::tempdir;

    // We'll import the mock crates.io server from your "workspacer-cratesio-mock" crate:
    #[allow(unused_imports)]
    use workspacer_cratesio_mock::{
        AppStateBuilder, MockCratesDb, not_found, publish_new,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex as AsyncMutex;

    #[test]
    fn test_publish_against_mock_cratesio() {
        // We'll do an actual "cargo publish --registry=mock" but set
        // the env so that mock = "http://127.0.0.1:port/index".

        let rt = rocket::tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            trace!("Starting test_publish_against_mock_cratesio.");

            // 1) pick a random port
            let port = pick_unused_port().expect("No free ports available");
            info!("Mock server using port: {}", port);

            // 2) build rocket config
            let rocket_config = Config {
                port,
                address: std::net::Ipv4Addr::LOCALHOST.into(),
                ..Config::debug_default()
            };

            // 3) create the in-memory DB state
            let state = AppStateBuilder::default()
                .db(Arc::new(AsyncMutex::new(MockCratesDb::default())))
                .build()
                .unwrap();

            // 4) build rocket instance
            let rocket_instance = rocket::custom(Figment::from(rocket_config))
                .manage(state)
                .mount("/", rocket::routes![publish_new])
                .register("/", rocket::catchers![not_found]);

            // 5) launch in background
            let rocket_handle: JoinHandle<_> = tokio::spawn(async move {
                if let Err(e) = rocket_instance.launch().await {
                    error!("Mock crates.io server failed to launch: {:?}", e);
                }
            });

            // give rocket time
            thread::sleep(Duration::from_millis(250));

            // 6) define the environment so `cargo publish --registry=mock`
            //    uses `CARGO_REGISTRIES_MOCK_INDEX=http://127.0.0.1:PORT/index`
            let index_url = format!("http://127.0.0.1:{}/index", port);
            unsafe { std::env::set_var("CARGO_REGISTRIES_MOCK_INDEX", &index_url) };

            // Optionally set "USE_MOCK_REGISTRY=1" if we want to 
            // cause "cargo publish" calls in `try_publish` to do `--registry=mock`
            unsafe { std::env::set_var("USE_MOCK_REGISTRY", "1") };

            // 7) create a local test crate
            let tmp = tempdir().unwrap();
            let crate_dir = tmp.path();
            std::fs::write(
                crate_dir.join("Cargo.toml"),
                r#"[package]
name = "test_integration_against_mock"
version = "0.1.0"
edition = "2021"
authors = ["IntegrationTest"]
license = "MIT"
"#,
            ).unwrap();

            let src_dir = crate_dir.join("src");
            std::fs::create_dir_all(&src_dir).unwrap();
            std::fs::write(src_dir.join("lib.rs"), "// test lib").unwrap();

            // 8) run cargo publish
            let status = Command::new("cargo")
                .current_dir(&crate_dir)
                .arg("publish")
                .arg("--allow-dirty")
                .arg("--registry=mock")
                .env_remove("CARGO_HOME") // optional
                .status()
                .expect("Failed to run cargo publish with mock registry");
            
            if !status.success() {
                panic!("cargo publish failed with code {:?}", status.code());
            } else {
                info!("Successfully published test crate to the mock registry!");
            }

            // 9) optionally do a second publish => should fail as "already exists"
            let status2 = Command::new("cargo")
                .current_dir(&crate_dir)
                .arg("publish")
                .arg("--allow-dirty")
                .arg("--registry=mock")
                .env_remove("CARGO_HOME")
                .status()
                .expect("Failed to run cargo publish (second time)");
            // This second publish should fail with code != 0, 
            // and the rocket logs would show "crate <name> already exists." 
            info!("Second publish returned code={:?}", status2.code());
            assert!(!status2.success());

            // cleanup
            rocket_handle.abort();
            unsafe { std::env::remove_var("CARGO_REGISTRIES_MOCK_INDEX") };
            unsafe { std::env::remove_var("USE_MOCK_REGISTRY") };
        });
    }
}
