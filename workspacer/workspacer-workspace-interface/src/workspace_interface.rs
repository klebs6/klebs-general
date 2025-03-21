// ---------------- [ File: workspacer-workspace-interface/src/workspace_interface.rs ]
crate::ix!();

pub trait WorkspaceInterface<P,T>
: GetCrates<P,T>
+ Send
+ Sync
+ NumCrates
+ ValidateIntegrity
+ AsyncTryFrom<P>
+ AsyncPathValidator
+ AsyncFindItems
+ AsRef<Path>
+ GetAllCrateNames
+ FindCrateByName<P,T>
where 
for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
T: CrateHandleInterface<P>
{}

pub trait GetCrates<P,T> 
where 
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    T: CrateHandleInterface<P> 
{
    fn crates(&self) -> &[Arc<Mutex<T>>];
}

pub trait NumCrates {
    fn n_crates(&self) -> usize;
}

pub trait FindCrateByName<P,T> 
where 
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    T: CrateHandleInterface<P> 
{
    fn find_crate_by_name(&self, name: &str) -> Option<Arc<Mutex<T>>>;
}

pub trait GetAllCrateNames {
    fn get_all_crate_names(&self) -> Vec<String>;
}
