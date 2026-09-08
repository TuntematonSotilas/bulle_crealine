use icons::{ArrowRight, CalendarDays, ImageOff, Palette, Users};
use leptos::either::Either;
use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize};
use crate::models::SessionView;

/// One upcoming session, as the home page advertises it.
///
/// Everything shown is already on the [`SessionView`] or derivable from its kind
/// of workshop, so the card needs no lookup of its own.
#[component]
pub fn SessionCard(session: SessionView) -> impl IntoView {
    let service = session.service_type;
    // Read before the view is destructured, and not reactive: the list is built
    // once from a loaded resource, so a full session stays full until it reloads.
    let full = session.is_full();
    let availability = session.availability_label();

    let SessionView {
        date_label,
        theme_name,
        photo_url,
        ..
    } = session;

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
            <Button size=ButtonSize::Sm href=format!("/booking/{}", service.slug())>
                "Réserver"
            </Button>
        })
    };

    view! {
        <article class="flex overflow-hidden flex-col rounded-[2rem] border transition-shadow group border-border bg-card text-card-foreground shadow-(--shadow) hover:shadow-lg">

            // 16/9 rather than the 4/3 a photo usually gets: the picture is here
            // to set a mood, and a quarter less height lets more dates sit above
            // the fold.
            <div class="overflow-hidden relative aspect-16/9">
                {photo}
                // The kind of workshop as a badge over the photo. Not decoration:
                // the home page mixes every kind into one date-ordered grid, so
                // this is the only thing telling a visitor which workshop a date
                // belongs to.
                <span class="absolute top-2 left-2 px-2.5 py-0.5 text-xs font-medium rounded-full shadow-sm backdrop-blur bg-card/90 text-heading">
                    {service.label()}
                </span>
            </div>

            <div class="flex flex-col flex-1 gap-2.5 p-4">
                <p class="text-sm leading-relaxed text-muted-foreground">
                    {service.description()}
                </p>

                <div class="grid gap-1.5 text-sm">
                    <div class="flex gap-2 items-start">
                        <CalendarDays class="mt-0.5 w-3.5 h-3.5 shrink-0 text-heading-soft"/>
                        <span class="font-medium">{date_label}</span>
                    </div>
                    <div class="flex gap-2 items-start">
                        <Palette class="mt-0.5 w-3.5 h-3.5 shrink-0 text-heading-soft"/>
                        <span>"Thème : "{theme_name}</span>
                    </div>
                    <div class="flex gap-2 items-start">
                        <Users class="mt-0.5 w-3.5 h-3.5 shrink-0 text-heading-soft"/>
                        <span class=if full {
                            "font-medium text-destructive"
                        } else {
                            "text-muted-foreground"
                        }>{availability}</span>
                    </div>
                </div>

                <div class="flex gap-3 justify-between items-center pt-1 mt-auto">
                    {action}
                    <a
                        href=service.page_path()
                        class="inline-flex gap-1 items-center text-sm font-medium underline-offset-4 text-primary hover:underline"
                    >
                        "voir +"
                        <ArrowRight class="w-4 h-4"/>
                    </a>
                </div>
            </div>
        </article>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::models::ServiceType;

    fn session(booked_persons: u32) -> SessionView {
        SessionView {
            id: "651d1f0a0000000000000001".to_owned(),
            service_type: ServiceType::AperosCreatifs,
            date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000002".to_owned(),
            theme_name: "Aquarelle".to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000002?v=7".to_owned(),
            price: 65.0,
            max_persons: 8,
            booked_persons,
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

    /// The card is the whole advertisement for a date: photo, what the workshop
    /// is, when it is, what it is about, what is left, and the two ways on.
    #[test]
    fn a_card_shows_the_whole_offer() {
        let html = card_html(session(5));

        assert!(html.contains("/media/theme/651d1f0a0000000000000002?v=7"), "no photo: {html}");
        assert!(html.contains(ServiceType::AperosCreatifs.label()), "no workshop kind: {html}");
        assert!(
            html.contains(ServiceType::AperosCreatifs.description()),
            "no description: {html}"
        );
        assert!(html.contains("dimanche 5 juillet 2026 à 14h00"), "no date: {html}");
        assert!(html.contains("Aquarelle"), "no theme: {html}");
        assert!(html.contains("3 places restantes"), "no remaining places: {html}");
        assert!(html.contains("/booking/aperos-creatifs"), "no booking link: {html}");
        assert!(html.contains("voir +"), "no \"voir +\" link: {html}");
        assert!(
            html.contains(ServiceType::AperosCreatifs.page_path()),
            "\"voir +\" points nowhere: {html}"
        );
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
