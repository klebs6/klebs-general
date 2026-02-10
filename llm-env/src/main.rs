#![cfg(target_os = "macos")]

#[macro_use]
extern crate objc;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreType, NSWindow,
    NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Once;

const NS_VIEW_WIDTH_SIZABLE: u64 = 2;
const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;

static REGISTER_ONCE: Once = Once::new();

fn class_ref(name: &str) -> &'static Class {
    Class::get(name).unwrap_or_else(|| panic!("Objective-C class not found: {name}"))
}

// Convenient for msg_send! receivers (matches what many objc examples do).
fn class_ptr(name: &str) -> *const Class {
    class_ref(name) as *const Class
}

fn nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s) }
}

fn nsstring_to_string(s: id) -> String {
    if s == nil {
        return String::new();
    }
    unsafe {
        let c_str: *const c_char = msg_send![s, UTF8String];
        if c_str.is_null() {
            return String::new();
        }
        CStr::from_ptr(c_str).to_string_lossy().into_owned()
    }
}

// --------- Objective-C callback: WKScriptMessageHandler ----------
extern "C" fn did_receive_script_message(_this: &Object, _cmd: Sel, _controller: id, message: id) {
    unsafe {
        let name: id = msg_send![message, name];
        let body: id = msg_send![message, body];

        let name_s = nsstring_to_string(name);

        // body can be NSString/NSDictionary/etc; description is safe-ish.
        let body_desc: id = msg_send![body, description];
        let body_s = nsstring_to_string(body_desc);

        eprintln!("[bridge:{}] {}", name_s, body_s);
    }
}

// --------- Objective-C callback: WKNavigationDelegate ----------
extern "C" fn webview_did_finish_navigation(_this: &Object, _cmd: Sel, webview: id, _nav: id) {
    unsafe {
        eprintln!("[nav] didFinishNavigation");

        // Rust -> JS -> Rust roundtrip (posts back into did_receive_script_message)
        let js = r#"
            (function() {
              try {
                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.rust) {
                  window.webkit.messageHandlers.rust.postMessage(
                    "rust->js roundtrip: " + document.title + " @ " + location.href
                  );
                }
              } catch (e) {}
            })();
        "#;

        let js_ns = nsstring(js);
        let _: () = msg_send![webview, evaluateJavaScript: js_ns completionHandler: nil];
    }
}

unsafe fn register_objc_classes() {
    REGISTER_ONCE.call_once(|| {
        // Script message handler class
        let mut handler = ClassDecl::new("RustWKScriptMessageHandler", class_ref("NSObject"))
            .expect("Failed to declare RustWKScriptMessageHandler");
        handler.add_method(
            sel!(userContentController:didReceiveScriptMessage:),
            did_receive_script_message as extern "C" fn(&Object, Sel, id, id),
        );
        handler.register();

        // Navigation delegate class
        let mut nav = ClassDecl::new("RustWKNavigationDelegate", class_ref("NSObject"))
            .expect("Failed to declare RustWKNavigationDelegate");
        nav.add_method(
            sel!(webView:didFinishNavigation:),
            webview_did_finish_navigation as extern "C" fn(&Object, Sel, id, id),
        );
        nav.register();
    });
}

fn env_flag(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => matches!(
            v.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn main() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        register_objc_classes();

        // --- App + window ----------------------------------------------------
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1200.0, 800.0));
        let style = NSWindowStyleMask::NSTitledWindowMask
            | NSWindowStyleMask::NSClosableWindowMask
            | NSWindowStyleMask::NSResizableWindowMask
            | NSWindowStyleMask::NSMiniaturizableWindowMask;

        let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
            frame,
            style,
            NSBackingStoreType::NSBackingStoreBuffered,
            NO,
        );

        window.setTitle_(NSString::alloc(nil).init_str("WKWebView Bridge Test (Rust)"));
        window.makeKeyAndOrderFront_(nil);
        app.activateIgnoringOtherApps_(YES);

        // --- WKWebView configuration ----------------------------------------
        let config: id = msg_send![class_ptr("WKWebViewConfiguration"), new];

        // Persistent vs ephemeral store (useful for login persistence)
        let ephemeral = env_flag("WEBVIEW_EPHEMERAL");
        let data_store: id = if ephemeral {
            msg_send![class_ptr("WKWebsiteDataStore"), nonPersistentDataStore]
        } else {
            msg_send![class_ptr("WKWebsiteDataStore"), defaultDataStore]
        };
        let _: () = msg_send![config, setWebsiteDataStore: data_store];

        // User content controller: scripts + message handler
        let ucc: id = msg_send![class_ptr("WKUserContentController"), new];

        // Install our handler under name "rust"
        let handler: id = msg_send![class_ptr("RustWKScriptMessageHandler"), new];
        let handler_name = nsstring("rust");
        let _: () = msg_send![ucc, addScriptMessageHandler: handler name: handler_name];

        // Inject JS shim at document end (runs on every navigation)
        let shim = r#"
            (function() {
              try {
                if (!window.__rustBridgeInstalled) {
                  window.__rustBridgeInstalled = true;

                  var post = function(msg) {
                    try { window.webkit.messageHandlers.rust.postMessage(msg); } catch (e) {}
                  };

                  post("shim installed @ " + location.href);

                  window.addEventListener('load', function() {
                    post("window load @ " + location.href + " title=" + document.title);
                  }, { once: true });
                }
              } catch (e) {}
            })();
        "#;

        let shim_ns = nsstring(shim);
        let injection_time_at_document_end: i64 = 1; // WKUserScriptInjectionTimeAtDocumentEnd
        let for_main_frame_only = YES;

        let user_script: id = msg_send![class_ptr("WKUserScript"), alloc];
        let user_script: id = msg_send![user_script,
            initWithSource: shim_ns
            injectionTime: injection_time_at_document_end
            forMainFrameOnly: for_main_frame_only
        ];

        let _: () = msg_send![ucc, addUserScript: user_script];
        let _: () = msg_send![config, setUserContentController: ucc];

        // --- WKWebView -------------------------------------------------------
        let content_view: id = window.contentView();
        let bounds: NSRect = msg_send![content_view, bounds];

        let webview: id = msg_send![class_ptr("WKWebView"), alloc];
        let webview: id = msg_send![webview, initWithFrame: bounds configuration: config];

        let mask = NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE;
        let _: () = msg_send![webview, setAutoresizingMask: mask];
        let _: () = msg_send![content_view, addSubview: webview];

        // Navigation delegate (for Rust->JS eval + lifecycle hooks)
        let nav_delegate: id = msg_send![class_ptr("RustWKNavigationDelegate"), new];
        let _: () = msg_send![webview, setNavigationDelegate: nav_delegate];

        // --- Load URL --------------------------------------------------------
        let start_url = std::env::var("START_URL").unwrap_or_else(|_| "https://example.com".into());
        eprintln!(
            "[boot] START_URL={} (WEBVIEW_EPHEMERAL={})",
            start_url, ephemeral
        );

        let url_ns = NSString::alloc(nil).init_str(&start_url);
        let url: id = msg_send![class_ptr("NSURL"), URLWithString: url_ns];
        let req: id = msg_send![class_ptr("NSURLRequest"), requestWithURL: url];
        let _: id = msg_send![webview, loadRequest: req];

        // --- Run loop --------------------------------------------------------
        app.run();
    }
}
