use icons::{Mail, Phone};
use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::blocks::studio_place::StudioPlace;
use crate::models::OWNER;

/// Renders the "Contact" page.
///
/// A card rather than a form. Everything here is already public — a professional's
/// contact details are what the legal notice is obliged to carry — so a form would
/// open a second collection to document and keep, for a message an email already
/// carries.
///
/// Every line reads [`OWNER`], the same value the legal notice reads: a phone number
/// written twice is a phone number that will one day differ between two pages.
#[component]
pub fn ContactPage() -> impl IntoView {
    view! {
        <Title text="Contact — Bulle Créaline (E.I)"/>

        <div class="py-6 mx-auto space-y-10 max-w-3xl">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Me contacter"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
                <p class="mx-auto max-w-2xl text-lg leading-8 text-muted-foreground">
                    "Pour un atelier dans votre structure, une question sur une séance, ou
                     simplement pour échanger."
                </p>
            </div>

            <div class="flex flex-col gap-4 p-6 rounded-[2rem] border shadow-sm border-border bg-card">
                <p class="text-lg font-semibold text-heading">{OWNER.owner}</p>

                // The same card the workshop pages and the booking form show, so the
                // address cannot end up saying two different things.
                <StudioPlace/>

                <Reach
                    label="Téléphone"
                    value=OWNER.phone
                    href=OWNER.phone_link()
                    icon=view! { <Phone class="w-4 h-4 flex-shrink-0"/> }.into_any()
                />
                <Reach
                    label="Email"
                    value=OWNER.email
                    href=OWNER.email_link()
                    icon=view! { <Mail class="w-4 h-4 flex-shrink-0"/> }.into_any()
                />
            </div>
        </div>
    }
}

/// One way of getting in touch, laid out like the address above it.
#[component]
fn Reach(
    label: &'static str,
    value: &'static str,
    href: String,
    icon: AnyView,
) -> impl IntoView {
    view! {
        <div class="p-4 rounded-2xl border bg-surface text-surface-foreground border-border">
            <div class="text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                {label}
            </div>
            <a
                href=href
                class="inline-flex gap-2 items-center mt-1 font-medium transition-colors hover:text-heading"
            >
                {icon}
                <span class="underline">{value}</span>
            </a>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn page_html() -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The map link resolves `aria-current` against the location being
            // rendered.
            provide_context(RequestUrl::new("/contact"));

            view! { <Router><ContactPage/></Router> }.to_html()
        })
    }

    /// Asserted against `OWNER` rather than against copies of the strings: a test
    /// holding its own spelling of the phone number would go on passing after the
    /// constant changed, which is the one failure that matters here.
    #[test]
    fn the_page_carries_every_way_of_reaching_her() {
        let html = page_html();

        assert!(html.contains(OWNER.owner), "no name: {html}");
        assert!(html.contains(OWNER.address), "no address: {html}");
        assert!(html.contains(OWNER.phone), "no phone number: {html}");
        assert!(html.contains(OWNER.email), "no email: {html}");
    }

    /// Reading a number is not the same as dialling one: on a phone the link is the
    /// whole point, and it needs the international form the page never shows.
    #[test]
    fn the_phone_and_the_email_are_links_that_act() {
        let html = page_html();

        assert!(html.contains(&OWNER.phone_link()), "the number does not dial: {html}");
        assert!(html.contains(&OWNER.email_link()), "the address does not open: {html}");
    }

    /// The address comes from the shared block, so it cannot drift from the one the
    /// booking form and the workshop pages show.
    #[test]
    fn the_address_is_the_one_every_other_page_shows() {
        let html = page_html();

        assert!(html.contains(OWNER.map_url), "no map link: {html}");
        assert!(html.contains("Lieu"), "not the shared address block: {html}");
    }

    /// No form here, so nothing to collect and nothing more to document: the page
    /// hands over details that the legal notice is obliged to publish anyway.
    #[test]
    fn the_page_collects_nothing() {
        let html = page_html();

        assert!(!html.contains("<form"), "a form would be a collection: {html}");
        assert!(!html.contains("<input"), "nor a field: {html}");
    }
}
