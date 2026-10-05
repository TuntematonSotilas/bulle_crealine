use leptos::prelude::*;

use crate::api::sessions::next_sessions;
use crate::components::blocks::SessionCard;
use crate::components::seo::PageMeta;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::models::{HOME_CARDS, SessionGroup};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    // How many cards the visitor has asked for so far. It drives the resource, so
    // each click is a fresh round trip rather than a reveal out of something already
    // in hand -- the page is paid for by every visitor, most of whom never click.
    let shown = RwSignal::new(HOME_CARDS);

    // The ask travels back with the answer. The page cannot infer it from the number
    // of cards on screen: the server may legitimately answer a batch of six with
    // fewer cards and still say more follow, since one workshop running one theme
    // weekly fills a whole window and collapses into a single card. Comparing counts
    // would read that as a request in flight and leave the button disabled for good.
    let sessions = Resource::new(move || shown.get(), |count| async move {
        next_sessions(count).await.map(|page| (count, page))
    });

    view! {
        <PageMeta
            title="Ateliers créatifs à Feurs (42) — Bulle Créaline (E.I)"
            description="Ateliers créatifs bien-être à Feurs, dans la Loire : parents-enfants, apéros créatifs entre adultes, accompagnements individuels et ateliers en institution."
            path="/"
        />

        <div class="home-hero border border-(--border) rounded-[2rem] shadow-(--shadow) max-w-4xl mx-auto">
            // The page's one `h1`, and the only heading on the site that may be
            // just the name: it is the home page, and the name is what it is about.
            <h1 class="home-hero-title">
                "Bulle Créaline"
                <span class="home-hero-suffix">"(E.I)"</span>
            </h1>
            // Taglines, not sections. They were `h3` under an `h2`, which made the
            // page look like it had three levels of structure and no subject.
            <p>"Ma source de créativité"</p>
            <p>"Ateliers créatifs bien-être"</p>
        </div>

        // Nothing on this page used to say where the workshops are held. A visitor
        // could tell from the booking form, three clicks in; a search engine could
        // not tell at all.
        <p class="mx-auto mt-8 max-w-3xl text-lg leading-8 text-center text-muted-foreground">
            "Bulle Créaline est un atelier créatif installé à Feurs, dans la Loire. J'y anime
             des ateliers parents-enfants, des après-midis et apéros créatifs entre adultes et
             des accompagnements individuels, et je me déplace en institution ou hors les murs.
             Chaque séance est un temps pour soi : on crée, on souffle, et on repart avec
             quelque chose fait de ses mains."
        </p>

        // A `Transition` rather than a `Suspense` so that asking for more leaves the
        // cards already on screen where they are: the fallback below is for the
        // first load only, and a refetch must not blank the grid under the button
        // that was just clicked.
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
                    .filter(|(_, page)| !page.groups.is_empty())
                    .map(|(answered, page)| {
                        view! {
                            <UpcomingSessions
                                groups=page.groups
                                more=page.more
                                waiting=Signal::derive(move || shown.get() > answered)
                                shown=shown
                            />
                        }
                    })
            })}
        </Transition>
    }
}

/// One grid of cards, every kind of workshop mixed together, soonest date first.
///
/// `groups` arrives already ordered, already capped and already trimmed by the
/// server, so this neither sorts nor truncates: the order on screen is the order it
/// was given.
///
/// `waiting` and `shown` are props rather than state born here, on the model of
/// [`crate::components::blocks::upcoming_themes::UpcomingThemes`]: they let a test
/// render each state, which a signal owned inside would put out of reach. `shown`
/// is here for the click alone -- it counts cards, and the button adds a batch to it.
#[component]
fn UpcomingSessions(
    groups: Vec<SessionGroup>,
    more: bool,
    #[prop(into)] waiting: Signal<bool>,
    shown: RwSignal<usize>,
) -> impl IntoView {
    let cards = groups
        .into_iter()
        .map(|group| view! { <SessionCard group=group/> })
        .collect::<Vec<_>>();

    // Only while the server says something follows. A button that stays and does
    // nothing reads as broken, and the visitor has no way to tell which it is.
    let load_more = more.then(|| {
        view! {
            <div class="flex justify-center mt-8">
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Pill
                    attr:r#type="button"
                    attr:disabled=waiting
                    on:click=move |_| shown.update(|count| *count += HOME_CARDS)
                >
                    // The wording changes rather than a spinner appearing beside
                    // it: the wait is one round trip, and a button that renames
                    // itself says the click landed without the layout shifting.
                    {move || if waiting.get() { "Chargement…" } else { "Voir plus de séances" }}
                </Button>
            </div>
        }
    });

    view! {
        <section class="mx-auto mt-12 max-w-6xl">
            <div>
                <h2 class="text-2xl font-bold lg:text-3xl text-heading">"Prochaines séances"</h2>
                <span class="block mt-3 w-10 h-1 rounded-full bg-heading-soft"></span>
            </div>

            // Three across from `lg` rather than `xl`: on a laptop that keeps each
            // card narrow instead of stretching two of them across the full width.
            <div class="grid gap-5 mt-6 sm:grid-cols-2 lg:grid-cols-3">{cards}</div>

            {load_more}
        </section>
    }
}


