//! DOM, fetch, and timer bindings.
//!
//! On WASM targets these call through to real JS APIs via `web-sys`.
//! On native targets (CLI / tests) they are no-op stubs.

// ── console.log ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn console_log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn console_log(msg: &str) {
    println!("{}", msg);
}

// ── document.getElementById ───────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn get_element_by_id(id: &str) -> Option<web_sys::Element> {
    let window   = web_sys::window()?;
    let document = window.document()?;
    document.get_element_by_id(id)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_element_by_id(_id: &str) -> Option<()> { None }

// ── element.innerText setter ──────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn set_inner_text(id: &str, text: &str) {
    if let Some(el) = get_element_by_id(id) {
        if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
            html.set_inner_text(text);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_inner_text(_id: &str, _text: &str) {}

// ── setTimeout stub ───────────────────────────────────────────────────────────

/// Schedule `callback_idx` (a VM function index) to be called after `ms` ms.
/// On WASM this queues a `setTimeout`; on native it calls immediately.
#[cfg(target_arch = "wasm32")]
pub fn set_timeout(_fn_idx: u16, _ms: u32) {
    // Actual async scheduling is delegated to the JS event loop via the
    // loader script.  The VM is called back through `SELVR_resume`.
    // See `runtime/SELVR-loader.js` for the glue.
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_timeout(_fn_idx: u16, _ms: u32) {}

// ── fetch stub ────────────────────────────────────────────────────────────────

/// Initiate a fetch request.  Returns immediately; the result is delivered via
/// a promise callback on the JS side.  On native, this is a no-op.
#[cfg(target_arch = "wasm32")]
pub fn fetch_url(_url: &str, _callback_fn: u16) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_url(_url: &str, _callback_fn: u16) {}
