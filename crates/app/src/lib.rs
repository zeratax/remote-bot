pub mod api;
pub mod components;
pub mod pages;
pub mod util;

use components::dark_mode_toggle::DarkModeToggle;
use leptos::{
    IntoView, component,
    prelude::{ClassAttribute, ElementChild},
    view,
};
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use pages::{debt_game::DebtGame, history::History, home_page::HomePage, not_found::NotFound};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Remote Bot" />
        <Stylesheet id="leptos" href="/pkg/remote-bot.css" />

        <div class="min-h-screen bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-900 dark:to-gray-800 text-gray-900 dark:text-gray-100">
            <header class="bg-white/70 dark:bg-gray-900/70 backdrop-blur sticky top-0 z-50 shadow-sm">
                <div class="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
                    <h1 class="text-xl font-semibold tracking-wide">"🖼 Remote Bot"</h1>
                        <nav class="flex items-center gap-4 text-sm">
                        <a href="/" class="hover:underline">"Home"</a>
                        <a href="/history" class="hover:underline">"History"</a>
                        <a href="/debt-game" class="hover:underline">"Debt Game"</a>
                        <DarkModeToggle />
                    </nav>
                </div>
            </header>

            <main class="max-w-4xl mx-auto">
                <Router>
                    <Routes fallback=|| view! { <NotFound /> }>
                        <Route path=path!("/") view=|| view! { <HomePage /> } />
                        <Route path=path!("/history") view=|| view! { <History /> } />
                        <Route path=path!("/debt-game") view=|| view! { <DebtGame /> } />
                    </Routes>
                </Router>
            </main>
        </div>
    }
}
