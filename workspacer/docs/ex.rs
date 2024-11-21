
/// Returns the `PathBuf` to the parent directory's `Cargo.toml` file.
///
/// This function will panic if the parent directory's `Cargo.toml` does not exist.
///
/// # Panics
///
/// - Panics if it fails to get the current directory.
/// - Panics if the parent directory's `Cargo.toml` does not exist.
///
/// # Examples
///
/// ```no_run
/// use scan_crate_for_typedefs::*;
///
/// let path = parent_cargo_toml();
/// ```
pub fn parent_cargo_toml() -> PathBuf {
    ...
}

/// Returns the `PathBuf` to the current directory's `Cargo.toml` file.
///
/// # Panics
///
/// - Panics if it fails to get the current directory.
///
/// # Examples
///
/// ```no_run
/// use scan_crate_for_typedefs::*;
///
/// let path = current_cargo_toml();
/// ```
pub fn current_cargo_toml() -> PathBuf {
    ...
}

/// Retrieves the crate name from a given `Cargo.toml` file path.
///
/// # Errors
///
/// Returns an `io::Result` which is an `Err` if:
///
/// - The `Cargo.toml` file could not be opened.
/// - The content of `Cargo.toml` could not be parsed.
/// - The `name` field is not found in the `[package]` section.
///
/// # Examples
///
/// ```no_run
/// use scan_crate_for_typedefs::*;
///
/// let name = get_crate_name_from_cargo_toml("path/to/Cargo.toml").unwrap();
/// ```
pub fn get_crate_name_from_cargo_toml(path: &str) -> io::Result<String> {
    ...
}

/// Retrieves the parent directory path from a given `Cargo.toml` file path.
///
/// # Errors
///
/// Returns an `io::Result` which is an `Err` if:
///
/// - The `path` could not be converted to a `Path` object.
/// - The parent directory path could not be converted to a `String`.
///
/// # Examples
///
/// ```rust
/// use scan_crate_for_typedefs::*;
///
/// let parent_dir = get_parent_directory_from_cargo_toml_path("path/to/Cargo.toml").unwrap();
/// ```
///
pub fn get_parent_directory_from_cargo_toml_path(path: &str) -> io::Result<String> {
    ...
}


/// Represents the location of a Rust crate in
/// various scenarios.
///
/// This enum can handle crates that are vendored,
/// in the current workspace, or identified by
/// a direct filesystem path.
///
#[derive(Debug)]
pub enum CrateLocation<'a> {

    /// Represents a vendored crate with a given
    /// name.
    ///
    Vendored {
        crate_name: &'a str,
    },

    /// Represents a crate within the current
    /// workspace with a given name.
    ///
    InCurrentWorkspace {
        crate_name: &'a str,
    },

    /// Represents a crate identified by the
    /// absolute path to its Cargo.toml.
    ///
    DirectPath {
        absolute_path_to_cargo_toml: String,
    }
}

impl<'a> CrateLocation<'a> {

    /// Returns the name of the crate.
    ///
    /// This function extracts the crate name
    /// based on the variant of `CrateLocation`.
    ///
    pub fn name(&self) -> String {
        ...
    }

    /// Returns the root directory of the crate.
    ///
    /// The returned path depends on the variant
    /// of `CrateLocation`
    ///
    pub fn root(&self) -> String {
        ...
    }

    /// Retrieves the contents of all source files
    /// in the crate.
    ///
    /// This function walks the directory tree
    /// starting from the crate's root and reads
    /// the content of all `.rs` files.
    ///
    pub fn all_source_file_contents(&self) -> Vec<String> {
        ...
    }
}


#[macro_export] macro_rules! example_macro { 
    ($x:ident) => { 
        ...
    }
}


/// Represents various types of items in a Rust crate.
///
/// Holds sets of names for traits, functions, structs,
/// enums, and other types that exist in the crate.
///
#[derive(PartialEq,Serialize,Deserialize,Debug,Clone)]
pub struct CrateTypes {

    /// Name of the crate.
    pub crate_name: String,

    /// Set of public traits in the crate.
    pub traits:     HashSet<String>,

    /// Set of public functions in the crate.
    pub fns:        HashSet<String>,

    /// Set of public structs in the crate.
    pub structs:    HashSet<String>,

    /// Set of public enums in the crate.
    pub enums:      HashSet<String>,

