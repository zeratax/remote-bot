use leptos::{
    IntoView, component,
    prelude::{ClassAttribute, ElementChild, Get, IntoAny, Show, Suspense},
    server::Resource,
    view,
};

use crate::api::wallpaper::get_current_wallpaper;

#[component]
pub fn CurrentWallpaper() -> impl IntoView {
    let filename = Resource::new(|| (), |_| get_current_wallpaper());
    let current_wallpaper = move || {
        filename
            .get()
            .and_then(|res| res.as_ref().ok().and_then(|s| s.clone()))
    };

    view! {
        <div class="flex flex-col items-center gap-4 p-8">
            <Suspense fallback=|| view! {
                <p class="text-lg text-gray-500 italic">"Loading wallpaper..."</p>
            }>
                <Show
                    when=move || filename.get().is_some()
                    fallback=|| view! {
                        <p class="text-lg text-gray-500 italic">"No wallpaper has been set."</p>
                    }
                >
                    {move || match current_wallpaper() {
                        Some(wallpaper) => view! {
                            <img
                                src=format!("data/wallpapers/{}", wallpaper.path)
                                alt="Current Wallpaper"
                                class="max-w-full max-h-[80vh] rounded-2xl shadow-xl object-cover"
                            />
                        }.into_any(),
                        None => view! {
                            <p class="text-lg text-gray-500 italic">"No wallpaper has been set."</p>
                        }.into_any(),
                    }}
                </Show>
            </Suspense>
        </div>
    }
}
