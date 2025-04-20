use codee::string::JsonSerdeCodec;
use leptos::{
    IntoView, component,
    logging::log,
    prelude::{ClassAttribute, Effect, ElementChild, Get, OnAttribute, Signal, document},
    view,
};
use leptos_use::{UseCookieOptions, use_cookie_with_options, use_preferred_dark};
use reactive_graph::traits::Set;

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    let (cookie, set_cookie) =
        use_cookie_with_options::<bool, JsonSerdeCodec>("dark_mode", UseCookieOptions::default());

    let system_pref = use_preferred_dark();
    let resolved = Signal::derive(move || cookie.get().unwrap_or_else(|| system_pref.get()));

    Effect::new(move |_| {
        if let Some(html) = document().document_element() {
            if resolved.get() {
                html.class_list().add_1("dark").ok();
                log!("setting theme to dark");
            } else {
                html.class_list().remove_1("dark").ok();
                log!("setting theme to light");
            }
        }
    });

    let on_click = move |_| {
        set_cookie.set(Some(!resolved.get()));
    };

    Effect::new(move |_| {
        set_cookie.set(Some(system_pref.get()));
    });

    view! {
        <button
            class="text-sm px-3 py-1 rounded border border-gray-400 \
                   dark:border-gray-600 hover:bg-gray-200 \
                   dark:hover:bg-gray-700 transition"
            on:click=on_click
        >
            {move || if resolved.get() { "🌙 Dark" } else { "☀️ Light" }}
        </button>
    }
}
