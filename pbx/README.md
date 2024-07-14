# pbx

`pbx` is a Rust crate that provides convenient macros for creating various types of boxed and atomic reference-counted values, along with default and zeroed instances. 

The crate is named in honor of its most notable member, the `Pbx` type.

Additionally, this crate can be used as a shorthand to access the following common types: `Pin`, `Arc`, `Mutex`, `Rc`, `RefCell` via the following toplevel `lib.rs` exports:

```rust
pub use std::pin::Pin;
pub use std::sync::{Arc, Mutex};
pub use std::rc::Rc;
pub use std::cell::RefCell;
```

If you find yourself using these types frequently, this crate can be used to simplify things:

```rust
use pbx::*;
```

## Features

- **Pbx**: A type alias for `Pin<Box<T>>`.
- **Macros**: Several macros for creating `Pin<Box<T>>`, `Arc<Mutex<T>>`, `Arc<T>`, default instances, and zeroed instances.

## Usage

Add `pbx` to your `Cargo.toml`:

```toml
[dependencies]
pbx = "0.1.0"

```
## Examples

```rust
extern crate pbx;

use pbx::*;

fn main() {
    // Using pbx macro to create a Pin<Box<T>>
    let boxed_value = pbx!(5);
    println!("Boxed value: {:?}", boxed_value);

    // Using arcmut macro to create an Arc<Mutex<T>>
    let arc_mutex_value = arcmut!(10);
    {
        let mut data = arc_mutex_value.lock().unwrap();
        *data += 1;
    }
    println!("Arc<Mutex> value: {:?}", arc_mutex_value);

    // Using arc macro to create an Arc<T>
    let arc_value = arc!(20);
    println!("Arc value: {:?}", arc_value);

    // Using default macro to create a default instance
    let default_value: i32 = default!();
    println!("Default value: {:?}", default_value);

    // Using zeroed macro to create a zeroed instance
    let zeroed_value: i32 = zeroed!();
    println!("Zeroed value: {:?}", zeroed_value);

    // Using rc macro to create an Rc<T>
    let rc_value = rc!(30);
    println!("Rc value: {:?}", rc_value);

    // Using rcmut macro to create an Rc<RefCell<T>>
    let rc_mut_value = rcmut!(40);
    {
        let mut data = rc_mut_value.borrow_mut();
        *data += 1;
    }
    println!("Rc<RefCell> value: {:?}", rc_mut_value);
}
```

## Macros

`pbx!`

Creates a Pin<Box<T>> from a given expression.

```rust
let pinned_boxed_value = pbx!(value);
```

`arcmut!`

Creates an Arc<Mutex<T>> from a given expression.

```rust
let arc_mutex_value = arcmut!(value);
```

`arcmut_with!`
Creates an Arc<Mutex<T>> with an initializer function.

```rust
let arc_mutex_value = arcmut_with!(value, |v| v + 1);
```

`arc!`

Creates an Arc<T> from a given expression.

```rust
let arc_value = arc!(value);
```

`rc!`
Creates an Rc<T> from a given expression.

```rust
let rc_value = rc!(value);
```

`rcmut!`
Creates an Rc<RefCell<T>> from a given expression.

```rust
let rc_mut_value = rcmut!(value);
```

`default!`

Creates a default instance of a type.

```rust
let default_instance: T = default!();
```

`zeroed!`

Creates a zeroed instance of a type. This is typically used when interacting with wrappers around C APIs which do not have or need default constructors.

```rust
let zeroed_instance: T = zeroed!();
```

## Utilities
`pin_box`
Converts a Box<T> to a Pbx<T>.

```rust
let boxed_value: Box<T> = Box::new(value);
let pinned_boxed_value = pin_box(boxed_value);
```

`pin_arc`
Converts an Arc<T> to a Pin<Arc<T>>.

```rust
let arc_value: Arc<T> = Arc::new(value);
let pinned_arc_value = pin_arc(arc_value);
```

## License

This project is licensed under the MIT License.
