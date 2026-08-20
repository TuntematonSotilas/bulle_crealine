use leptos::prelude::*;

/// Renders the "apéros créatifs (adultes)" page.
#[component]
pub fn AperosCreatifs() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl space-y-4 py-6">
            <h1 class="text-4xl font-semibold tracking-tight text-heading">
                "Apéros créatifs (adultes)"
            </h1>
            <span class="block h-1 w-10 rounded-full bg-heading-soft"></span>
        </div>
    }
}
