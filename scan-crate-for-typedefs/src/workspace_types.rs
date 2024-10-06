crate::ix!();

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

        let mut x = Self {
            typemap: HashMap::new(),
            index:   HashMap::new(),
        };

        // get_workspace_members is assumed to be defined and working as expected
        let members = get_workspace_members(&path_to_workspace_cargo_toml);

        for member in &members {

            // Form the path to the member's Cargo.toml
            let mut pathbuf = PathBuf::from(format!("../{}", member));
            pathbuf.push("Cargo.toml");

            // Convert it to a canonicalized, absolute path
            let loc = pathbuf.canonicalize()?
                .as_os_str()
                .to_str()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?
                .to_string();

            // Initialize CrateTypes for this crate
            let types = CrateTypes::try_from(loc.as_str())?;

            // Insert it into the WorkspaceTypes object
            x.insert(member, types);
        }

        Ok(x)
    }

    /// Loads `WorkspaceTypes` from a JSON file.
    ///
    pub fn load_from_json<P: AsRef<Path>>(path_to_json: P) -> std::io::Result<Self> {
        let file = File::open(path_to_json)?;
        let reader = BufReader::new(file);
        let workspace_types = from_reader(reader).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to deserialize JSON: {}", e),
            )
        })?;
        Ok(workspace_types)
    }

    /// Saves the `WorkspaceTypes` instance to a JSON file.
    ///
    pub fn save_to_json<P: AsRef<Path>>(&self, path_to_json: P) -> std::io::Result<()> {
        let file = File::create(path_to_json)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize to JSON: {}", e),
            )
        })
    }

    /// Searches for crates that define or use a given symbol name.
    ///
    pub fn find_crates_by_symbol(&self, name: &str) -> Option<Vec<String>> {

        let mut results = Vec::new();

        for (crate_name, crate_types) in &self.typemap {
            if crate_types.traits.contains(name)
                || crate_types.fns.contains(name)
                    || crate_types.structs.contains(name)
                    || crate_types.enums.contains(name)
                    || crate_types.types.contains(name)
                    || crate_types.macros.contains(name)
                {
                    results.push(crate_name.clone());
                }
        }
        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    /// Searches for crates that define or use a given trait name.
    ///
    pub fn find_crates_by_trait(&self, trait_name: &str) -> Option<Vec<String>> {
        self.find_crates_by(trait_name, |c| &c.traits)
    }

    /// Searches for crates that define or use a given fn name.
    ///
    pub fn find_crates_by_fn(&self, fn_name: &str) -> Option<Vec<String>> {
        self.find_crates_by(fn_name, |c| &c.fns)
    }

    /// Searches for crates that define or use a given struct name.
    ///
    pub fn find_crates_by_struct(&self, struct_name: &str) -> Option<Vec<String>> {
        self.find_crates_by(struct_name, |c| &c.structs)
    }

    /// Searches for crates that define or use a given enum name.
    ///
    pub fn find_crates_by_enum(&self, enum_name: &str) -> Option<Vec<String>> {
        self.find_crates_by(enum_name, |c| &c.enums)
    }

    /// Searches for crates that define or use a given type name.
    ///
    pub fn find_crates_by_type(&self, type_name: &str) -> Option<Vec<String>> {
        self.find_crates_by(type_name, |c| &c.types)
    }

    /// Searches for crates that define or use a given macro
    ///
    pub fn find_crates_by_macro(&self, macro_name: &str) -> Option<Vec<String>> {
        self.find_crates_by(macro_name, |c| &c.macros)
    }

    fn find_crates_by<F>(&self, name: &str, selector: F) -> Option<Vec<String>>
        where F: Fn(&CrateTypes) -> &HashSet<String>,
    {
        let mut results = Vec::new();
        for (crate_name, crate_types) in &self.typemap {
            if selector(crate_types).contains(name) {
                results.push(crate_name.clone());
            }
        }
        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    /// Inserts a new crate along with its `CrateTypes` into
    /// the `WorkspaceTypes` instance, updating the index
    /// accordingly.
    ///
    /// Call this method whenever you add a new CrateTypes
    /// object to your WorkspaceTypes
    ///
    pub fn insert(&mut self, workspace_crate: &str, types: CrateTypes) {

        // Update the index for traits
        for trait_name in types.traits.iter() {
            self.index
                .entry(trait_name.clone())
                .or_insert_with(Vec::new)
                .push(workspace_crate.to_string());
        }

        // Update the index for fns
        for fn_name in types.fns.iter() {
            self.index
                .entry(fn_name.clone())
                .or_insert_with(Vec::new)
                .push(workspace_crate.to_string());
        }

        // Update the index for structs
        for struct_name in types.structs.iter() {
            self.index
                .entry(struct_name.clone())
                .or_insert_with(Vec::new)
                .push(workspace_crate.to_string());
        }

        // Update the index for enums
        for enum_name in types.enums.iter() {
            self.index
                .entry(enum_name.clone())
                .or_insert_with(Vec::new)
                .push(workspace_crate.to_string());
        }

        // Update the index for types
        for type_name in types.types.iter() {
            self.index
                .entry(type_name.clone())
                .or_insert_with(Vec::new)
                .push(workspace_crate.to_string());
        }

        // Update the index for macros
        for macro_name in types.macros.iter() {
            self.index
                .entry(macro_name.clone())
                .or_insert_with(Vec::new)
                .push(workspace_crate.to_string());
        }

        // Insert in the main typemap
        self.typemap.insert(workspace_crate.to_string(), types.clone());
    }
}
