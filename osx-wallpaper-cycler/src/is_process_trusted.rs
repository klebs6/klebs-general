crate::ix!();

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[cfg(target_os = "macos")]
pub fn is_process_trusted_for_accessibility() -> bool {
    unsafe { AXIsProcessTrusted() }
}
