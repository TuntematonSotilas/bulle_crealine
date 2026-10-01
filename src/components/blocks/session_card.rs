use icons::{ArrowRight, ImageOff, Users};
use leptos::either::Either;
use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize};
use crate::models::SessionView;

/// One upcoming session, as the home page advertises it.
///
/// Everything shown is already on the [`SessionView`], the workshop's own wording
/// included, so the card needs no lookup of its own. That matters more than it
/// used to: workshops are rows now, so a card that resolved its own would turn one
/// grid into one request per tile.
#[component]
pub fn SessionCard(session: SessionView) -> impl IntoView {
    // Read before the view is destructured, and not reactive: the list is built
    // once from a loaded resource, so a full session stays full until it reloads.
    let full = session.is_full();
    let availability = session.availability_label();

    let SessionView {
        id,
        service_slug,
        service_label,
        service_description,
        service_path,
        date_label,
        theme_name,
        photo_url,
        ..
    } = session;

    // The workshop page opens on this very date rather than on its theme grid.
    // Not carried when the workshop is gone: `service_path` is then "/", where
    // the parameter would name nothing.
    let more_path = if service_path == "/" {
        service_path
    } else {
        format!("{service_path}?session={id}")
    };

    let photo = if photo_url.is_empty() {
        // Only reachable when the themes collection was edited by hand, since
        // deleting a theme a session uses is refused. Still worth drawing: an
        // empty `src` has the browser request the page itself as the image.
        Either::Left(view! {
            <div class="flex justify-center items-center w-full h-full bg-surface text-muted-foreground">
                <ImageOff class="w-7 h-7"/>
            </div>
        })
    } else {
        Either::Right(view! {
            <img
                src=photo_url
                alt=format!("Thème : {theme_name}")
                loading="lazy"
                class="object-cover w-full h-full transition-transform duration-300 group-hover:scale-105"
            />
        })
    };

    // A full session keeps its card but loses its link: there is nothing to book,
    // and a disabled anchor is not a thing browsers offer.
    let action = if full {
        // Matches ButtonSize::Sm below, so the two states leave the card the same
        // height.
        Either::Left(view! {
            <span class="inline-flex justify-center items-center px-3 h-8 text-sm font-medium rounded-md border cursor-not-allowed bg-muted text-muted-foreground">
                "Complet"
            </span>
        })
    } else {
        Either::Right(view! {
            // The date travels with the link, so the booking page opens on it
            // rather than asking the visitor to pick it a second time.
            <Button size=ButtonSize::Sm href=format!("/booking/{service_slug}?session={id}")>
                "Réserver"
            </Button>
        })
    };

    view! {
        <article class="flex overflow-hidden flex-col rounded-[2rem] border transition-shadow group border-border bg-card text-card-foreground shadow-(--shadow) hover:shadow-lg">

            // Date first, as the card's title: the home page mixes every kind of
            // workshop into one date-ordered grid, so when a session runs is what
            // a visitor scans for. It carries no icon for the same reason -- a
            // heading beside a glyph reads as a list item rather than a title.
            <header class="flex flex-col gap-1.5 items-start px-4 pt-4 pb-3">
                <p class="text-base font-semibold leading-snug text-heading">{date_label}</p>
                // Full-strength text rather than the muted grey: this line is part
                // of the offer, not a caption. It stays neutral so the date above
                // keeps the accent colour to itself.
                <p class="text-base font-medium text-card-foreground">{theme_name}</p>
                // The chip is the only thing naming which workshop this date
                // belongs to, the grid being one flat list. Its text sits on the
                // pale surface rather than in the accent colour, which at this
                // size would not clear the contrast floor.
                <span class="px-2.5 py-0.5 text-xs font-medium rounded-full bg-surface text-surface-foreground">
                    {service_label}
                </span>
            </header>

            // 16/9 rather than the 4/3 a photo usually gets: the picture is here
            // to set a mood, and a quarter less height lets more dates sit above
            // the fold.
            <div class="overflow-hidden relative aspect-16/9">
                {photo}
            </div>

            <footer class="flex flex-col flex-1 gap-2.5 p-4">
                <p class="text-sm leading-relaxed text-muted-foreground">
                    {service_description}
                </p>

                <div class="flex gap-2 items-start text-sm">
                    <Users class="mt-0.5 w-3.5 h-3.5 shrink-0 text-heading-soft"/>
                    <span class=if full {
                        "font-medium text-destructive"
                    } else {
                        "text-muted-foreground"
                    }>{availability}</span>
                </div>

                <div class="flex gap-3 justify-between items-center pt-1 mt-auto">
                    {action}
                    <a
                        href=more_path
                        class="inline-flex gap-1 items-center text-sm font-medium underline-offset-4 text-primary hover:underline"
                    >
                        "voir +"
                        <ArrowRight class="w-4 h-4"/>
                    </a>
                </div>
            </footer>
        </article>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// A session already resolved the way the server hands it over: the workshop
    /// is a row now, so its wording arrives on the view rather than being derived.
    const LABEL: &str = "Apéros créatifs (adultes)";
    const DESCRIPTION: &str = "Une soirée entre adultes autour d'un verre.";

    fn session(booked_persons: u32) -> SessionView {
        SessionView {
            id: "651d1f0a0000000000000001".to_owned(),
            service_slug: "aperos-creatifs".to_owned(),
            service_label: LABEL.to_owned(),
            service_description: DESCRIPTION.to_owned(),
            service_path: "/services/aperos-creatifs".to_owned(),
            date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000002".to_owned(),
            theme_name: "Aquarelle".to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000002?v=7".to_owned(),
            price: 65.0,
            max_persons: 8,
            booked_persons,
            min_persons: 1,
        }
    }

    /// Rendered inside a `<Router>`: the booking button is a link, and a link
    /// resolves its `aria-current` against the current location, which server-side
    /// means the URL of the request being rendered.
    fn card_html(session: SessionView) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            provide_context(RequestUrl::new("/"));

            view! {
                <Router>
                    <SessionCard session=session/>
                </Router>
            }
            .to_html()
        })
    }

    /// "voir +" leads to the workshop's page, and says which date was clicked so
    /// that page can open on it rather than on its grid of themes.
    #[test]
    fn the_more_link_carries_the_session_it_was_clicked_from() {
        let html = card_html(session(5));

        assert!(
            html.contains(r#"href="/services/aperos-creatifs?session=651d1f0a0000000000000001""#),
            "{html}"
        );
    }

    /// A deleted workshop leaves its sessions pointing at the home page, where the
    /// parameter would name nothing.
    #[test]
    fn a_session_whose_workshop_is_gone_carries_no_parameter() {
        let orphan = SessionView { service_path: "/".to_owned(), ..session(5) };

        let html = card_html(orphan);

        // Bounded to the "voir +" anchor: the booking button beside it carries
        // the parameter legitimately, and a check over the whole card would read
        // that one instead.
        let more = html
            .split("<a ")
            .find(|fragment| fragment.contains("voir +"))
            .expect(r#"the "voir +" link should render"#);

        assert!(more.contains(r#"href="/""#), "no link home: {more}");
        assert!(!more.contains("?session="), "a stray parameter: {more}");
    }

    /// The card is the whole advertisement for a date: photo, what the workshop
    /// is, when it is, what it is about, what is left, and the two ways on.
    #[test]
    fn a_card_shows_the_whole_offer() {
        let html = card_html(session(5));

        assert!(html.contains("/media/theme/651d1f0a0000000000000002?v=7"), "no photo: {html}");
        assert!(html.contains(LABEL), "no workshop kind: {html}");
        assert!(
            html.contains(DESCRIPTION),
            "no description: {html}"
        );
        assert!(html.contains("dimanche 5 juillet 2026 à 14h00"), "no date: {html}");
        assert!(html.contains("Aquarelle"), "no theme: {html}");
        assert!(html.contains("3 places restantes"), "no remaining places: {html}");
        assert!(
            html.contains("/booking/aperos-creatifs?session=651d1f0a0000000000000001"),
            "the booking link should carry the date: {html}"
        );
        assert!(html.contains("voir +"), "no \"voir +\" link: {html}");
        assert!(
            html.contains("/services/aperos-creatifs"),
            "\"voir +\" points nowhere: {html}"
        );
    }

    /// The card reads top to bottom: when it is, what it is about and which
    /// workshop it belongs to, then the photo, then what it involves, what is left
    /// and the way in.
    #[test]
    fn the_card_is_laid_out_header_photo_then_footer() {
        let html = card_html(session(5));

        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle:?} is missing: {html}"))
        };

        // The photo's alt text names the theme too, so the first hit is the
        // header's line, which is the one that has to come before the image.
        let order = [
            at("dimanche 5 juillet 2026 à 14h00"),
            at("Aquarelle"),
            at(LABEL),
            at("/media/theme/"),
            at(DESCRIPTION),
            at("3 places restantes"),
            at("/booking/aperos-creatifs"),
        ];

        assert!(
            order.windows(2).all(|pair| pair[0] < pair[1]),
            "the card is out of order: {order:?}"
        );
    }

    /// A `<header>` and a `<footer>` around the photo, rather than one flat run of
    /// divs, so the grouping the layout implies is in the markup too.
    #[test]
    fn the_card_groups_its_three_parts() {
        let html = card_html(session(5));

        assert_eq!(html.matches("<header").count(), 1, "no single header: {html}");
        assert_eq!(html.matches("<footer").count(), 1, "no single footer: {html}");
    }

    /// Offering to book a session that cannot take anyone would send the visitor
    /// to a booking page that refuses them.
    #[test]
    fn a_full_session_says_so_and_offers_no_booking() {
        let html = card_html(session(8));

        assert!(html.contains("Complet"), "does not say it is full: {html}");
        assert!(
            !html.contains("/booking/aperos-creatifs"),
            "still links to the booking page: {html}"
        );
        // The workshop's own page is still worth reaching.
        assert!(html.contains("voir +"), "lost the \"voir +\" link: {html}");
    }

    /// An `<img src="">` has the browser fetch the page itself as the image, which
    /// on this route means re-rendering the home page to draw a thumbnail.
    #[test]
    fn a_session_without_a_photo_renders_no_empty_image() {
        let mut without_photo = session(0);
        without_photo.photo_url = String::new();

        let html = card_html(without_photo);

        assert!(!html.contains("<img"), "drew an image with no source: {html}");
        assert!(html.contains("dimanche 5 juillet 2026 à 14h00"), "lost the date: {html}");
    }
}
