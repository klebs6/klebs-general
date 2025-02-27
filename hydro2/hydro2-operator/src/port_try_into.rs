// ---------------- [ File: src/port_try_into.rs ]
crate::ix!();

/// A local trait for "port-aware" conversions
pub trait PortTryInto0<T> {
    type Error;
    /// Convert `self` into a `T`, using `port` as needed.
    fn port_try_into0(self) -> Result<T, Self::Error>;
}

/// A local trait for "port-aware" conversions
pub trait PortTryInto1<T> {
    type Error;
    /// Convert `self` into a `T`, using `port` as needed.
    fn port_try_into1(self) -> Result<T, Self::Error>;
}

/// A local trait for "port-aware" conversions
pub trait PortTryInto2<T> {
    type Error;
    /// Convert `self` into a `T`, using `port` as needed.
    fn port_try_into2(self) -> Result<T, Self::Error>;
}

/// A local trait for "port-aware" conversions
pub trait PortTryInto3<T> {
    type Error;
    /// Convert `self` into a `T`, using `port` as needed.
    fn port_try_into3(self) -> Result<T, Self::Error>;
}
