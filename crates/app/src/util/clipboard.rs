use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::window;

pub fn copy_to_clipboard(text: &str) -> bool {
    if let Some(window) = window() {
        let navigator = window.navigator();
        if let Ok(clipboard) = Reflect::get(&navigator, &JsValue::from_str("clipboard")) {
            if !clipboard.is_undefined() {
                if let Ok(write_text) = Reflect::get(&clipboard, &JsValue::from_str("writeText")) {
                    if let Some(write_fn) = write_text.dyn_ref::<Function>() {
                        let _ = write_fn.call1(&clipboard, &JsValue::from_str(text));
                        return true;
                    }
                }
            }
        }
    }
    false
}
