use leptos::prelude::*;

use crate::components::blocks::studio_place::StudioPlace;
use crate::components::seo::PageMeta;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};

/// The place itself, at `/moi-et-mon-atelier/mon-atelier`.
///
/// The menu has pointed here from every page since before the page existed, and so
/// have two buttons on "Qui suis-je" -- all of them answering 404. This is a page
/// about a room, so what it owes a visitor is where it is and what it is like.
///
/// The address comes from [`StudioPlace`], the same block the workshop pages and the
/// booking form show, so the one place a visitor is asked to travel to cannot end up
/// spelled four different ways.
///
/// The wording below is drafted from what the site already says of itself, on "Qui
/// suis-je" -- it is a stand-in for the owner's own words, not a replacement for
/// them. Nothing here claims anything the rest of the site does not already claim:
/// an invented detail about the room would be worse than a short page.
#[component]
pub fn MonAtelierPage() -> impl IntoView {
    view! {
        <PageMeta
            title="Mon atelier à Feurs — Bulle Créaline (E.I)"
            description="L'atelier Bulle Créaline, 5 rue Marc Seguin à Feurs (42110) : le lieu où se tiennent les ateliers créatifs, et comment s'y rendre."
            path="/moi-et-mon-atelier/mon-atelier"
        />

        <div class="py-6 mx-auto space-y-10 max-w-3xl">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Mon atelier"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
                <p class="mx-auto max-w-2xl text-lg leading-8 text-muted-foreground">
                    "Le lieu où se tiennent les séances, à Feurs, dans la Loire."
                </p>
            </div>

            <div class="space-y-4 rounded-[2rem] border border-border bg-surface text-surface-foreground p-6 shadow-sm">
                <p class="text-base leading-8">
                    "L'atelier est un lieu chaleureux, vivant et rassurant, pensé pour qu'on s'y
                     installe sans se presser. On y vient créer, souffler, et repartir avec un
                     peu plus de fierté et de joie."
                </p>
                <p class="text-base leading-8">
                    "C'est là que se tiennent les ateliers parents-enfants, les après-midis et
                     les apéros créatifs entre adultes, ainsi que les accompagnements
                     individuels. Les ateliers menés en institution, eux, se déroulent chez
                     vous."
                </p>
            </div>

            <section class="space-y-4">
                <h2 class="text-2xl font-bold lg:text-3xl text-heading">"Venir à l'atelier"</h2>
                <span class="block w-10 h-1 rounded-full bg-heading-soft"></span>

                // The same block the workshop pages and the booking form show, so
                // the address cannot end up saying two different things.
                <StudioPlace/>
            </section>

            <div class="flex flex-wrap gap-6 justify-center">
                <Button size=ButtonSize::Pill href="/moi-et-mon-atelier/photos">
                    "Album de mes ateliers"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Pill
                    href="/contact"
                >
                    "Me contacter"
                </Button>
            </div>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    use crate::models::OWNER;

    fn page_html() -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The map link and the buttons resolve `aria-current` against the
            // location being rendered.
            provide_context(RequestUrl::new("/moi-et-mon-atelier/mon-atelier"));

            view! { <Router><MonAtelierPage/></Router> }.to_html()
        })
    }

    /// The page is about a room, so the one thing it cannot leave out is where that
    /// room is. Asserted against `OWNER` rather than a copy of the string: a test
    /// holding its own spelling would go on passing after the address changed.
    #[test]
    fn the_page_says_where_the_atelier_is() {
        let html = page_html();

        assert!(html.contains(OWNER.address), "no address: {html}");
        assert!(html.contains(OWNER.map_url), "no map link: {html}");
    }

    /// One `h1`, and it names the subject rather than the business.
    #[test]
    fn the_page_opens_on_a_single_h1() {
        let html = page_html();

        assert_eq!(html.matches("<h1").count(), 1, "not exactly one h1: {html}");
        assert!(html.contains("Mon atelier"), "{html}");
    }

    /// The menu linked here from every page while the route did not exist. A page
    /// that leads nowhere would only move the dead end one click further in.
    #[test]
    fn the_page_leads_on_rather_than_ending() {
        let html = page_html();

        assert!(html.contains(r#"href="/contact""#), "no way to get in touch: {html}");
        assert!(
            html.contains(r#"href="/moi-et-mon-atelier/photos""#),
            "no way through to the album: {html}"
        );
    }
}
