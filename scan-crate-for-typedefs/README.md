## Overview

This is a simple crate that lets us scan one or
all crates in the current workspace.

We can also use it to scan a vendored crate.

This helps us search a crate directly for trait, fn, struct,
enum, type names, and macro defs.

This crate uses the rust-analyzer API to perform
the heavy lifting.


## Usage

I typically use it like this:

```Cargo.toml
[build-dependencies]
scan-crate-for-typedefs = "0.6.0"
```

```rust
//this is the `build.rs` file for one of the most
// stable crates in the workspace:

use scan_crate_for_typedefs::*;

fn main() -> std::io::Result<()> {

    let typemap = PersistentWorkspaceTypeMap::new_with_path("..")?;

    Ok(())
}
```


Then, all we have to do is build the project and
we get a `rust-workspace-typemap.json` at the
top-level

I typically parse the output of cargo build to
find types which cannot be found.

Next, I scan the index contained within file to
figure out which crate they belong to.
