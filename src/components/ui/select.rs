use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

/// A native `<select>`, styled like [`Input`](crate::components::ui::input::Input).
///
/// Deliberately not a custom dropdown: a real `<select>` posts with the form even
/// without JavaScript, and the browser handles keyboard and mobile behaviour.
#[component]
pub fn Select(
    // Styling
    #[prop(into, optional)] class: String,

    // Common HTML attributes
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] required: bool,

    // Ref for direct DOM access
    #[prop(optional)] node_ref: NodeRef<html::Select>,

    /// The `<option>` elements.
    children: Children,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "text-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
        "focus-visible:border-ring focus-visible:ring-ring/50",
        "focus-visible:ring-2",
        "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
        class
    );

    view! {
        <select
            data-name="Select"
            class=merged_class
            name=name
            id=id
            disabled=disabled
            required=required
            node_ref=node_ref
        >
            {children()}
        </select>
    }
}