#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::models::{SessionView, group_by_service_and_theme};

    /// Workshops the seed ships with, named here the way the server hands them
    /// over: resolved onto the view, not derived from anything the page knows.
    const APEROS: &str = "Apéros créatifs (adultes)";
    const APRES_MIDIS: &str = "Ateliers après-midi créatifs (adultes)";
    const PARENTS_ENFANTS: &str = "Ateliers parents-enfants (6 à 12 ans)";

    /// The wording of the button that asks the server for another batch, kept
    /// apart from the "voir +" each card carries towards its workshop's page.
    const MORE: &str = "Voir plus de séances";

    /// The hero on its own, resources and all.
    ///
    /// With no storage configured the resource resolves to an error and the
    /// `Transition` renders nothing, which is all this needs: the hero and the
    /// paragraph under it are synchronous, and they are what is being asserted.
    fn hero_html() -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        crate::pages::admin::init_test_executor();

        Owner::new().with(|| {
            provide_context(RequestUrl::new("/"));

            view! { <Router><HomePage/></Router> }.to_html()
        })
    }

    /// Every page owes a crawler one `h1`, and this is the page that most needs to
    /// say what the site is. It had none: the hero was an `h2` under a title that
    /// said nothing but the business name.
    #[test]
    fn the_home_page_opens_on_a_single_h1() {
        let html = hero_html();

        assert_eq!(html.matches("<h1").count(), 1, "not exactly one h1: {html}");
        assert!(html.contains("Bulle Créaline"), "{html}");
    }

    /// Nothing on this page used to say where the workshops are held -- not the
    /// town, not the département, not the address. A visitor could find out three
    /// clicks in, from the booking form; a search engine could not find out at all.
    #[test]
    fn the_home_page_says_where_the_workshops_are_held() {
        let html = hero_html();

        assert!(html.contains("Feurs"), "the town is not named: {html}");
        assert!(html.contains("Loire"), "the département is not named: {html}");
    }

    // That the page sets a title, a description and a canonical is asserted where
    // the tags can actually be seen -- in `components::seo`, which splices them into
    // a head the way the server does. They render nothing where they stand, so no
    // assertion on this page's own HTML could tell a correct one from a missing one.

    /// Whether the button carries the `disabled` attribute.
    ///
    /// Told apart from the classes rather than merely searched for: every button
    /// ships `disabled:opacity-50` and `disabled:pointer-events-none`, so a plain
    /// `contains("disabled")` is true of every render and asserts nothing. The
    /// attribute stands alone between spaces, the classes are followed by a colon.
    fn refuses_clicks(html: &str) -> bool {
        html.contains(" disabled ")
    }

    /// One card per given workshop, each on a theme of its own and carrying a date
    /// label of its own, so the cards can be told apart in the rendered markup.
    ///
    /// Built through the real grouping rather than by hand: a fixture that assembled
    /// a `SessionGroup` itself could describe a card the server never makes.
    fn cards(labels: &[&str]) -> Vec<SessionGroup> {
        labels
            .iter()
            .enumerate()
            .map(|(rank, label)| {
                group_by_service_and_theme(vec![SessionView {
                    id: format!("651d1f0a00000000000000{rank:02}"),
                    service_slug: format!("atelier-{rank}"),
                    service_label: (*label).to_owned(),
                    service_description: format!("Description de {label}."),
                    service_path: format!("/services/atelier-{rank}"),
                    date_label: format!("séance {rank}"),
                    date_input: "2026-07-05T14:00".to_owned(),
                    theme_id: format!("651d1f0a0000000000000{rank:03}"),
                    theme_name: format!("Thème {rank}"),
                    photo_url: format!("/media/theme/651d1f0a0000000000000{rank:03}"),
                    price: 65.0,
                    max_persons: 8,
                    booked_persons: 0,
                    min_persons: 1,
                }])
                .remove(0)
            })
            .collect()
    }

    /// Renders the grid as it stands once a batch has arrived and nothing is
    /// outstanding.
    fn sections_html(groups: Vec<SessionGroup>) -> String {
        sections_html_while(groups, true, false)
    }

    /// The same, with the two things the page would otherwise own: whether the
    /// server said more follow, and whether a click is waiting on an answer.
    fn sections_html_while(groups: Vec<SessionGroup>, more: bool, waiting: bool) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            provide_context(RequestUrl::new("/"));

            let shown = RwSignal::new(HOME_CARDS);

            view! {
                <Router>
                    <UpcomingSessions
                        groups=groups
                        more=more
                        waiting=waiting
                        shown=shown
                    />
                </Router>
            }
            .to_html()
        })
    }

    /// One flat grid: a single heading over the lot, and one card per offer
    /// whatever kind of workshop it belongs to.
    #[test]
    fn draws_one_grid_of_every_kind() {
        let html = sections_html(cards(&[APEROS, APRES_MIDIS, APEROS]));

        assert_eq!(html.matches("<section").count(), 1, "not one section: {html}");
        assert_eq!(html.matches("<article").count(), 3, "not three cards: {html}");
        assert_eq!(
            html.matches("Prochaines séances").count(),
            1,
            "the heading is not shown exactly once: {html}"
        );
    }

    /// Nothing sorts or regroups here, so the cards must come out in the order the
    /// server handed them over -- which is by their soonest date.
    #[test]
    fn keeps_the_order_it_was_given() {
        let html = sections_html(cards(&[APEROS, APRES_MIDIS, PARENTS_ENFANTS]));

        let positions: Vec<_> = (0..3)
            .map(|rank| {
                html.find(&format!("Thème {rank}"))
                    .unwrap_or_else(|| panic!("Thème {rank} is missing: {html}"))
            })
            .collect();

        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "the cards were reordered: {positions:?}"
        );
    }

    /// With no section per kind, the card's own chip is the only thing naming the
    /// workshop, so every kind on show has to be legible from the markup.
    #[test]
    fn every_card_still_names_its_workshop() {
        let labels = [APEROS, PARENTS_ENFANTS];
        let html = sections_html(cards(&labels));

        for label in labels {
            assert!(html.contains(label), "{label} is not named: {html}");
        }
    }

    /// The whole point of the button: the server said more cards follow this batch.
    ///
    /// On a full batch, which is the ordinary case -- the short one is a case of its
    /// own further down.
    #[test]
    fn offers_another_batch_when_the_server_says_more_follow() {
        let full = vec![APEROS; HOME_CARDS];
        let html = sections_html_while(cards(&full), true, false);

        assert_eq!(html.matches("<article").count(), HOME_CARDS, "not a full batch: {html}");
        assert!(html.contains(MORE), "no way to ask for more: {html}");
        assert!(!refuses_clicks(&html), "nothing is being waited on: {html}");
    }

    /// The mirror, and the reason the server looks one past its window at all: a
    /// button that stayed on the last batch would answer a click with nothing.
    #[test]
    fn offers_nothing_more_at_the_end_of_the_list() {
        let html = sections_html_while(cards(&[APEROS, APRES_MIDIS]), false, false);

        assert!(!html.contains(MORE), "the list has ended: {html}");
        assert_eq!(html.matches("<article").count(), 2, "the cards still show: {html}");
    }

    /// Between the click and the server's answer the grid stays put, so the button
    /// is the only thing that can say the click landed.
    #[test]
    fn the_button_says_so_while_it_waits() {
        let html = sections_html_while(cards(&[APEROS, APRES_MIDIS]), true, true);

        assert!(html.contains("Chargement…"), "the wait is invisible: {html}");
        assert!(!html.contains(MORE), "it should not still invite a second click: {html}");
        assert!(refuses_clicks(&html), "a waiting button should refuse clicks: {html}");
    }

    /// Grouping lets the server answer a batch of six with two cards and still have
    /// more to give: one workshop running one theme weekly fills a whole window on
    /// its own. The button must stay live through that.
    ///
    /// This is why waiting is told rather than inferred. Derived from the number of
    /// cards on screen, as it was before the merging, this render would show a
    /// button disabled on "Chargement…" with nothing in flight and no way back.
    #[test]
    fn a_short_batch_still_offers_more_without_waiting() {
        let html = sections_html_while(cards(&[APEROS, APRES_MIDIS]), true, false);

        assert_eq!(html.matches("<article").count(), 2, "fewer cards than a batch: {html}");
        assert!(html.contains(MORE), "the button should stay live: {html}");
        assert!(!refuses_clicks(&html), "nothing is in flight: {html}");
    }

    /// "voir +" means "go to this workshop" on every card. The button under the
    /// grid means something else entirely, so it must not borrow the wording.
    #[test]
    fn the_button_does_not_borrow_the_wording_of_the_cards() {
        let html = sections_html_while(cards(&[APEROS, APRES_MIDIS]), true, false);

        assert_eq!(
            html.matches("voir +").count(),
            2,
            "\"voir +\" should belong to the two cards alone: {html}"
        );
    }
}
