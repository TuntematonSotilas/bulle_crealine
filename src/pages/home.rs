use leptos::prelude::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {

    view! {
        <div class="home-hero border border-(--border) rounded-[2rem] shadow-(--shadow) max-w-4xl mx-auto">
            <h2 class="home-hero-title">"Bulle Créaline"</h2>
            <h3>"Ma source de créativité"</h3>
            <h3>"Ateliers créatifs bien-être"</h3>
        </div>
    }
}