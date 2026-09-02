#[macro_export]
macro_rules! x { 
    ($x:ident) => { 
        mod $x; 
        pub use $x::*; 
    }
}

#[macro_export]
macro_rules! xp { 
    ($x:ident) => { 
        mod $x; 
        use $x::*; 
    }
}

#[macro_export]
macro_rules! ix { 
    () => { 
        use crate::{ 
            imports::* , 
        };
        use crate::*;
    } 
}

#[macro_export]
macro_rules! xt {
    ($name:ident) => {
        #[cfg(test)]
        mod $name;

        #[cfg(test)]
        pub use $name::*;
    };
}
