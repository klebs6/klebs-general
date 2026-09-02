// ---------------- [ File: osx-wallpaper-cycler/src/file_system_operation_kind.rs ]
crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilesystemOperationKind {
    ReadFile,
    WriteFile,
    CreateDirAll,
    RemoveFile,
    Rename,
    CreateFile,
    FlushFile,
}

impl std::fmt::Display for FilesystemOperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile => write!(f, "io.read_file"),
            Self::WriteFile => write!(f, "io.write_file"),
            Self::CreateDirAll => write!(f, "io.create_dir_all"),
            Self::RemoveFile => write!(f, "io.remove_file"),
            Self::Rename => write!(f, "io.rename"),
            Self::CreateFile => write!(f, "io.create_file"),
            Self::FlushFile => write!(f, "io.flush_file"),
        }
    }
}

#[cfg(test)]
mod filesystem_operation_kind_display_contract_suite {
    use super::*;

    #[traced_test]
    fn display_is_stable_for_each_operation_kind() {
        assert_eq!(FilesystemOperationKind::ReadFile.to_string(), "io.read_file");
        assert_eq!(FilesystemOperationKind::WriteFile.to_string(), "io.write_file");
        assert_eq!(
            FilesystemOperationKind::CreateDirAll.to_string(),
            "io.create_dir_all"
        );
        assert_eq!(FilesystemOperationKind::RemoveFile.to_string(), "io.remove_file");
        assert_eq!(FilesystemOperationKind::Rename.to_string(), "io.rename");
        assert_eq!(FilesystemOperationKind::CreateFile.to_string(), "io.create_file");
        assert_eq!(FilesystemOperationKind::FlushFile.to_string(), "io.flush_file");
    }
}
