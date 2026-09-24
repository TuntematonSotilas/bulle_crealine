use icons::MapPin;
use leptos::prelude::*;

use crate::models::{STUDIO_ADDRESS, STUDIO_MAP_URL};

/// Where a bookable workshop is held, with the address linked to a map.
///
/// One component for the three places it shows -- the workshop's own page, the
/// booking form and the confirmation screen -- so the address cannot end up
/// saying three different things.
///
/// Shown only where a visitor is expected to travel: a workshop run for a
/// structure takes place at the structure's address, not at this one.
#[component]
pub fn StudioPlace() -> impl IntoView {
    view! {
        <div class="p-4 rounded-2xl border bg-surface text-surface-foreground border-border">
            <div class="text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                "Lieu"
            </div>
            <a
                href=STUDIO_MAP_URL
                target="_blank"
                rel="noreferrer noopener"
                class="inline-flex gap-2 items-center mt-1 font-medium transition-colors hover:text-heading"
            >
                <MapPin class="w-4 h-4 flex-shrink-0"/>
                <span class="underline">{STUDIO_ADDRESS}</span>
                // The link leaves the site for a new tab, which nothing else here
                // would announce.
                <span class="sr-only">" (nouvel onglet)"</span>
            </a>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn html() -> String {
        Owner::new().with(|| view! { <StudioPlace/> }.to_html())
    }

    #[test]
    fn the_address_is_shown_and_linked_to_a_map() {
        let html = html();

        assert!(html.contains(STUDIO_ADDRESS), "no address: {html}");
        assert!(html.contains(STUDIO_MAP_URL), "no map link: {html}");
    }

    /// A tab opening behind the page is a surprise worth naming, and `noopener`
    /// keeps the opened page from reaching back into this one.
    #[test]
    fn the_map_link_says_it_opens_a_tab_and_cannot_reach_back() {
        let html = html();

        assert!(html.contains(r#"target="_blank""#), "{html}");
        assert!(html.contains("noopener"), "{html}");
        assert!(html.contains("nouvel onglet"), "{html}");
    }
}