    /// Set of other public types in the crate (e.g., type aliases).
    pub types:      HashSet<String>,

    /// Set of public macros in the crate
    pub macros:     HashSet<String>,
}

impl Default for CrateTypes {

    /// Creates a default `CrateTypes` instance with empty
    /// sets and "<unknown>" as the crate name.
    ///
    fn default() -> Self {
        ...
    }
}

impl CrateTypes {

    /// Checks if all item sets are empty.
    ///
    /// Returns `true` if all sets (traits, fns, structs,
    /// enums, types) are empty.
    ///
    pub fn empty(&self) -> bool {
        ...
    }
}

impl TryFrom<&str> for CrateTypes {

    type Error = std::io::Error;

    /// Attempts to create a `CrateTypes` instance from
    /// a path string.
    ///
    /// The function will try to canonicalize the path and
    /// then generate the types from it.
    ///
    fn try_from(path: &str) -> std::io::Result<Self> {
        ...
    }
}

impl TryFrom<&PathBuf> for CrateTypes {

    type Error = std::io::Error;

    /// Attempts to create a `CrateTypes` instance from
    /// a `PathBuf`.
    ///
    /// The function will try to canonicalize the path and
    /// then generate the types from it.
    ///
    fn try_from(path: &PathBuf) -> std::io::Result<Self> {
        ...
    }
}

impl From<&CrateLocation<'_>> for CrateTypes {

    /// Creates a `CrateTypes` instance from
    /// a `CrateLocation`.
    ///
    /// This function will inspect the source files in the
    /// crate location to populate the sets of traits,
    /// functions, structs, enums, and types.
    ///
    fn from(loc: &CrateLocation<'_>) -> Self {
        ...
    }
}

/// Default JSON file name for storing the workspace type map.
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
        ...
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
        ...
    }

    /// Constructs a new `PersistentWorkspaceTypeMap` using
    /// the current directory.
    ///
    pub fn new() -> io::Result<Self> {
        ...
    }
}

impl Drop for PersistentWorkspaceTypeMap {

    /// When a `PersistentWorkspaceTypeMap` object is
    /// dropped, this method attempts to save it as a JSON
    /// file if it was built from scratch.
    ///
    fn drop(&mut self) {
        ...
    }
}

/// Determines if a given syntax node in a Rust AST represents a public entity.
///
/// This function checks the visibility of various kinds of syntax nodes
/// that can be part of a Rust Abstract Syntax Tree (AST), and returns
/// `true` if the node is marked as public (`pub`), or `false` otherwise.
///
/// Currently, this function handles the following kinds of syntax nodes:
/// - Functions
/// - Structs
/// - Enums
/// - Traits
/// - Type Aliases
///
/// # Arguments
///
/// - `node`: A reference to the `SyntaxNode` being checked for visibility.
///
/// # Returns
///
/// Returns `true` if the syntax node is marked as public, `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use scan_crate_for_typedefs::is_node_public;
///
/// use ra_ap_syntax::SyntaxNode; // Please replace with the actual import
///
/// let node = todo!();/* Obtain the SyntaxNode instance somehow */;
///
/// let is_public = is_node_public(&node);
/// ```
///
/// # Note
///
/// More syntax node kinds can be added in the future to extend the function's functionality.
pub fn is_node_public(node: &SyntaxNode) -> bool {
    ...
}

/// Saves the `WorkspaceTypes` instance to a JSON file at the specified path.
///
/// # Arguments
///
/// - `workspace_types`: A reference to the `WorkspaceTypes` instance to save.
/// - `path`: The path to the file where the `WorkspaceTypes` instance will be saved.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the operation.
pub fn save_workspace_types<P: AsRef<Path>>(workspace_types: &WorkspaceTypes, path: P) 
    -> std::io::Result<()> 
{
    ...
}

/// Loads a `WorkspaceTypes` instance from a JSON file at the specified path.
///
/// # Arguments
///
/// - `path`: The path to the JSON file to load the `WorkspaceTypes` instance from.
///
/// # Returns
///
/// Returns a `Result` containing the loaded `WorkspaceTypes` instance on success,
/// or an error on failure.
pub fn load_workspace_types<P: AsRef<Path>>(path: P) -> std::io::Result<WorkspaceTypes> {
    ...
}

