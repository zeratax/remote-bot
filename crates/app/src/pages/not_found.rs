use leptos::{
    IntoView, component,
    prelude::{ClassAttribute, ElementChild},
    view,
};

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="text-center mt-20">
            <h1 class="text-4xl font-bold mb-4">"404"</h1>
            <p class="text-lg text-gray-500">"Page not found."</p>
        </div>
    }
}
