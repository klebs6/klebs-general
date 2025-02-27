// ---------------- [ File: src/port_try_from.rs ]
crate::ix!();

/// The dual trait: from a source type `Src` into `Self`, given port 0.
pub trait PortTryFrom0<Src>: Sized {
    type Error;
    fn port_try_from0(src: Src) -> Result<Self, Self::Error>;
}

/// The dual trait: from a source type `Src` into `Self`, given port 0.
pub trait PortTryFrom1<Src>: Sized {
    type Error;
    fn port_try_from1(src: Src) -> Result<Self, Self::Error>;
}

/// The dual trait: from a source type `Src` into `Self`, given port 0.
pub trait PortTryFrom2<Src>: Sized {
    type Error;
    fn port_try_from2(src: Src) -> Result<Self, Self::Error>;
}

/// The dual trait: from a source type `Src` into `Self`, given port 0.
pub trait PortTryFrom3<Src>: Sized {
    type Error;
    fn port_try_from3(src: Src) -> Result<Self, Self::Error>;
}
