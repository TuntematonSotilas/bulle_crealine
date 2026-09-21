use std::str::FromStr;

use icons::common::IconType;
use icons::leptos::icon_component::LeptosIcon;
use leptos::prelude::*;

/// Draws the picto a workshop was given, by name.
///
/// Workshops are rows the admin edits, so the icon can only be stored as a name.
/// Every other icon in this project is a component chosen at compile time; this is
/// the one place a name has to be resolved at runtime, which the icon crate allows
/// through `IconType`'s `FromStr` and its single [`LeptosIcon`] renderer.
///
/// Renders nothing at all for an empty or unknown name: a workshop is free to have
/// no picto, and an empty `<svg>` would leave a gap in the layout instead.
#[component]
pub fn ServiceIcon(
    /// A Lucide name such as `"Palette"`, exactly as the picker offers it.
    #[prop(into)]
    name: String,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    IconType::from_str(&name)
        .ok()
        .map(|icon| view! { <LeptosIcon icon=icon class=class/> })
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn draws_the_icon_it_is_named() {
        let html = Owner::new()
            .with(|| view! { <ServiceIcon name="Palette" class="w-6 h-6"/> }.to_html());

        assert!(html.contains("<svg"), "no icon was drawn: {html}");
        assert!(html.contains("w-6 h-6"), "the classes were dropped: {html}");
    }

    /// The name comes out of the database, so it can be empty, and it can be
    /// whatever an earlier version of the picker offered. Neither may leave an
    /// empty box in the middle of a heading.
    #[test]
    fn draws_nothing_for_a_name_it_cannot_resolve() {
        for name in ["", "NotAnIcon", "palette"] {
            let html = Owner::new()
                .with(|| view! { <ServiceIcon name=name class="w-6 h-6"/> }.to_html());

            assert!(!html.contains("<svg"), "{name:?} should draw nothing: {html}");
        }
    }
}
