use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Label(
    // Styling
    #[prop(into, optional)] class: String,

    /// Id of the field being described, rendered into the `for` attribute.
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
