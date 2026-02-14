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
use std::io::{self, BufRead, Write};
use std::os::raw::{c_char, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

const NS_VIEW_WIDTH_SIZABLE: u64 = 2;
const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;

static REGISTER_ONCE: Once = Once::new();
static BRIDGE_READY: AtomicBool = AtomicBool::new(false);

// ---- libdispatch: schedule work on the main queue (WKWebView must be touched on main thread)
#[repr(C)]
struct dispatch_queue_s {
    _private: [u8; 0],
}
type dispatch_queue_t = *mut dispatch_queue_s;

unsafe extern "C" {
    fn dispatch_get_main_queue() -> dispatch_queue_t;
    fn dispatch_async_f(queue: dispatch_queue_t, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

fn class_ref(name: &str) -> &'static Class {
    Class::get(name).unwrap_or_else(|| panic!("Objective-C class not found: {name}"))
}

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

fn env_flag(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => matches!(
            v.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

// Minimal JS string quoting (no serde dependency).
fn js_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// -------------------- Agent Task scheduling --------------------

enum MainTaskKind {
    EvalJs(String),
    LoadUrl(String),
    Quit,
}

struct MainTask {
    webview: id,
    kind: MainTaskKind,
}

extern "C" fn run_on_main(ctx: *mut c_void) {
    let task = unsafe { Box::from_raw(ctx as *mut MainTask) };
    unsafe {
        match task.kind {
            MainTaskKind::EvalJs(js) => {
                let js_ns = nsstring(&js);
                let _: () = msg_send![task.webview, evaluateJavaScript: js_ns completionHandler: nil];
            }
            MainTaskKind::LoadUrl(url) => {
                let url_ns = nsstring(&url);
                let nsurl: id = msg_send![class_ptr("NSURL"), URLWithString: url_ns];
                let req: id = msg_send![class_ptr("NSURLRequest"), requestWithURL: nsurl];
                let _: id = msg_send![task.webview, loadRequest: req];
            }
            MainTaskKind::Quit => {
                let app = NSApp();
                let _: () = msg_send![app, terminate: nil];
            }
        }
    }
}

fn dispatch_main(webview: id, kind: MainTaskKind) {
    let boxed = Box::new(MainTask { webview, kind });
    let ptr = Box::into_raw(boxed) as *mut c_void;
    unsafe {
        dispatch_async_f(dispatch_get_main_queue(), ptr, run_on_main);
    }
}

// -------------------- ObjC callback: JS -> Rust --------------------

fn handle_bridge_message(s: &str) {
    // Protocol: "type\npayload"
    let (ty, payload) = s.split_once('\n').unwrap_or(("log", s));

    match ty {
        "bridge_ready" => {
            BRIDGE_READY.store(true, Ordering::Relaxed);
            eprintln!("[bridge] ready: {}", payload);
        }
        "bridge_info" => eprintln!("[bridge] {}", payload),
        "error" => eprintln!("[bridge:error] {}", payload),

        "user_send" => {
            // Make the tmux pane a readable transcript
            println!("\nYOU: {}\n", payload);
        }

        "assistant_start" => {
            print!("ASSISTANT: ");
            let _ = io::stdout().flush();
        }
        "assistant_delta" => {
            print!("{}", payload);
            let _ = io::stdout().flush();
        }
        "assistant_done" => {
            println!("\n");
        }

        other => {
            // Fallback: don’t drop information
            println!("[{}] {}", other, payload);
        }
    }
}

extern "C" fn did_receive_script_message(_this: &Object, _cmd: Sel, _controller: id, message: id) {
    unsafe {
        let body: id = msg_send![message, body];
        let body_desc: id = msg_send![body, description];
        let body_s = nsstring_to_string(body_desc);

        // Print/handle on Rust side
        handle_bridge_message(&body_s);
    }
}

// -------------------- ObjC callback: navigation lifecycle --------------------

extern "C" fn webview_did_finish_navigation(_this: &Object, _cmd: Sel, _webview: id, _nav: id) {
    eprintln!("[nav] didFinishNavigation");
}

// -------------------- Register ObjC classes --------------------

unsafe fn register_objc_classes() {
    REGISTER_ONCE.call_once(|| {
        let mut handler = ClassDecl::new("RustWKScriptMessageHandler", class_ref("NSObject"))
            .expect("Failed to declare RustWKScriptMessageHandler");
        unsafe {
            handler.add_method(
                sel!(userContentController:didReceiveScriptMessage:),
                did_receive_script_message as extern "C" fn(&Object, Sel, id, id),
            );
        }
        handler.register();

        let mut nav = ClassDecl::new("RustWKNavigationDelegate", class_ref("NSObject"))
            .expect("Failed to declare RustWKNavigationDelegate");
        unsafe {
            nav.add_method(
                sel!(webView:didFinishNavigation:),
                webview_did_finish_navigation as extern "C" fn(&Object, Sel, id, id),
            );
        }
        nav.register();
    });
}

fn main() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        register_objc_classes();

        // ----- App + window
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
        window.setTitle_(NSString::alloc(nil).init_str("WKWebView Agent (Rust)"));
        window.makeKeyAndOrderFront_(nil);
        app.activateIgnoringOtherApps_(YES);

        // ----- WKWebView config
        let config: id = msg_send![class_ptr("WKWebViewConfiguration"), new];

        // Persistent vs ephemeral data store (login persistence control)
        let ephemeral = env_flag("WEBVIEW_EPHEMERAL");
        let data_store: id = if ephemeral {
            msg_send![class_ptr("WKWebsiteDataStore"), nonPersistentDataStore]
        } else {
            msg_send![class_ptr("WKWebsiteDataStore"), defaultDataStore]
        };
        let _: () = msg_send![config, setWebsiteDataStore: data_store];

        // User content controller: install message handler + inject shim
        let ucc: id = msg_send![class_ptr("WKUserContentController"), new];

        let handler: id = msg_send![class_ptr("RustWKScriptMessageHandler"), new];
        let handler_name = nsstring("rust");
        let _: () = msg_send![ucc, addScriptMessageHandler: handler name: handler_name];

        // JS shim: installs window.__rustBridge.send(text) + assistant streaming observer
        //
        // It posts messages back as: "type\npayload"
        // Types:
        // - bridge_ready
        // - user_send
        // - assistant_start / assistant_delta / assistant_done
        // - error
        let shim = include_str!("shim.js");

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

        // ----- WKWebView
        let content_view: id = window.contentView();
        let bounds: NSRect = msg_send![content_view, bounds];

        let webview: id = msg_send![class_ptr("WKWebView"), alloc];
        let webview: id = msg_send![webview, initWithFrame: bounds configuration: config];

        let mask = NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE;
        let _: () = msg_send![webview, setAutoresizingMask: mask];
        let _: () = msg_send![content_view, addSubview: webview];

        let nav_delegate: id = msg_send![class_ptr("RustWKNavigationDelegate"), new];
        let _: () = msg_send![webview, setNavigationDelegate: nav_delegate];

        // ----- Load URL
        let start_url = std::env::var("START_URL").unwrap_or_else(|_| "https://chat.openai.com/".into());
        eprintln!("[boot] START_URL={} (WEBVIEW_EPHEMERAL={})", start_url, ephemeral);

        let url_ns = NSString::alloc(nil).init_str(&start_url);
        let url: id = msg_send![class_ptr("NSURL"), URLWithString: url_ns];
        let req: id = msg_send![class_ptr("NSURLRequest"), requestWithURL: url];
        let _: id = msg_send![webview, loadRequest: req];

        // ----- stdin thread: tmux -> agent
        //
        // Commands:
        //   :open <url>
        //   :eval <js>
        //   :ping
        //   :quit
        // Otherwise: sends the whole line as a message.
        std::thread::spawn({
            let webview = webview;
            move || {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    let Ok(line) = line else { break };
                    let line = line.trim_end().to_string();
                    if line.is_empty() {
                        continue;
                    }

                    if line.starts_with(":quit") {
                        dispatch_main(webview, MainTaskKind::Quit);
                        break;
                    }

                    if let Some(rest) = line.strip_prefix(":open ") {
                        dispatch_main(webview, MainTaskKind::LoadUrl(rest.trim().to_string()));
                        continue;
                    }

                    if let Some(rest) = line.strip_prefix(":eval ") {
                        dispatch_main(webview, MainTaskKind::EvalJs(rest.to_string()));
                        continue;
                    }

                    if line == ":ping" {
                        let js = r#"(function(){ try{ window.webkit.messageHandlers.rust.postMessage("bridge_info\nping"); }catch(e){} })();"#.to_string();
                        dispatch_main(webview, MainTaskKind::EvalJs(js));
                        continue;
                    }

                    // Send message (default)
                    // If bridge isn’t ready yet, we still try; shim will appear quickly after load.
                    let quoted = js_quote(&line);
                    let js = format!(
                        r#"(function() {{
  try {{
    if (window.__rustBridge && window.__rustBridge.send) {{
      window.__rustBridge.send({quoted});
    }} else {{
      if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.rust) {{
        window.webkit.messageHandlers.rust.postMessage("error\nbridge not ready yet");
      }}
    }}
  }} catch (e) {{}}
}})();"#,
                        quoted = quoted
                    );
                    dispatch_main(webview, MainTaskKind::EvalJs(js));
                }
            }
        });

        // ----- Run loop
        app.run();
    }
}
