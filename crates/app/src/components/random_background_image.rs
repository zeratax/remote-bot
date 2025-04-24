use crate::api::debt_game::get_random_background;
use leptos::prelude::{
    ChildrenFn, ClassAttribute, Effect, ElementChild, Get, IntoAny, StyleAttribute, Transition,
    signal,
};
use leptos::server::OnceResource;
use leptos::{IntoView, component, view};

#[component]
pub fn RandomBackgroundImage(children: ChildrenFn) -> impl IntoView {
    let bg_url = OnceResource::new(get_random_background());
    let children_fallback = children.clone();
    let children_loaded = children;

    let (visible, set_visible) = signal(false);

    Effect::new(move |_| {
        set_visible(false);
        if bg_url.get().is_some() {
            set_visible(true);
        }
    });

    view! {
        <Transition fallback=move || view! {
            <div class="min-h-screen w-full flex flex-col items-center overflow-auto">
                { children_fallback() }
            </div>
        }>
            { move || {
                if let Some(Ok(url)) = bg_url.get() {
                    view! {
                        <div
                            class="min-h-screen w-full bg-cover bg-center bg-no-repeat flex flex-col items-center overflow-auto transition-opacity duration-1000"
                            class=("opacity-0", !visible.get())
                            class=("opacity-100", visible.get())
                            style=format!("background-image: url('{}');", url)
                        >
                            { children_loaded() }
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="min-h-screen w-full bg-gray-100 flex flex-col items-center overflow-auto">
                            { children_loaded() }
                        </div>
                    }.into_any()
                }
            }}
        </Transition>
    }
}
