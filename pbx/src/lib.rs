pub use std::pin::Pin;
pub use std::sync::{Arc, Mutex};
pub use std::rc::Rc;
pub use std::cell::RefCell;

pub type Pbx<T> = Pin<Box<T>>;

#[macro_export] macro_rules! pbx {
    ($stuff:expr) => {
        Pin::new(Box::new($stuff))
    }
}

/// Converts a Box<T> to a Pin<Box<T>>
pub fn pin_box<T: Unpin>(value: Box<T>) -> Pbx<T> {
    Pin::new(value)
}

/// Converts an Arc<T> to a Pin<Arc<T>>
pub fn pin_arc<T>(value: Arc<T>) -> Pin<Arc<T>> {
    unsafe { Pin::new_unchecked(value) }
}


#[macro_export] macro_rules! arcmut { 
    ($v:expr) => { 
        Arc::new(Mutex::new($v))
    }
}

#[macro_export] macro_rules! arcmut_with {
    ($v:expr, $f:expr) => {
        Arc::new(Mutex::new($f($v)))
    }
}

#[macro_export] macro_rules! arc { 
    ($v:expr) => { 
        Arc::new($v)
    }
}

#[macro_export] macro_rules! rc {
    ($v:expr) => {
        Rc::new($v)
    }
}

#[macro_export] macro_rules! rcmut {
    ($v:expr) => {
        Rc::new(RefCell::new($v))
    }
}



/// construct me one default, please!
///
#[macro_export] macro_rules! default { 
    () => { 
        Default::default()
    }
}

/// we typically use this when interactinge with
/// wrappers around C apis which do not have/need
/// default constructors
///
#[macro_export] macro_rules! zeroed {
    () => {
        unsafe { std::mem::zeroed() }
    }
}
