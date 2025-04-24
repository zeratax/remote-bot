use crate::api::wallpaper::{WallpaperRecord, get_wallpapers};
use leptos::{
    IntoView, component,
    control_flow::Show,
    html::Div,
    prelude::{
        ClassAttribute, Effect, ElementChild, For, Get, IntoAny, NodeRef, NodeRefAttribute,
        ReadSignal, Set, Transition, Update, With, WriteSignal, signal, window,
    },
    server::Resource,
    view,
};
use leptos_use::use_intersection_observer;

#[component]
pub fn PreviousWallpapers() -> impl IntoView {
    let (page, set_page): (ReadSignal<u32>, WriteSignal<u32>) = signal(1);
    let (has_more, set_has_more) = signal(true);
    let (had_error, set_had_error) = signal(false);

    let per_page = 20;
    let wallpapers = Resource::new(move || page.get(), move |p| get_wallpapers(p, per_page));

    let sentinel = NodeRef::<Div>::new();

    let (scrolled, set_scrolled) = signal(false);
    Effect::new(move |_| {
        let win = window();
        let y = win.scroll_y().unwrap_or(0.0);
        if y > 50.0 {
            set_scrolled.set(true);
        }
    });

    Effect::new(move |_| {
        use_intersection_observer(sentinel.clone(), move |entries, _| {
            if !entries.is_empty() && has_more.get() && !had_error.get() && scrolled.get() {
                set_page.update(|p| *p += 1);
            }
        });
    });

    Effect::new(move |_| {
        if let Some(Err(_)) = wallpapers.get() {
            set_has_more.set(false);
            set_had_error.set(true);
        }
    });

    view! {
        <div class="p-4 space-y-6">
            <h2 class="text-2xl font-bold">"All Wallpapers"</h2>
            <Transition fallback=move || view! {
                <p class="text-center italic">"Loading…"</p>
            }>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    <Show
                        when=move || {
                            match wallpapers.get() {
                                Some(Ok(_)) => true,
                                _ => false
                            }
                        }
                        fallback=|| view! { <div></div> }
                    >
                        {move || {
                            if let Some(Ok(data)) = wallpapers.get() {
                                if data.len() < per_page as usize {
                                    set_has_more.set(false);
                                }

                                view! {
                                    <For
                                        each=move || data.clone()
                                        key=|w| w.id
                                        children=move |w: WallpaperRecord| {
                                            view! {
                                                <div class="border rounded-lg overflow-hidden shadow">
                                                    <img
                                                        src={format!("/data/wallpapers/{}", w.path)}
                                                        alt={w.name.clone()}
                                                        class="w-full h-auto object-contain"
                                                    />
                                                    <div class="p-2">
                                                        <p class="text-sm text-gray-600">
                                                            {format!("by {} on {}", w.set_by, w.created_at)}
                                                        </p>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}
                    </Show>
                </div>

                {move || {
                    if had_error.get() {
                        view! {
                            <div class="p-4 mt-4 text-red-600 bg-red-50 border border-red-200 rounded-md">
                                <p class="text-center">"Error loading wallpapers. Please try again later."</p>
                            </div>
                        }.into_any()
                    } else if !has_more.get() && wallpapers.with(|w| match w {
                        Some(Ok(data)) => data.is_empty(),
                        _ => true
                    }) {
                        view! {
                            <div class="p-4 mt-4 text-gray-600">
                                <p class="text-center">"No wallpapers found."</p>
                            </div>
                        }.into_any()
                    } else if !has_more.get() {
                        view! {
                            <div class="p-4 mt-4 text-gray-600">
                                <p class="text-center">"No more wallpapers to load."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div node_ref=sentinel class="h-8"/> }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}
