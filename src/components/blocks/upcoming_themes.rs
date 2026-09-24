use icons::{ChevronDown, ImageOff, Users};
use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize};
use crate::models::{SessionView, ThemeSessions};

/// Which part of the offer the workshop page is currently showing.
///
/// A state machine rather than a pair of booleans, on the model of the admin
/// pages' `Editing`: the three states are exclusive, and an enum makes that
/// impossible to get wrong.
#[derive(Clone, Debug, PartialEq)]
pub enum Showing {
    /// The grid alone, nothing unfolded.
    Themes,
    /// A theme's id: the grid, with its dates unfolded underneath.
    Theme(String),
    /// A session's id, which is how the visitor arrived from the home page.
    Session(String),
}

/// The themes of a workshop's upcoming sessions, and the dates behind them.
///
/// The state is a prop rather than a local signal so that the page can open it on
/// a session -- and so that every state can be rendered from a test, which a
/// signal born inside would put out of reach.
///
/// Only a bookable workshop gets this far, so an empty schedule is worth a
/// sentence: it is the page's one word on dates now that the presentation above
/// carries no booking button of its own.
#[component]
pub fn UpcomingThemes(groups: Vec<ThemeSessions>, showing: RwSignal<Showing>) -> impl IntoView {
    let body = if groups.is_empty() {
        Either::Left(view! {
            <p class="mt-6 text-muted-foreground">
                "Aucune date n'est programmée pour le moment."
            </p>
        })
    } else {
        Either::Right(view! { <Body groups=groups showing=showing/> })
    };

    view! {
        <section class="mx-auto mt-12 max-w-6xl">
            <h2 class="text-2xl font-bold lg:text-3xl text-heading">"Thèmes"</h2>
            <span class="block mt-3 w-10 h-1 rounded-full bg-heading-soft"></span>

            {body}
        </section>
    }
}

/// Picks what to draw for the current state.
///
/// Split out so the section above stays a shell: this closure re-runs on every
/// click, and rebuilding the heading with it would be wasted work.
#[component]
fn Body(groups: Vec<ThemeSessions>, showing: RwSignal<Showing>) -> impl IntoView {
    let groups = StoredValue::new(groups);

    move || match showing.get() {
        Showing::Session(id) => match find_session(&groups.get_value(), &id) {
            // A session that no longer exists -- deleted, or simply past since the
            // link was shared -- falls back to the grid rather than to a blank.
            None => EitherOf3::A(view! { <ThemeGrid groups=groups.get_value() showing=showing/> }),
            Some((theme_name, session)) => EitherOf3::B(view! {
                <div class="mt-6">
                    <BackToThemes showing=showing/>
                    <h3 class="mt-4 text-lg font-semibold text-heading">{theme_name}</h3>
                    <div class="mt-3">
                        <SessionRow session=session/>
                    </div>
                </div>
            }),
        },
        _ => EitherOf3::C(view! { <ThemeGrid groups=groups.get_value() showing=showing/> }),
    }
}

/// The session carrying this id, with the name of the theme it is filed under.
fn find_session(groups: &[ThemeSessions], id: &str) -> Option<(String, SessionView)> {
    groups.iter().find_map(|group| {
        group
            .sessions
            .iter()
            .find(|session| session.id == id)
            .map(|session| (group.theme_name.clone(), session.clone()))
    })
}

/// Sends the page back to the whole offer.
#[component]
fn BackToThemes(showing: RwSignal<Showing>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="inline-flex gap-1 items-center text-sm font-medium underline-offset-4 text-primary hover:underline"
            on:click=move |_| showing.set(Showing::Themes)
        >
            "‹ Tous les thèmes"
        </button>
    }
}

