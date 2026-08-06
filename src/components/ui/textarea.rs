use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Textarea(
    // Styling
    #[prop(into, optional)] class: String,

    // Common HTML attributes
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(optional)] rows: Option<u16>,
    #[prop(optional)] maxlength: Option<u32>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] readonly: bool,
    #[prop(optional)] required: bool,

    /// Initial content. The browser owns the value afterwards, which is what a
    /// plain form submission needs.
    #[prop(into, optional)]
    value: Option<String>,

    // Two-way binding (like bind:value)
    #[prop(into, optional)] bind_value: Option<RwSignal<String>>,

    // Ref for direct DOM access
    #[prop(optional)] node_ref: NodeRef<html::Textarea>,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "text-foreground placeholder:text-muted-foreground dark:bg-input/30 border-input flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
        "focus-visible:border-ring focus-visible:ring-ring/50",
        "focus-visible:ring-2",
        "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
        "read-only:bg-muted",
        class
    );

    match bind_value {
        Some(signal) => view! {
            <textarea
                data-name="Textarea"
                class=merged_class
                placeholder=placeholder
                name=name
                id=id
                rows=rows
                maxlength=maxlength
                disabled=disabled
                readonly=readonly
                required=required
                bind:value=signal
                node_ref=node_ref
            />
        }
        .into_any(),
        None => view! {
            <textarea
                data-name="Textarea"
                class=merged_class
                placeholder=placeholder
                name=name
                id=id
                rows=rows
                maxlength=maxlength
                disabled=disabled
                readonly=readonly
                required=required
                node_ref=node_ref
            >
                {value}
            </textarea>
        }
        .into_any(),
    }
}
