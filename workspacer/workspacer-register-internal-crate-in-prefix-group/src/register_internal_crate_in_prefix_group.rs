crate::ix!();

// ---------------------- [ File: workspacer-register-internal-crate-in-prefix-group/src/lib.rs ] ----------------------

/// This trait says: "Given a prefix group’s facade crate, register a new internal crate in it."
/// Typically means:
/// 1) Add a `[dependencies] new_crate = { path = ... }` to the facade crate’s Cargo.toml.
/// 2) Possibly add `pub use new_crate::*;` or a mod statement in facade crate’s code.
///
#[async_trait]
pub trait RegisterInPrefixGroup {
    async fn register_in_prefix_crate(
        &self,
        prefix_crate: &CrateHandle,
        new_crate: &CrateHandle,
    ) -> Result<(), RegisterCrateInPrefixGroupError>;
}

/// Minimal default impl
pub struct PrefixGroupRegistrar;

impl PrefixGroupRegistrar {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RegisterInPrefixGroup for PrefixGroupRegistrar {
    async fn register_in_prefix_crate(
        &self,
        prefix_crate: &CrateHandle,
        new_crate: &CrateHandle,
    ) -> Result<(), RegisterCrateInPrefixGroupError> {
        info!("Registering crate '{}' into prefix crate '{}'",
              new_crate.name(), prefix_crate.name());
        // 1) Open prefix_crate’s Cargo.toml, add a dependency to new_crate
        let facade_path = prefix_crate.as_ref().join("Cargo.toml");
        let content = fs::read_to_string(&facade_path).await.map_err(|io_err| {
            RegisterCrateInPrefixGroupError::IoError {
                context: format!("reading facade Cargo.toml: {}", facade_path.display()),
                io_error: Arc::new(io_err),
            }
        })?;
        let appended = format!(
r#"
[dependencies.{}]
path = "{}"
"#,
            new_crate.name(),
            new_crate.as_ref().display(),
        );
        let updated = format!("{}\n{}", content, appended);
        fs::write(&facade_path, updated).await.map_err(|io_err| {
            RegisterCrateInPrefixGroupError::IoError {
                context: format!("writing facade Cargo.toml: {}", facade_path.display()),
                io_error: Arc::new(io_err),
            }
        })?;

        // 2) Possibly update a `src/lib.rs` or `src/imports.rs` in prefix crate to reexport
        let imports_rs = prefix_crate.as_ref().join("src").join("imports.rs");
        if imports_rs.exists() {
            let mut existing = match fs::read_to_string(&imports_rs).await {
                Ok(txt) => txt,
                Err(e) => {
                    warn!("Could not read imports.rs from {}: {:?}", imports_rs.display(), e);
                    String::new()
                }
            };
            existing.push_str(&format!("pub use {}::*;\n", new_crate.name()));
            fs::write(&imports_rs, existing).await.map_err(|io_err| {
                RegisterCrateInPrefixGroupError::IoError {
                    context: format!("writing prefix facade imports.rs: {}", imports_rs.display()),
                    io_error: Arc::new(io_err),
                }
            })?;
        } else {
            info!("No imports.rs in prefix crate '{}'; skipping reexport step", prefix_crate.name());
        }

        info!("Successfully registered '{}' into prefix facade '{}'", new_crate.name(), prefix_crate.name());
        Ok(())
    }
}