/// Prints the content of the file located at the specified path to the standard output.
///
/// # Arguments
///
/// - `path`: The path to the file whose content will be printed.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the operation.
pub fn cat_file_to_screen<P: AsRef<Path>>(path: P) -> io::Result<()> {
    ...
}

/// Fetches the list of member crate names from a given workspace-level `Cargo.toml` file.
///
/// # Parameters
///
/// - `path`: A path to the workspace's `Cargo.toml` file. The path can be a string slice, `String`,
/// or anything that implements `AsRef<Path>`.
///
/// # Returns
///
/// Returns a `Vec<String>` containing the names of all member crates in the workspace.
///
/// # Panics
///
/// - Panics if the file at `path` cannot be read.
/// - Panics if the content of `Cargo.toml` cannot be parsed.
/// - Panics if the `Cargo.toml` does not have a `[workspace]` section.
/// - Panics if the `[workspace]` section does not have a `members` field.
/// - Panics if the `members` field is not an array.
///
/// # Examples
///
/// ```no_run
///
///  use scan_crate_for_typedefs::get_workspace_members;
///
/// let members = get_workspace_members("path/to/workspace/Cargo.toml");
/// println!("{:?}", members);
/// ```
pub fn get_workspace_members<P: AsRef<Path>>(path: P) -> Vec<String> {
    ...
}

/// A mapping between workspace crate names and the types
/// they define, along with an index for quick lookups.
///
/// `typemap`: A hash map where the key is the name of
/// a crate in the workspace, and the value is the types it
/// defines.
///
/// `index`: An index that maps a type name to a vector of
/// crates that define or use this type.
///
#[derive(PartialEq,Serialize,Deserialize,Default,Debug)]
pub struct WorkspaceTypes {
    typemap: HashMap<String,CrateTypes>,
    index:   HashMap<String, Vec<String>>,
}

impl WorkspaceTypes {

    /// Constructs a new `WorkspaceTypes` instance by
    /// reading a workspace-level `Cargo.toml` and gathering
    /// types from its member crates.
    ///
    pub fn from_cargo_toml<P: AsRef<Path>>(path_to_workspace_cargo_toml: P) -> std::io::Result<Self> {
        ...
    }

    /// Loads `WorkspaceTypes` from a JSON file.
    ///
    pub fn load_from_json<P: AsRef<Path>>(path_to_json: P) -> std::io::Result<Self> {
        ...
    }

    /// Saves the `WorkspaceTypes` instance to a JSON file.
    ///
    pub fn save_to_json<P: AsRef<Path>>(&self, path_to_json: P) -> std::io::Result<()> {
        ...
    }

    /// Searches for crates that define or use a given symbol name.
    ///
    pub fn find_crates_by_symbol(&self, name: &str) -> Option<Vec<String>> {
        ...
    }

    /// Searches for crates that define or use a given trait name.
    ///
    pub fn find_crates_by_trait(&self, trait_name: &str) -> Option<Vec<String>> {
        ...
    }

    /// Searches for crates that define or use a given fn name.
    ///
    pub fn find_crates_by_fn(&self, fn_name: &str) -> Option<Vec<String>> {
        ...
    }

    /// Searches for crates that define or use a given struct name.
    ///
    pub fn find_crates_by_struct(&self, struct_name: &str) -> Option<Vec<String>> {
        ...
    }

    /// Searches for crates that define or use a given enum name.
    ///
    pub fn find_crates_by_enum(&self, enum_name: &str) -> Option<Vec<String>> {
        ...
    }

    /// Searches for crates that define or use a given type name.
    ///
    pub fn find_crates_by_type(&self, type_name: &str) -> Option<Vec<String>> {
        ...
    }

    /// Searches for crates that define or use a given macro
    ///
    pub fn find_crates_by_macro(&self, macro_name: &str) -> Option<Vec<String>> {
        ...
    }

    fn find_crates_by<F>(&self, name: &str, selector: F) -> Option<Vec<String>>
        where F: Fn(&CrateTypes) -> &HashSet<String>,
    {
        ...
    }

    /// Inserts a new crate along with its `CrateTypes` into
    /// the `WorkspaceTypes` instance, updating the index
    /// accordingly.
    ///
    /// Call this method whenever you add a new CrateTypes
    /// object to your WorkspaceTypes
    ///
    pub fn insert(&mut self, workspace_crate: &str, types: CrateTypes) {
        ...
    }
}
