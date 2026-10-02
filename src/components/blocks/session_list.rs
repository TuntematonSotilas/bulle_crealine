//! A run of dates, each with its price, what is left of it, and the way in.
//!
//! Shared by the two places that list dates: a theme unfolded on a workshop's page
//! ([`crate::components::blocks::upcoming_themes`]) and a card on the home page
//! ([`crate::components::blocks::session_card`]), which since it merges the sessions
//! of one workshop-and-theme has several dates to show. Written once so the two
//! cannot drift: a visitor meets the same row in both.

use icons::Users;
use leptos::either::Either;
use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize};
use crate::models::SessionView;

/// The dates of one offer.
#[component]
pub fn SessionList(sessions: Vec<SessionView>) -> impl IntoView {
    let rows = sessions
        .into_iter()
        .map(|session| view! { <li><SessionRow session=session/></li> })
        .collect::<Vec<_>>();

    view! { <ul class="flex flex-col gap-3">{rows}</ul> }
}

/// One date, with what it costs, what is left of it, and how to take it.
///
/// The price sits on the row rather than on whatever holds it: it belongs to the
/// session, so two dates of one workshop on one theme may legitimately differ, and
/// a figure stated once above them could only be wrong about one of the two.
#[component]
pub fn SessionRow(session: SessionView) -> impl IntoView {
    let full = session.is_full();
    let availability = session.availability_label();
    let price = session.price_label();

    let action = if full {
        Either::Left(view! {
            <span class="inline-flex justify-center items-center px-3 h-8 text-sm font-medium rounded-md border cursor-not-allowed bg-muted text-muted-foreground">
                "Complet"
            </span>
        })
    } else {
        Either::Right(view! {
            // The date travels with the link: it was just chosen here, and the
            // booking page has no reason to ask for it again.
            <Button
                size=ButtonSize::Sm
                href=format!("/booking/{}?session={}", session.service_slug, session.id)
            >
                "Réserver"
            </Button>
        })
    };

    view! {
        <div class="flex flex-wrap gap-3 justify-between items-center p-4 rounded-2xl border bg-surface text-surface-foreground border-border">
            <div class="flex flex-col gap-1">
                <span class="font-medium">{session.date_label}</span>
                <span class="flex gap-2 items-center text-sm">
                    <span class="text-muted-foreground">{price}</span>
                    <Users class="w-3.5 h-3.5 shrink-0 text-heading-soft"/>
                    <span class=if full {
                        "font-medium text-destructive"
                    } else {
                        "text-muted-foreground"
                    }>{availability}</span>
                </span>
            </div>
            {action}
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// One date, named and priced so that two of them cannot be confused.
    fn date(id: &str, label: &str, price: f64, booked_persons: u32) -> SessionView {
        SessionView {
            id: id.to_owned(),
            service_slug: "aperos-creatifs".to_owned(),
            service_label: "Apéros créatifs".to_owned(),
            service_description: "Une soirée entre adultes.".to_owned(),
            service_path: "/services/aperos-creatifs".to_owned(),
            date_label: label.to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000099".to_owned(),
            theme_name: "Aquarelle".to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000099".to_owned(),
            price,
            max_persons: 8,
            booked_persons,
            min_persons: 1,
        }
    }

    fn list_html(sessions: Vec<SessionView>) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The booking button is a link, and a link resolves `aria-current`
            // against the location being rendered.
            provide_context(RequestUrl::new("/"));

            view! { <Router><SessionList sessions=sessions/></Router> }.to_html()
        })
    }

    #[test]
    fn a_date_shows_what_it_costs_and_what_is_left() {
        let html = list_html(vec![date("s1", "dimanche 5 juillet", 65.0, 5)]);

        assert!(html.contains("dimanche 5 juillet"), "no date: {html}");
        assert!(html.contains("65 €"), "no price: {html}");
        assert!(html.contains("3 places restantes"), "no availability: {html}");
    }

    /// The date was just chosen here, so the booking page has no reason to ask for
    /// it again.
    #[test]
    fn booking_a_date_carries_it_to_the_booking_page() {
        let html = list_html(vec![date("s1", "dimanche 5 juillet", 65.0, 0)]);

        assert!(
            html.contains("/booking/aperos-creatifs?session=s1"),
            "the date does not travel with the link: {html}"
        );
    }

    #[test]
    fn a_full_date_says_so_and_offers_no_booking() {
        let html = list_html(vec![date("s1", "dimanche 5 juillet", 65.0, 8)]);

        assert!(html.contains("Complet"), "a full date should say so: {html}");
        assert!(!html.contains("/booking/"), "nothing left to book: {html}");
    }

    /// Nothing here sorts or truncates: whoever builds the list decides what it
    /// holds and in what order.
    #[test]
    fn every_date_gets_a_row() {
        let html = list_html(vec![
            date("s1", "dimanche 5 juillet", 65.0, 0),
            date("s2", "dimanche 12 juillet", 70.0, 0),
        ]);

        assert_eq!(html.matches("<li").count(), 2, "not one row per date: {html}");
        assert!(html.contains("dimanche 12 juillet"), "the second is missing: {html}");
    }

    /// The price belongs to the session, so one offer's dates may differ -- and
    /// each row has to carry its own rather than inherit a figure stated above.
    #[test]
    fn each_date_carries_its_own_price() {
        let html = list_html(vec![
            date("s1", "dimanche 5 juillet", 65.0, 0),
            date("s2", "dimanche 12 juillet", 70.0, 0),
        ]);

        assert!(html.contains("65 €"), "the first price is missing: {html}");
        assert!(html.contains("70 €"), "the second price is missing: {html}");
    }
}