/// Every theme on offer, each card unfolding its own dates.
///
/// `items-start` so a card that unfolds grows alone: stretched to the row, its
/// closed neighbours would gain the same height in empty space.
#[component]
fn ThemeGrid(groups: Vec<ThemeSessions>, showing: RwSignal<Showing>) -> impl IntoView {
    let cards = groups
        .into_iter()
        .map(|group| {
            let id = group.theme_id.clone();
            let open = {
                let id = id.clone();
                move || showing.get() == Showing::Theme(id.clone())
            };

            view! {
                <li>
                    <ThemeCard group=group open=Signal::derive(open) showing=showing/>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <ul class="grid gap-5 items-start mt-6 sm:grid-cols-2 lg:grid-cols-3">{cards}</ul>
    }
}

/// One theme: its photo, its name, and its dates once unfolded at the foot.
///
/// The shell is a `div` and only the head is a `button`: a button may not hold
/// interactive content, and each date carries a "Réserver" link.
#[component]
fn ThemeCard(group: ThemeSessions, open: Signal<bool>, showing: RwSignal<Showing>) -> impl IntoView {
    let dates = match group.sessions.len() {
        1 => "1 date".to_owned(),
        count => format!("{count} dates"),
    };

    let theme_id = group.theme_id.clone();
    let theme_name = group.theme_name.clone();
    let sessions = StoredValue::new(group.sessions.clone());

    // Clicking the open theme closes it, so the button undoes itself rather than
    // leaving the only way back to a click on another card.
    let toggle = move |_| {
        showing
            .set(if open.get() { Showing::Themes } else { Showing::Theme(theme_id.clone()) });
    };

    let photo = if group.photo_url.is_empty() {
        // An empty `src` has the browser request the page itself as the image.
        Either::Left(view! {
            <div class="flex justify-center items-center w-full h-full bg-surface text-muted-foreground">
                <ImageOff class="w-7 h-7"/>
            </div>
        })
    } else {
        Either::Right(view! {
            <img
                src=group.photo_url
                alt=format!("Thème : {theme_name}")
                loading="lazy"
                class="object-cover w-full h-full transition-transform duration-300 group-hover:scale-105"
            />
        })
    };

    let head = view! {
        <button
            type="button"
            aria-expanded=move || open.get().to_string()
            // The chevron turns off this attribute rather than off a reactive
            // class: the icon component takes a plain `String`. Same device as
            // the navigation menu's triggers.
            data-state=move || if open.get() { "open" } else { "closed" }
            // `items-stretch` is not the decoration it looks like: browsers give
            // a button its own `align-items`, unlike a div, so without it the
            // photo's `aspect-16/9` box shrinks to its contents -- and the image
            // inside, sized in percentages, collapses to nothing.
            class="flex flex-col items-stretch w-full text-left group"
            on:click=toggle
        >
            // Rounded on the head alone: the foot below carries the bottom
            // corners, and a full round here would show through when unfolded.
            <div class="overflow-hidden relative rounded-t-[2rem] aspect-16/9">{photo}</div>

            <div class="flex gap-3 justify-between items-center p-4">
                <div class="flex flex-col gap-1">
                    <span class="text-base font-semibold leading-snug text-heading">
                        {group.theme_name}
                    </span>
                    <span class="text-sm text-muted-foreground">{dates}</span>
                </div>
                <ChevronDown class="w-5 h-5 shrink-0 text-heading-soft transition-transform group-data-[state=open]:rotate-180"/>
            </div>
        </button>
    };

    view! {
        <div class="flex overflow-hidden flex-col rounded-[2rem] border transition-shadow border-border bg-card text-card-foreground shadow-(--shadow) hover:shadow-lg">
            {head}

            // Mounted only while open, so the "Réserver" links of a folded theme
            // stay out of the tab order.
            <Show when=move || open.get()>
                <div class="px-4 pt-4 pb-4 border-t border-border">
                    <SessionList sessions=sessions.get_value()/>
                </div>
            </Show>
        </div>
    }
}

/// The dates of one theme.
#[component]
fn SessionList(sessions: Vec<SessionView>) -> impl IntoView {
    let rows = sessions
        .into_iter()
        .map(|session| view! { <li><SessionRow session=session/></li> })
        .collect::<Vec<_>>();

    view! { <ul class="flex flex-col gap-3">{rows}</ul> }
}

/// One date, with what it costs, what is left of it, and how to take it.
#[component]
fn SessionRow(session: SessionView) -> impl IntoView {
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

    fn session(id: &str, date_label: &str, theme_id: &str, theme_name: &str) -> SessionView {
        SessionView {
            id: id.to_owned(),
            service_slug: "aperos-creatifs".to_owned(),
            service_label: "Apéros créatifs".to_owned(),
            service_description: "Une soirée entre adultes.".to_owned(),
            service_path: "/services/aperos-creatifs".to_owned(),
            date_label: date_label.to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: theme_id.to_owned(),
            theme_name: theme_name.to_owned(),
            photo_url: format!("/media/theme/{theme_id}?v=1"),
            price: 65.0,
            max_persons: 8,
            booked_persons: 2,
        }
    }

    /// Two themes whose names and dates share nothing, so an assertion about one
    /// cannot pass on the other.
    fn groups() -> Vec<ThemeSessions> {
        crate::models::group_by_theme(vec![
            session("s1", "dimanche 5 juillet", "t1", "Aquarelle"),
            session("s2", "dimanche 12 juillet", "t1", "Aquarelle"),
            session("s3", "samedi 18 juillet", "t2", "Collage"),
        ])
    }

    fn render(groups: Vec<ThemeSessions>, state: Showing) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The booking buttons are links, and a link resolves `aria-current`
            // against the location being rendered.
            provide_context(RequestUrl::new("/services/aperos-creatifs"));

            let showing = RwSignal::new(state);

            view! {
                <Router>
                    <UpcomingThemes groups=groups showing=showing/>
                </Router>
            }
            .to_html()
        })
    }

    #[test]
    fn the_grid_names_every_theme_and_counts_its_dates() {
        let html = render(groups(), Showing::Themes);

        assert!(html.contains("Aquarelle"), "no theme: {html}");
        assert!(html.contains("Collage"), "no second theme: {html}");
        assert!(html.contains("2 dates"), "Aquarelle has two: {html}");
        assert!(html.contains("1 date"), "Collage has one: {html}");
        assert!(html.contains("/media/theme/t1?v=1"), "no photo: {html}");
    }

    /// The dates are what the click is for, so they must not already be there.
    #[test]
    fn the_grid_alone_shows_no_date() {
        let html = render(groups(), Showing::Themes);

        assert!(!html.contains("dimanche 5 juillet"), "a date slipped in: {html}");
        assert!(!html.contains("samedi 18 juillet"), "a date slipped in: {html}");
    }

    #[test]
    fn unfolding_a_theme_shows_its_dates_and_no_others() {
        let html = render(groups(), Showing::Theme("t1".to_owned()));

        assert!(html.contains("dimanche 5 juillet"), "no date: {html}");
        assert!(html.contains("dimanche 12 juillet"), "only one date: {html}");
        assert!(
            !html.contains("samedi 18 juillet"),
            "another theme's date showed: {html}"
        );
        // The grid stays: unfolding happens in place, it does not replace it.
        assert!(html.contains("Collage"), "the grid should stay: {html}");
    }

    /// The dates belong to the card that was clicked, not to a panel under the
    /// grid. Proved by where they fall: after the first theme's name and before
    /// the second theme's card begins, which is inside the first card and nowhere
    /// else. Splitting on `<li` cannot serve here -- the date list carries its
    /// own list items.
    #[test]
    fn the_dates_unfold_inside_their_card() {
        let html = render(groups(), Showing::Theme("t1".to_owned()));

        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle:?} is missing: {html}"))
        };

        assert!(at("Aquarelle") < at("dimanche 5 juillet"), "the date precedes its card");
        assert!(
            at("dimanche 12 juillet") < at("Collage"),
            "the dates spilled past their card and into the next"
        );
        assert!(
            at("/booking/aperos-creatifs") < at("Collage"),
            "the booking link sits outside the card"
        );
    }

    /// Arriving from the home page lands on the one date that was clicked.
    #[test]
    fn a_session_arrived_at_from_the_home_page_stands_alone() {
        let html = render(groups(), Showing::Session("s2".to_owned()));

        assert!(html.contains("dimanche 12 juillet"), "no date: {html}");
        assert!(!html.contains("dimanche 5 juillet"), "a sibling date showed: {html}");
        assert!(!html.contains("Collage"), "the grid should be gone: {html}");
        assert!(html.contains("Tous les thèmes"), "no way back: {html}");
    }

    /// A shared link outlives the session it names, so an id nobody recognises
    /// has to land on the offer rather than on nothing.
    #[test]
    fn an_unknown_session_falls_back_to_the_grid() {
        let html = render(groups(), Showing::Session("gone".to_owned()));

        assert!(html.contains("Aquarelle"), "no grid: {html}");
        assert!(html.contains("Collage"), "no grid: {html}");
    }

    /// The card is a `<button>`, and a button carries an `align-items` of its own
    /// where a div would stretch its children. Without this class the photo's
    /// aspect box shrinks to its contents and the image vanishes -- the card
    /// still renders, so only this assertion catches it.
    #[test]
    fn the_card_stretches_its_photo_despite_being_a_button() {
        let html = render(groups(), Showing::Themes);

        let card = html
            .split("<button")
            .find(|fragment| fragment.contains("aspect-16/9"))
            .expect("a theme card should render");

        assert!(card.contains("items-stretch"), "{card}");
    }

    /// The date was chosen here, so the booking page should open on it rather
    /// than present the same list a second time.
    #[test]
    fn booking_a_date_carries_it_to_the_booking_page() {
        let html = render(groups(), Showing::Theme("t1".to_owned()));

        assert!(html.contains("/booking/aperos-creatifs?session=s1"), "{html}");
        assert!(html.contains("/booking/aperos-creatifs?session=s2"), "{html}");
    }

    /// An empty `src` has the browser request the page itself as the image.
    #[test]
    fn a_theme_without_a_photo_renders_no_empty_image() {
        let mut bare = groups();
        bare[0].photo_url = String::new();
        bare.truncate(1);

        let html = render(bare, Showing::Themes);

        assert!(!html.contains("<img"), "no image should render: {html}");
        assert!(html.contains("Aquarelle"), "the theme still shows: {html}");
    }

    /// The presentation above carries no booking button any more, so a workshop
    /// with nothing booked in has to say so rather than fall silent on dates.
    #[test]
    fn a_workshop_with_nothing_scheduled_says_so() {
        let html = render(Vec::new(), Showing::Themes);

        assert!(html.contains("Aucune date"), "{html}");
    }
}
