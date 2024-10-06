#![allow(unused_imports)]

macro_rules! x { 
    ($x:ident) => { 
        mod $x; 
        pub use $x::*; 
    }
}

mod internal {

    #[macro_export] macro_rules! ix { 
        () => { 
            use crate::{ 
                imports::* , 
            };
            use crate::*;
        } 
    }
}

pub mod imports; 

x!{crate_location}
x!{crate_types}
x!{workspace_types}
x!{workspace}
x!{public}
x!{cargo_toml}
x!{saveload}
x!{persist}
