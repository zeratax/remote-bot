use js_sys::{Function, Object, Reflect};
use leptos::logging::log;
use wasm_bindgen::{JsCast, JsValue};

pub fn show_notification(title: &str, body: &str) {
    log!("Attempting to show notification");
    if let Some(window) = web_sys::window() {
        if let Ok(notification) = Reflect::get(&window, &JsValue::from_str("Notification")) {
            if notification.is_undefined() {
                log!("Notifications not supported or not available.");
                return;
            }
            let permission = Reflect::get(&notification, &JsValue::from_str("permission"))
                .unwrap_or(JsValue::from_str("denied"));
            if permission.as_string().unwrap_or_default() != "granted" {
                log!("Notification permission not granted.");
                return;
            }
            let options = Object::new();
            let _ = Reflect::set(
                &options,
                &JsValue::from_str("body"),
                &JsValue::from_str(body),
            );
            let _ = Reflect::set(
                &options,
                &JsValue::from_str("icon"),
                &JsValue::from_str("/favicon.ico"),
            );
            if let Some(constructor) = notification.dyn_ref::<Function>() {
                let js_func =
                    Function::new_with_args("title,options", "return new this(title, options)");
                let _ = js_func
                    .call2(&constructor, &JsValue::from_str(title), &options)
                    .expect("Failed to create Notification");
            } else {
                log!("Notification constructor not a function.");
            }
        } else {
            log!("'Notification' not found on window object.");
        }
    } else {
        log!("Window object not available.");
    }
}
