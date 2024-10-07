crate::ix!();

error_tree!{

    pub enum RckError {
        GitError(git2::Error),
    }
}
