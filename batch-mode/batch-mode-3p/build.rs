// ---------------- [ File: batch-mode-3p/build.rs ]
use scan_crate_for_typedefs::*;

fn main() -> std::io::Result<()> {

    let _typemap = PersistentWorkspaceTypeMap::new_with_path("..")?;

    Ok(())
}

