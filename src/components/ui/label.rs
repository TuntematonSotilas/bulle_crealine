use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Label(
    // Styling
    #[prop(into, optional)] class: String,

    /// Identifiant du champ décrit, rendu dans l'attribut `for`.
    #[prop(into, optional)]
    r#for: Option<String>,

    children: Children,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "flex items-center gap-2 text-sm font-medium leading-none select-none",
        "group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50",
        "peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        class
    );

    view! {
        <label data-slot="label" class=merged_class for=r#for>
            {children()}
        </label>
    }
}
