use leptos::prelude::*;

use crate::api::sessions::next_sessions;
use crate::components::blocks::SessionCard;
use crate::models::SessionView;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    let sessions = Resource::new(|| (), |()| async move { next_sessions().await });

    view! {
        <div class="home-hero border border-(--border) rounded-[2rem] shadow-(--shadow) max-w-4xl mx-auto">
            <h2 class="home-hero-title">"Bulle Créaline"</h2>
            <h3>"Ma source de créativité"</h3>
            <h3>"Ateliers créatifs bien-être"</h3>
        </div>

        <Transition fallback=|| {
            view! {
                <p class="mx-auto mt-12 max-w-6xl text-sm text-muted-foreground">
                    "Chargement des prochaines séances…"
                </p>
            }
        }>
            {move || Suspend::new(async move {
                // A visitor came for the workshops, not for a database error: when
                // the sessions cannot be read the page simply stops after the hero
                // rather than explaining itself. The failure is already logged
                // server-side.
                sessions
                    .await
                    .ok()
                    .filter(|sessions| !sessions.is_empty())
                    .map(|sessions| view! { <UpcomingSessions sessions=sessions/> })
            })}
        </Transition>
    }
}

/// One grid of cards, every kind of workshop mixed together, soonest date first.
///
/// `sessions` arrives already ordered and already capped by the server, so this
/// neither sorts nor truncates: the order on screen is the order it was given.
#[component]
fn UpcomingSessions(sessions: Vec<SessionView>) -> impl IntoView {
    let cards = sessions
        .into_iter()
        .map(|session| view! { <SessionCard session=session/> })
        .collect::<Vec<_>>();

    view! {
        <section class="mx-auto mt-12 max-w-6xl">
            <div>
                <h2 class="text-2xl font-bold lg:text-3xl text-heading">"Prochaines séances"</h2>
                <span class="block mt-3 w-10 h-1 rounded-full bg-heading-soft"></span>
            </div>

            // Three across from `lg` rather than `xl`: on a laptop that keeps each
            // card narrow instead of stretching two of them across the full width.
            <div class="grid gap-5 mt-6 sm:grid-cols-2 lg:grid-cols-3">{cards}</div>
        </section>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::models::ServiceType;

    /// One session per given kind, each carrying a date label of its own so the
    /// cards can be told apart in the rendered markup.
    fn sessions(kinds: &[ServiceType]) -> Vec<SessionView> {
        kinds
            .iter()
            .enumerate()
            .map(|(rank, kind)| SessionView {
                id: format!("651d1f0a00000000000000{rank:02}"),
                service_type: *kind,
                date_label: format!("séance {rank}"),
                date_input: "2026-07-05T14:00".to_owned(),
                theme_id: "651d1f0a0000000000000099".to_owned(),
                theme_name: "Aquarelle".to_owned(),
                photo_url: "/media/theme/651d1f0a0000000000000099?v=1".to_owned(),
                price: 65.0,
                max_persons: 8,
                booked_persons: 0,
            })
            .collect()
    }

    fn sections_html(sessions: Vec<SessionView>) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            provide_context(RequestUrl::new("/"));

            view! {
                <Router>
                    <UpcomingSessions sessions=sessions/>
                </Router>
            }
            .to_html()
        })
    }

    /// One flat grid: a single heading over the lot, and one card per session
    /// whatever kind of workshop it belongs to.
    #[test]
    fn draws_one_grid_of_every_kind() {
        let html = sections_html(sessions(&[
            ServiceType::AperosCreatifs,
            ServiceType::ApresMidisCreatifs,
            ServiceType::AperosCreatifs,
        ]));

        assert_eq!(html.matches("<section").count(), 1, "not one section: {html}");
        assert_eq!(html.matches("<article").count(), 3, "not three cards: {html}");
        assert_eq!(
            html.matches("Prochaines séances").count(),
            1,
            "the heading is not shown exactly once: {html}"
        );
    }

    /// Nothing sorts or regroups here, so the cards must come out in the order the
    /// server handed them over -- which is by date.
    #[test]
    fn keeps_the_order_it_was_given() {
        let html = sections_html(sessions(&[
            ServiceType::AperosCreatifs,
            ServiceType::ApresMidisCreatifs,
            ServiceType::ParentsEnfantsMoinsSix,
        ]));

        let positions: Vec<_> = (0..3)
            .map(|rank| {
                html.find(&format!("séance {rank}"))
                    .unwrap_or_else(|| panic!("séance {rank} is missing: {html}"))
            })
            .collect();

        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "the cards were reordered: {positions:?}"
        );
    }

    /// With no section per kind, the card's own badge is the only thing naming the
    /// workshop, so every kind on show has to be legible from the markup.
    #[test]
    fn every_card_still_names_its_workshop() {
        let kinds = [ServiceType::AperosCreatifs, ServiceType::ParentsEnfantsSixADouze];
        let html = sections_html(sessions(&kinds));

        for kind in kinds {
            assert!(html.contains(kind.label()), "{kind:?} is not named: {html}");
        }
    }
}
