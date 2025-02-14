crate::ix!();

#[derive(Clone,PartialEq,Eq,Debug)]
pub enum DowncastError {
    /// Returned when the erased value does not match the requested concrete type.
    TypeMismatch,
    /// Reserved for future expansions.
    Other,
}

pub struct Erased {
    /// Pointer to a heap-allocated instance of some generic type `T`.
    data: core::ptr::NonNull<()>,
    /// Reference to a vtable that knows how to drop and identify `T`.
    vtable: &'static VTable,
}

/// A basic table of function pointers for erasing a type `T`.
struct VTable {
    /// Function pointer used to destruct the erased data correctly.
    drop_fn: unsafe fn(*mut ()),
    /// Uniquely identifies the stored type `T` (via pointer comparison).
    type_marker: *const (),
}

/// A "once" container storing a `VTable`. Each unique monomorphization
/// gets its own static `OnceVTable` and `initialized` flag.
struct OnceVTable {
    /// Slot that is populated at most once, after which it is never mutated.
    vtable: core::cell::UnsafeCell<Option<VTable>>,
    /// Flag indicating whether `vtable` has been populated.
    initialized: core::sync::atomic::AtomicBool,
}

// Safety: `OnceVTable` only stores function pointers and no references to `T`.
unsafe impl Sync for OnceVTable {}

impl OnceVTable {
    const fn new() -> Self {
        Self {
            vtable: core::cell::UnsafeCell::new(None),
            initialized: core::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Returns a reference to the per-type vtable, initializing it on first call.
    fn init<T>(&self) -> &'static VTable {
        use core::sync::atomic::Ordering;
        if !self.initialized.load(Ordering::Acquire) {
            let drop_fn = drop_impl::<T>;
            let marker = unique_type_marker_for::<T>();
            let new_vtable = VTable {
                drop_fn,
                type_marker: marker,
            };
            // Safety: only the first thread to succeed will fill in the VTable.
            // All others store the exact same `new_vtable`, so it is effectively identical.
            unsafe {
                *self.vtable.get() = Some(new_vtable);
            }
            self.initialized.store(true, Ordering::Release);
        }
        // Safety: once `.initialized == true`, we must have stored Some(VTable).
        let stored = unsafe { &*self.vtable.get() };
        stored.as_ref().expect("OnceVTable must be initialized")
    }
}

impl Erased {
    /// Create an `Erased` from a generic value `T`, storing it on the heap.
    ///
    /// # Safety
    /// - We rely on a raw pointer to hold `T`.
    /// - We produce no references into `T`, thus interior references remain safe
    ///   as long as we haven't dropped or reused the memory incorrectly.
    pub fn new<T>(value: T) -> Self {
        let boxed = Box::new(value);
        let raw = Box::into_raw(boxed) as *mut ();
        // SAFETY: into_raw() never yields null.
        let nonnull = unsafe { core::ptr::NonNull::new_unchecked(raw) };
        Self {
            data: nonnull,
            vtable: get_vtable::<T>(),
        }
    }

    /// Attempt to reify the stored object back into `T`.
    /// Returns `Ok(T)` on success or `Err(DowncastError::TypeMismatch)` if `T` doesn't match.
    pub fn downcast<T>(self) -> Result<T, DowncastError> {
        if core::ptr::eq(self.vtable, get_vtable::<T>()) {
            // It's the same type, so retrieve the original T.
            let ptr = self.data.as_ptr();
            core::mem::forget(self); // Don't run Drop again on this pointer.
            let boxed = unsafe { Box::from_raw(ptr as *mut T) };
            Ok(*boxed)
        } else {
            Err(DowncastError::TypeMismatch)
        }
    }
}

impl Drop for Erased {
    fn drop(&mut self) {
        // SAFETY: The pointer + drop_fn match the original type set in `Erased::new`.
        unsafe {
            (self.vtable.drop_fn)(self.data.as_ptr());
        }
    }
}

/// Retrieve or construct a unique `VTable` for a particular type `T`.
fn get_vtable<T>() -> &'static VTable {
    // Each monomorphization has its own static OnceVTable => unique place to store the `VTable`.
    static VTABLE_FOR_T: OnceVTable = OnceVTable::new();
    VTABLE_FOR_T.init::<T>()
}

