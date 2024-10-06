crate::ix!();

#[macro_export] macro_rules! example_macro { 
    ($x:ident) => { 
        todo!();
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
        Self {
            crate_name: "<unknown>".to_string(),
            traits:     HashSet::new(),
            fns:        HashSet::new(),
            structs:    HashSet::new(),
            enums:      HashSet::new(),
            types:      HashSet::new(),
            macros:     HashSet::new(),
        }
    }
}

impl CrateTypes {

    /// Checks if all item sets are empty.
    ///
    /// Returns `true` if all sets (traits, fns, structs,
    /// enums, types) are empty.
    ///
    pub fn empty(&self) -> bool {
        [
            self.traits.len()  == 0,
            self.fns.len()     == 0,
            self.structs.len() == 0,
            self.enums.len()   == 0,
            self.types.len()   == 0,
            self.macros.len()   == 0,
        ].iter().all(|x| *x == true)
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

        let pathbuf = PathBuf::from(path);

        Self::try_from(&pathbuf)
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

        let absolute_path = path.canonicalize()?;

        // Convert it to a String
        let absolute_path_str = absolute_path
            .to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Non-UTF8 path"))?;

        let absolute_path_string = absolute_path_str.to_string();

        let crate_location = CrateLocation::DirectPath {
            absolute_path_to_cargo_toml: absolute_path_string,
        };

        Ok(CrateTypes::from(&crate_location))
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

        let mut traits  = HashSet::new();
        let mut fns     = HashSet::new();
        let mut structs = HashSet::new();
        let mut enums   = HashSet::new();
        let mut types   = HashSet::new();
        let mut macros  = HashSet::new();

        let sources = loc.all_source_file_contents();

        for source in sources.iter() {

            let parse = SourceFile::parse(source,Edition::Edition2024).syntax_node();

            for node in parse.descendants() {
                match_ast! {
                    match node {
                        ast::Fn(it) => {

                            if let Some(name) = it.name() {
                                if is_node_public(&node) {
                                    fns.insert(name.text().to_string());
                                }
                            }
                        },
                        ast::Struct(it) => {

                            if let Some(name) = it.name() {
                                if is_node_public(&node) {
                                    structs.insert(name.text().to_string());
                                }
                            }
                        },
                        ast::Enum(it) => {

                            if let Some(name) = it.name() {
                                if is_node_public(&node) {
                                    enums.insert(name.text().to_string());
                                }
                            }
                        },
                        ast::Trait(it) => {

                            if let Some(name) = it.name() {
                                if is_node_public(&node) {
                                    traits.insert(name.text().to_string());
                                }
                            }
                        },
                        ast::TypeAlias(it) => {

                            if let Some(name) = it.name() {
                                if is_node_public(&node) {
                                    types.insert(name.text().to_string());
                                }
                            }
                        },
                        ast::MacroRules(it) => {

                            if let Some(name) = it.name() {
                                if is_node_public(&node) {
                                    macros.insert(name.text().to_string());
                                }
                            }
                        },
                        _ => {
                            //println!("found unprocessed node {}", node);
                        },
                    }
                }
            }
        }

        for vec in [&mut fns, &mut structs, &mut enums, &mut traits, &mut types, &mut macros].iter_mut()
        {
            vec.retain(|s| 
                s.len() > 5 
                && !s.starts_with("test_")
                && !s.starts_with("_")
            );
        }

        CrateTypes {
            crate_name: loc.name(),
            traits,
            fns,
            structs,
            enums,
            types,
            macros,
        }
    }
}
