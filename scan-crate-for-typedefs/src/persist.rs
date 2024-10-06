crate::ix!();

/// Default JSON file name for storing the workspace type map.
///
pub const WORKSPACE_TYPEMAP_DEFAULT_PERSISTANCE_FILE: &'static str = "rust-workspace-typemap.json";

/// A wrapper around `WorkspaceTypes` with persistence features.
///
/// `PersistentWorkspaceTypeMap` contains a `WorkspaceTypes` object and
/// adds functionality to load it from and save it to a JSON file.
///
/// The path to the JSON file is stored in the `path` field.
///
#[derive(Debug)]
pub struct PersistentWorkspaceTypeMap {
    inner:              WorkspaceTypes,
    path:               PathBuf,
    built_from_scratch: bool,
}

impl PersistentWorkspaceTypeMap {

    // Delegate some methods to the inner `WorkspaceTypes` object.
    delegate! {
        to self.inner {

            /// Finds crates that contain a specified symbol.
            pub fn find_crates_by_symbol(&self, name: &str) -> Option<Vec<String>>;

            /// Finds crates that define a specified trait.
            pub fn find_crates_by_trait(&self, trait_name: &str) -> Option<Vec<String>>;

            /// Finds crates that define a specified fn.
            pub fn find_crates_by_fn(&self, fn_name: &str) -> Option<Vec<String>>;

            /// Finds crates that define a specified struct.
            pub fn find_crates_by_struct(&self, struct_name: &str) -> Option<Vec<String>>;

            /// Finds crates that define a specified enum.
            pub fn find_crates_by_enum(&self, enum_name: &str) -> Option<Vec<String>>;

            /// Finds crates that define a specified type.
            pub fn find_crates_by_type(&self, type_name: &str) -> Option<Vec<String>>;

            /// Finds crates that define a specified macro.
            pub fn find_crates_by_macro(&self, macro_name: &str) -> Option<Vec<String>>;
        }
    }

    /// Returns `true` if the `PersistentWorkspaceTypeMap`
    /// was built from scratch.
    ///
    pub fn built_from_scratch(&self) -> bool {
        self.built_from_scratch
    }

    /// Constructs a new `PersistentWorkspaceTypeMap` with
    /// a specified path.
    ///
    /// It attempts to read an existing JSON file from the
    /// provided path.
    ///
    /// If that fails, it generates a new `WorkspaceTypes`
    /// object based on the Cargo.toml at that path.
    ///
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {

        let json_path = Path::join(path.as_ref(), WORKSPACE_TYPEMAP_DEFAULT_PERSISTANCE_FILE);

        if json_path.exists() {

            let loaded = WorkspaceTypes::load_from_json(&json_path)?;

            Ok(Self {
                inner:              loaded,
                path:               json_path,
                built_from_scratch: false,
            })

        } else {

            let cargo_toml_path = Path::join(path.as_ref(), "Cargo.toml");

            let generated = WorkspaceTypes::from_cargo_toml(&cargo_toml_path)?;

            Ok(Self {
                inner:              generated,
                path:               json_path,
                built_from_scratch: true,
            })
        }
    }

    /// Constructs a new `PersistentWorkspaceTypeMap` using
    /// the current directory.
    ///
    pub fn new() -> io::Result<Self> {

        let cwd = std::env::current_dir()?;

        Self::new_with_path(&cwd)
    }
}

impl Drop for PersistentWorkspaceTypeMap {

    /// When a `PersistentWorkspaceTypeMap` object is
    /// dropped, this method attempts to save it as a JSON
    /// file if it was built from scratch.
    ///
    fn drop(&mut self) {

        if self.built_from_scratch {

            match self.inner.save_to_json(&self.path) {
                Ok(_) => println!("Successfully saved to {}", self.path.display()),
                Err(e) => eprintln!("Failed to save: {}", e),
            }
        }
    }
}
