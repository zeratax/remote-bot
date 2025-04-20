use leptos::{logging::debug_warn, mount::hydrate_body};
use remote_bot_app::App;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    debug_warn!("hydrate mode - hydrating");

    hydrate_body(App);
}
