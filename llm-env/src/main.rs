#![cfg(target_os = "macos")]

#[macro_use]
extern crate objc;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreType, NSWindow,
    NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::runtime::Class;

const NS_VIEW_WIDTH_SIZABLE: u64 = 2;
const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;

unsafe fn class(name: &str) -> *const Class {
    Class::get(name).unwrap_or_else(|| panic!("Objective-C class not found: {name}"))
}

fn main() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

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

        window.setTitle_(NSString::alloc(nil).init_str("WKWebView Smoke Test (Rust)"));
        window.makeKeyAndOrderFront_(nil);
        app.activateIgnoringOtherApps_(YES);

        // --- WKWebView -------------------------------------------------------
        // Create configuration: WKWebViewConfiguration *config = [WKWebViewConfiguration new];
        let config: id = msg_send![class("WKWebViewConfiguration"), new];

        // Compute bounds of the content view so the webview fills the window.
        let content_view: id = window.contentView();
        let bounds: NSRect = msg_send![content_view, bounds];

        // WKWebView *webview = [[WKWebView alloc] initWithFrame:bounds configuration:config];
        let webview: id = msg_send![class("WKWebView"), alloc];
        let webview: id = msg_send![webview, initWithFrame: bounds configuration: config];

        // Make it resize with the window.
        let mask = NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE;
        let _: () = msg_send![webview, setAutoresizingMask: mask];

        // Add it to the window.
        let _: () = msg_send![content_view, addSubview: webview];

        // --- Load URL --------------------------------------------------------
        let start_url = std::env::var("START_URL").unwrap_or_else(|_| "https://example.com".into());
        let url_ns = NSString::alloc(nil).init_str(&start_url);

        // NSURL *url = [NSURL URLWithString:url_ns];
        let url: id = msg_send![class("NSURL"), URLWithString: url_ns];

        // NSURLRequest *req = [NSURLRequest requestWithURL:url];
        let req: id = msg_send![class("NSURLRequest"), requestWithURL: url];

        // [webview loadRequest:req];
        let _: id = msg_send![webview, loadRequest: req];

        // --- Run loop --------------------------------------------------------
        app.run();
    }
}
