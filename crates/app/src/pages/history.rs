use leptos::{
    IntoView, component,
    prelude::{ClassAttribute, ElementChild},
    view,
};

use crate::components::previous_wallpapers::PreviousWallpapers;

#[component]
pub fn History() -> impl IntoView {
    view! {
        <section class="flex flex-col items-center gap-6">
            <h2 class="text-2xl font-bold">"History"</h2>
            <PreviousWallpapers />
        </section>
    }
}
