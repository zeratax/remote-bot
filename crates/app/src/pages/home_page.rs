use leptos::{
    IntoView, component,
    prelude::{ClassAttribute, ElementChild},
    view,
};

use crate::components::current_wallpaper::CurrentWallpaper;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <section class="flex flex-col items-center gap-6">
            <h2 class="text-2xl font-bold">"Current Wallpaper"</h2>
            <CurrentWallpaper />
        </section>
    }
}
