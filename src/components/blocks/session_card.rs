use icons::{ArrowRight, ImageOff};
use leptos::either::Either;
use leptos::prelude::*;

use crate::components::blocks::session_list::SessionList;
use crate::models::SessionGroup;

/// One workshop on one theme, as the home page advertises it.
///
/// A card used to be one session, and a workshop running weekly filled the grid
/// with near-identical tiles differing only in their date. The sessions sharing a
/// workshop and a theme are the same offer on different days, so they share a card
/// and the dates sit inside it.
///
/// Everything shown is already on the [`SessionGroup`], the workshop's own wording
/// included, so the card needs no lookup of its own. That matters more than it used
/// to: workshops are rows now, so a card that resolved its own would turn one grid
/// into one request per tile.
///
/// This neither sorts nor truncates: the dates on screen are the dates it was given,
/// in the order it was given them. Which to show and how many are the server's call,
/// and `further` is the server's count of the rest.
#[component]
pub fn SessionCard(group: SessionGroup) -> impl IntoView {
    let further = group.further_label().map(|label| {
        view! { <p class="text-sm text-muted-foreground">{label}</p> }
    });

    let SessionGroup {
        service_label,
        service_description,
        service_path,
        theme_id,
        theme_name,
        photo_url,
        sessions,
        ..
    } = group;

    // The workshop page opens on this theme, its dates unfolded, rather than on its
    // grid: the card has just said there are more of them, and that page is where
    // they are. Not carried when the workshop is gone: `service_path` is then "/",
    // where the parameter would name nothing.
    let more_path = if service_path == "/" {
        service_path
    } else {
        format!("{service_path}?theme={theme_id}")
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

    view! {
        <article class="flex overflow-hidden flex-col rounded-[2rem] border transition-shadow group border-border bg-card text-card-foreground shadow-(--shadow) hover:shadow-lg">

            // The theme is the card's title: a card is one workshop on one theme,
            // and the dates below are what it offers. The date used to stand here,
            // back when a card was one date -- it has moved into the list in the
            // footer, where several of them can sit. No icon beside it, for the
            // same reason as before: a heading next to a glyph reads as a list item
            // rather than a title.
            <header class="flex flex-col gap-1.5 items-start px-4 pt-4 pb-3">
                <p class="text-base font-semibold leading-snug text-heading">{theme_name}</p>
                // The chip is the only thing naming which workshop these dates
                // belong to, the grid being one flat list. Its text sits on the
                // pale surface rather than in the accent colour, which at this
                // size would not clear the contrast floor.
                <span class="px-2.5 py-0.5 text-xs font-medium rounded-full bg-surface text-surface-foreground">
                    {service_label}
                </span>
            </header>

            // 16/9 rather than the 4/3 a photo usually gets: the picture is here
            // to set a mood, and a quarter less height leaves room for the dates
            // underneath without pushing the next card off the fold.
            <div class="overflow-hidden relative aspect-16/9">
                {photo}
            </div>

            <footer class="flex flex-col flex-1 gap-2.5 p-4">
                <p class="text-sm leading-relaxed text-muted-foreground">
                    {service_description}
                </p>

                // The same rows the workshop page unfolds under a theme, so the two
                // places that list dates list them the same way -- and each date
                // carries its own price and its own way in, which a card stating
                // one price above them could not.
                <SessionList sessions=sessions/>

                // Right under the list it counts rather than down on the link row:
                // cards in a row stretch to the tallest, which would otherwise
                // float this line away from the dates it is talking about.
                {further}

                <div class="flex gap-3 justify-end items-center pt-1 mt-auto">
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
    use crate::models::{SessionView, group_by_service_and_theme};

    /// An offer already resolved the way the server hands it over: the workshop is
    /// a row now, so its wording arrives on the view rather than being derived.
    const LABEL: &str = "Apéros créatifs (adultes)";
    const DESCRIPTION: &str = "Une soirée entre adultes autour d'un verre.";
    const THEME: &str = "Aquarelle";

    fn date(id: &str, label: &str, price: f64, booked_persons: u32) -> SessionView {
        SessionView {
            id: id.to_owned(),
            service_slug: "aperos-creatifs".to_owned(),
            service_label: LABEL.to_owned(),
            service_description: DESCRIPTION.to_owned(),
            service_path: "/services/aperos-creatifs".to_owned(),
            date_label: label.to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000099".to_owned(),
            theme_name: THEME.to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000099".to_owned(),
            price,
            max_persons: 8,
            booked_persons,
            min_persons: 1,
        }
    }

    /// One card over the given dates, announcing `further` more behind them. Built
    /// through the real grouping rather than by hand, so a change to what a card
    /// carries cannot leave the fixture describing something that is no longer made.
    fn card(dates: Vec<SessionView>, further: usize) -> SessionGroup {
        let mut group = group_by_service_and_theme(dates).remove(0);
        group.further = further;

        group
    }

    fn one_date(booked_persons: u32) -> SessionGroup {
        card(vec![date("s1", "dimanche 5 juillet", 65.0, booked_persons)], 0)
    }

    fn card_html(group: SessionGroup) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The booking buttons are links, and a link resolves `aria-current`
            // against the location being rendered.
            provide_context(RequestUrl::new("/"));

            view! { <Router><SessionCard group=group/></Router> }.to_html()
        })
    }

    #[test]
    fn a_card_shows_the_whole_offer() {
        let html = card_html(card(
            vec![
                date("s1", "dimanche 5 juillet", 65.0, 0),
                date("s2", "dimanche 12 juillet", 65.0, 0),
            ],
            0,
        ));

        assert!(html.contains(THEME), "no theme: {html}");
        assert!(html.contains(LABEL), "no workshop: {html}");
        assert!(html.contains(DESCRIPTION), "no description: {html}");
        assert!(html.contains("dimanche 5 juillet"), "no first date: {html}");
        assert!(html.contains("dimanche 12 juillet"), "no second date: {html}");
    }

    /// The whole point of merging: one workshop running the same theme twice is one
    /// tile with two dates, not two tiles.
    #[test]
    fn several_dates_sit_on_one_card() {
        let html = card_html(card(
            vec![
                date("s1", "dimanche 5 juillet", 65.0, 0),
                date("s2", "dimanche 12 juillet", 65.0, 0),
                date("s3", "dimanche 19 juillet", 65.0, 0),
            ],
            0,
        ));

        assert_eq!(html.matches("<article").count(), 1, "not one card: {html}");
        assert_eq!(html.matches("<li").count(), 3, "not one row per date: {html}");
    }

    /// Nothing here sorts: the dates come out as the server ranked them, which is
    /// soonest first.
    #[test]
    fn lists_the_dates_in_the_order_it_was_given() {
        let html = card_html(card(
            vec![
                date("s1", "dimanche 5 juillet", 65.0, 0),
                date("s2", "dimanche 12 juillet", 65.0, 0),
            ],
            0,
        ));

        let first = html.find("dimanche 5 juillet").expect("the first date should render");
        let second = html.find("dimanche 12 juillet").expect("the second should too");

        assert!(first < second, "the dates were reordered: {html}");
    }

    #[test]
    fn counts_the_dates_it_is_not_listing() {
        let html = card_html(card(vec![date("s1", "dimanche 5 juillet", 65.0, 0)], 4));

        assert!(html.contains("+ 4 autres dates"), "the rest is unannounced: {html}");
    }

    #[test]
    fn says_nothing_when_it_is_listing_them_all() {
        let html = card_html(one_date(0));

        assert!(!html.contains("autres dates"), "there is nothing to announce: {html}");
        assert!(!html.contains("autre date"), "nor in the singular: {html}");
    }

    /// A full date keeps its row but loses its link, and its siblings keep theirs:
    /// one session filling up says nothing about the next one.
    #[test]
    fn a_full_date_offers_no_booking_but_its_siblings_still_do() {
        let html = card_html(card(
            vec![
                date("s1", "dimanche 5 juillet", 65.0, 8),
                date("s2", "dimanche 12 juillet", 65.0, 0),
            ],
            0,
        ));

        assert!(html.contains("Complet"), "the full date should say so: {html}");
        assert!(
            html.contains("/booking/aperos-creatifs?session=s2"),
            "the open date is still bookable: {html}"
        );
        assert!(
            !html.contains("?session=s1"),
            "the full date should carry no booking link: {html}"
        );
    }

    /// The card promises dates it is not showing, so the link has to land where
    /// they all are -- the theme unfolded on the workshop's page, not one session
    /// of it shown alone.
    #[test]
    fn the_more_link_leads_to_the_theme_it_was_clicked_from() {
        let html = card_html(one_date(0));

        assert!(
            html.contains("/services/aperos-creatifs?theme=651d1f0a0000000000000099"),
            "\"voir +\" should open the theme: {html}"
        );
    }

    /// A workshop whose row was deleted has no page left to open, and `service_path`
    /// is then "/" -- where a theme parameter would name nothing.
    #[test]
    fn a_card_whose_workshop_is_gone_carries_no_parameter() {
        let mut group = one_date(0);
        group.service_path = "/".to_owned();

        let html = card_html(group);

        // Bounded to the "voir +" anchor: the booking links beside it carry their
        // own query, and a check over the whole card would pass on those.
        let anchor = html
            .split("<a ")
            .find(|fragment| fragment.contains("voir +"))
            .expect(r#"the "voir +" link should render"#);

        assert!(anchor.contains(r#"href="/""#), "it should lead home: {anchor}");
        assert!(!anchor.contains("?theme="), "there is no theme to open: {anchor}");
    }

    #[test]
    fn a_card_without_a_photo_renders_no_empty_image() {
        let mut group = one_date(0);
        group.photo_url = String::new();

        let html = card_html(group);

        assert!(!html.contains("<img"), "an empty src should draw nothing: {html}");
        assert!(html.contains(THEME), "the theme still shows: {html}");
    }

    /// The theme stands above the photo, and the photo above everything the footer
    /// holds. Asserted on the first occurrence of the theme, which is why the photo
    /// has to come second: its `alt` carries the theme name too, so a card that lost
    /// its header would still match further down.
    #[test]
    fn the_card_is_laid_out_header_photo_then_footer() {
        let html = card_html(one_date(0));

        let theme = html.find(THEME).expect("the theme should render");
        let photo = html.find("<img").expect("the photo should render");
        let description = html.find(DESCRIPTION).expect("the description should render");
        let date = html.find("dimanche 5 juillet").expect("the date should render");

        assert!(
            theme < photo && photo < description && description < date,
            "out of order: theme {theme}, photo {photo}, description {description}, date {date}"
        );
    }
}