/// Specialized drop function that reconstitutes a `Box<T>` from a raw pointer
/// and drops it appropriately.
unsafe fn drop_impl<T>(ptr: *mut ()) {
    let _ = unsafe { Box::<T>::from_raw(ptr as *mut T) };
    // `_` binding so we drop it immediately.
}

/// We manufacture a unique pointer for each distinct `T` by referencing
/// a per-`T` monomorphized static. This is a common trick (though not
/// strictly guaranteed by the Rust spec), but in practice it ensures
/// pointer identity is unique per type.
///
/// We use a zero-sized struct with a `PhantomData<T>` to force a unique
/// monomorphization for each `T`. Then we store it in a `static` so
/// that it has a unique address.
#[inline(never)]
fn unique_type_marker_for<T>() -> *const () {
    use core::marker::PhantomData;

    struct Marker<T>(PhantomData<T>);
    static MARKER: Marker<()> = Marker(PhantomData);

    // Even though `Marker<()>` is ZST, each monomorphization site in `unique_type_marker_for`
    // gets its own code instance with a distinct `static MARKER`. The address of this
    // `static` is unique. We just cast it to *const () for storage in the vtable.
    &MARKER as *const Marker<()> as *const ()
}

#[cfg(test)]
mod tests_erased {
    use super::*;

    // A custom struct for testing.
    struct Foo {
        /// Arbitrary numeric field
        _x: i32,
    }

    // A custom struct that can hold references, to ensure we do not require 'static.
    struct Bar<'a> {
        /// A reference to an i32
        _r: &'a i32,
    }

    #[test]
    fn test_downcast_u32_success() {
        let e = Erased::new(42_u32);
        match e.downcast::<u32>() {
            Ok(n) => assert_eq!(n, 42),
            Err(_) => panic!("Expected to succeed downcasting a u32"),
        }
    }

    #[test]
    #[should_panic(expected = "Should not downcast to u32")]
    fn test_downcast_u32_failure() {
        let e = Erased::new("hello world");
        match e.downcast::<u32>() {
            Ok(_) => panic!("Should not downcast to u32"),
            Err(DowncastError::TypeMismatch) => (),
            Err(_) => panic!("Unexpected error variant"),
        }
    }

    #[test]
    fn test_downcast_str_success() {
        let e = Erased::new("lorem ipsum");
        match e.downcast::<&str>() {
            Ok(s) => assert_eq!(s, "lorem ipsum"),
            Err(_) => panic!("Should have downcast to &str"),
        }
    }

    #[test]
    fn test_downcast_custom_struct() {
        let foo = Foo { _x: 100 };
        let e = Erased::new(foo);
        match e.downcast::<Foo>() {
            Ok(foo_val) => assert_eq!(foo_val._x, 100),
            Err(_) => panic!("Should have downcast to Foo"),
        }
    }

    #[test]
    fn test_reference_fields_no_static_bounds() {
        let data = 123;
        let bar = Bar { _r: &data };
        let e = Erased::new(bar);
        match e.downcast::<Bar>() {
            Ok(bar_val) => assert_eq!(*bar_val._r, 123),
            Err(_) => panic!("Should have downcast to Bar"),
        }
    }

    #[test]
    fn test_multiple_erased_objects() {
        let e1 = Erased::new(10_i64);
        let e2 = Erased::new("some string");
        let e3 = Erased::new(Foo { _x: -999 });

        // e1 -> i64
        match e1.downcast::<i64>() {
            Ok(val) => assert_eq!(val, 10),
            Err(_) => panic!("Should downcast i64"),
        }

        // e2 -> &str
        match e2.downcast::<&str>() {
            Ok(s) => assert_eq!(s, "some string"),
            Err(_) => panic!("Should downcast &str"),
        }

        // e3 -> Foo
        match e3.downcast::<Foo>() {
            Ok(foo) => assert_eq!(foo._x, -999),
            Err(_) => panic!("Should downcast Foo"),
        }
    }
}
