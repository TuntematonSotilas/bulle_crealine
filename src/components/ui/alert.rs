use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::tw_merge;

/// How loudly an [`Alert`] speaks.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Default,
    /// For an operation that cannot be undone.
    Destructive,
    Success,
}

impl AlertVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-card text-card-foreground",
            Self::Destructive => {
                "border-destructive/50 bg-destructive/10 text-destructive dark:border-destructive"
            }
            Self::Success => "border-primary/50 bg-primary/10 text-foreground",
        }
    }
}

#[component]
pub fn Alert(
    #[prop(into, optional)] class: String,
    #[prop(optional)] variant: AlertVariant,
    children: Children,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "relative w-full rounded-lg border px-4 py-3 text-sm",
        variant.class(),
        class
    );

    view! {
        <div data-name="Alert" role="alert" class=merged_class>
            {children()}
        </div>
    }
}

mod components {
    use super::*;
    clx! {AlertTitle, div, "mb-1 font-medium leading-none tracking-tight"}
    clx! {AlertDescription, div, "text-sm [&_p]:leading-relaxed"}
}

pub use components::*;
