// ---------------- [ File: src/port_try_into_any.rs ]
crate::ix!();

pub trait PortTryInto0Any {
    type Error;
    fn port_try_into0_any(self) -> Result<unsafe_erased::Erased, Self::Error>;
}

pub trait PortTryInto1Any {
    type Error;
    fn port_try_into1_any(self) -> Result<unsafe_erased::Erased, Self::Error>;
}

pub trait PortTryInto2Any {
    type Error;
    fn port_try_into2_any(self) -> Result<unsafe_erased::Erased, Self::Error>;
}

pub trait PortTryInto3Any {
    type Error;
    fn port_try_into3_any(self) -> Result<unsafe_erased::Erased, Self::Error>;
}
