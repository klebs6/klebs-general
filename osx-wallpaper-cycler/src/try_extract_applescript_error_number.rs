// ---------------- [ File: osx-wallpaper-cycler/src/try_extract_applescript_error_number.rs ]
crate::ix!();

#[cfg(target_os = "macos")]
pub fn try_extract_applescript_error_number(
    dict: &objc2_foundation::NSDictionary<objc2_foundation::NSString, objc2::runtime::AnyObject>,
) -> Option<i32> {
    use objc2::runtime::AnyObject;
    use objc2_foundation::NSString;

    let key = NSString::from_str("NSAppleScriptErrorNumber");

    let retained = dict.objectForKey(&key);
    let obj: Option<&AnyObject> = retained.as_deref();
    let obj = obj?;

    let n: i32 = unsafe { objc2::msg_send![obj, intValue] };
    Some(n)
}

#[cfg(target_os = "macos")]
pub fn try_extract_applescript_error_string(
    dict: &objc2_foundation::NSDictionary<objc2_foundation::NSString, objc2::runtime::AnyObject>,
    key_name: &str,
) -> Option<String> {
    use objc2::runtime::AnyObject;
    use objc2_foundation::NSString;
    use std::ffi::{c_char, CStr};

    let key = NSString::from_str(key_name);

    let retained = dict.objectForKey(&key);
    let obj: Option<&AnyObject> = retained.as_deref();
    let obj = obj?;

    let desc_ptr: *mut NSString = unsafe { objc2::msg_send![obj, description] };
    if desc_ptr.is_null() {
        return None;
    }

    let desc_ref: &NSString = unsafe { &*desc_ptr };
    let c_str: *const c_char = unsafe { objc2::msg_send![desc_ref, UTF8String] };
    if c_str.is_null() {
        return None;
    }

    Some(unsafe { CStr::from_ptr(c_str) }.to_string_lossy().to_string())
}
