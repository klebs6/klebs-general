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
where 
for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
T: CrateHandleInterface<P>
{}

pub trait GetCrates<P,T> 
where 
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    T: CrateHandleInterface<P> 
{
    fn crates(&self) -> &[T];
    fn crates_mut(&mut self) -> &mut Vec<T>;
}

pub trait NumCrates {
    fn n_crates(&self) -> usize;
}
