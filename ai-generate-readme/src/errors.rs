crate::ix!();

error_tree!{

    pub enum AIGenerateReadmesError {
        WorkspaceError(WorkspaceError),
    }
}
