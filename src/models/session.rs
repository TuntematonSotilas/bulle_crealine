use serde::{Deserialize, Serialize};

use crate::models::ServiceView;

/// How many cards the home page shows at a time, every kind of workshop mixed
/// together and soonest first.
///
/// A card is one workshop on one theme, however many dates it carries, so this
/// counts cards rather than dates. That is what keeps the shop window full: a
/// workshop running weekly used to take a whole batch to itself, six near-identical
/// tiles differing only in their date.
///
/// The home page opens on this many and asks for this many more each time the
/// visitor says so. A visitor who wants the whole run of dates for one workshop is
/// still better served by its own page, which the "voir +" on each card points at.
pub const HOME_CARDS: usize = 6;

/// The most the home page will ever hand out, however many batches are asked for.
///
/// The count travels from the browser, so it is an untrusted number: without a
/// ceiling a crafted request would have the server read, resolve and serialise the
/// whole collection. Twenty batches is far past what anyone scrolls, and reaching
/// it leaves the list simply ending -- which is what it does at the real end too.
pub const HOME_CARDS_MAX: usize = HOME_CARDS * 20;

/// How many dates one card lists before falling back to a count.
///
/// Three shows that a workshop runs more than once while leaving the card roughly
/// the height it had when it carried a single date. A card ten rows tall would
/// stretch its whole line of the grid.
pub const HOME_CARD_DATES: usize = 3;

/// How many sessions are read for each card asked for.
///
/// Cards are made of sessions, and how many sessions a card swallows is not known
/// until they have been read, so the window has to be wider than the batch. Four
/// rather than two because it has two jobs at once: finding enough distinct
/// workshop-and-theme pairs, and holding enough dates on each to fill the three
/// slots a card lists.
pub const SESSIONS_PER_CARD: usize = 4;

/// One batch of the home page's cards, and whether any follow.
///
/// `more` is answered by the server because only it can tell "the batch happens to
/// be full" from "this is everything". Without it the button would show on a total
/// that is an exact multiple of the batch and then do nothing when clicked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpcomingPage {
    /// Soonest first, at most the number asked for. May be empty.
    pub groups: Vec<SessionGroup>,
    /// Whether asking for a larger batch would bring anything new.
    pub more: bool,
}

/// One workshop on one theme, with the upcoming dates it is running.
///
/// This is one card on the home page. Sessions sharing both a workshop and a theme
/// are the same offer on different days, so they belong on one tile.
///
/// The workshop and the theme are carried here as well as on every session inside:
/// they are the same on all of them by construction, and the header reads them from
/// one place rather than from a session it would have to assume exists. Nothing may
/// index `sessions` -- the type is deserialised off the wire, so no constructor can
/// promise it is not empty, and a card with no date must degrade to a card with no
/// rows rather than panic in the browser.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionGroup {
    /// Half the grouping key. The dates inside build their own booking links from
    /// their own copy of it.
    pub service_slug: String,
    pub service_label: String,
    pub service_description: String,
    /// Where that workshop's own page lives, `"/"` when it has been deleted.
    pub service_path: String,
    /// The other half of the key, and what `?theme=` hands to that page.
    pub theme_id: String,
    pub theme_name: String,
    /// Empty when the theme has been deleted, exactly as on a session.
    pub photo_url: String,
    /// In the order they were given, which is soonest first. Never empty as built,
    /// and at most [`HOME_CARD_DATES`] long once the server has trimmed.
    pub sessions: Vec<SessionView>,
    /// How many further dates this card is not listing.
    ///
    /// Counted by the server rather than derived here: a card is handed the dates
    /// it shows and nothing else, so it has no way to know.
    pub further: usize,
}

impl SessionGroup {
    /// Keeps the soonest `keep` dates and counts the rest into `further`.
    ///
    /// The count it leaves behind is only of the dates it was given. The server
    /// replaces it with a true total, which this cannot know: a card's own dates
    /// run past whatever window produced it.
    pub fn keep_soonest(&mut self, keep: usize) {
        self.further = self.sessions.len().saturating_sub(keep);
        self.sessions.truncate(keep);
    }

    /// Line standing in for the dates the card is not listing, or `None` when it is
    /// listing them all.
    pub fn further_label(&self) -> Option<String> {
        match self.further {
            0 => None,
            1 => Some("+ 1 autre date".to_owned()),
            count => Some(format!("+ {count} autres dates")),
        }
    }
}

/// Gathers sessions under the workshop and theme they share, keeping the order they
/// arrived in.
///
/// Two halves to the key where [`group_by_theme`] has one: the home page mixes every
/// workshop together, so the same theme run by two workshops is two offers, and a
/// visitor choosing between them is choosing between the workshops.
///
/// A card takes the rank of its soonest date, the list being already sorted by date.
/// That also makes a wider read a prefix-extension of a narrower one: asking for
/// another batch appends cards and appends dates, rather than reshuffling what is
/// already on screen under the visitor.
pub fn group_by_service_and_theme(sessions: Vec<SessionView>) -> Vec<SessionGroup> {
    let mut groups: Vec<SessionGroup> = Vec::new();

    for session in sessions {
        let found = groups.iter_mut().find(|group| {
            group.service_slug == session.service_slug && group.theme_id == session.theme_id
        });

        match found {
            Some(group) => group.sessions.push(session),
            None => groups.push(SessionGroup {
                service_slug: session.service_slug.clone(),
                service_label: session.service_label.clone(),
                service_description: session.service_description.clone(),
                service_path: session.service_path.clone(),
                theme_id: session.theme_id.clone(),
                theme_name: session.theme_name.clone(),
                photo_url: session.photo_url.clone(),
                sessions: vec![session],
                further: 0,
            }),
        }
    }

    groups
}

/// A session as the browser sees it.
///
/// Dates arrive already formatted, so the WASM bundle needs neither a date nor a
/// timezone library. Capacity arrives already resolved, so a page can tell
/// whether a session is full without a second round trip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionView {
    /// Hex form of the Mongo `ObjectId`.
    pub id: String,
    /// Slug of the kind of workshop, which is what the admin form posts back and
    /// what `/booking/<slug>` carries.
    pub service_slug: String,
    /// Name of that workshop, and the two lines below: resolved server-side.
    ///
    /// Workshops are rows now rather than an enum, so a page could no longer
    /// derive any of this from the slug on its own. Carried here rather than
    /// fetched by the card, which would turn one grid into one request per tile.
    pub service_label: String,
    pub service_description: String,
    /// Where that workshop's own page lives, `"/"` when it has been deleted.
    pub service_path: String,
    /// Human label, e.g. `"dimanche 5 juillet 2026 à 14h00"`.
    pub date_label: String,
    /// `"2026-07-05T14:00"`, ready for an `<input type="datetime-local">`.
    pub date_input: String,
    /// Hex id of the theme, which is what the admin form posts back.
    pub theme_id: String,
    /// Name of that theme, resolved server-side so no page has to join anything.
    pub theme_name: String,
    /// Where to fetch the theme's photo, stamp included, or empty when the theme
    /// is gone. Resolved here rather than rebuilt from `theme_id` by the page,
    /// which has no way to know the stamp.
    pub photo_url: String,
    pub price: f64,
    /// How many people the session can take in total.
    pub max_persons: u32,
    /// How many people are already booked, summed over every booking.
    pub booked_persons: u32,
    /// Smallest party its workshop takes, carried along so a page can tell a
    /// session that is full apart from one that merely has too little left.
    ///
    /// Never zero, a session whose workshop is gone reading one: every use is a
    /// `remaining < min_persons`, and no `u32` is below zero.
    pub min_persons: u32,
}

/// What the booking page shows, fetched in one round trip.
///
/// The workshop comes along with its dates because the page needs it even when
/// there are none: it still has to name the workshop and link back to its page.
/// Workshops being rows now, that could no longer be derived from the URL.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingOffer {
    pub service: ServiceView,
    /// Upcoming sessions of that workshop, soonest first. May be empty.
    pub sessions: Vec<SessionView>,
}

/// One theme, with the upcoming sessions filed under it.
///
/// Built from the sessions rather than fetched: a session already carries its
/// theme's name and photo, so a workshop page can show what is on offer without
/// a second round trip -- and without a public themes endpoint, which does not
/// exist.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeSessions {
    pub theme_id: String,
    pub theme_name: String,
    /// Empty when the theme has been deleted, exactly as on a session.
    pub photo_url: String,
    /// In the order they were given, which is soonest first.
    pub sessions: Vec<SessionView>,
}

/// Gathers sessions under their theme, keeping the order they arrived in.
///
/// A theme takes the rank of its soonest session, since the list is already
/// sorted by date: what a visitor sees first is what is happening first, rather
/// than an alphabet or an id.
pub fn group_by_theme(sessions: Vec<SessionView>) -> Vec<ThemeSessions> {
    let mut groups: Vec<ThemeSessions> = Vec::new();

    for session in sessions {
        match groups.iter_mut().find(|group| group.theme_id == session.theme_id) {
            Some(group) => group.sessions.push(session),
            None => groups.push(ThemeSessions {
                theme_id: session.theme_id.clone(),
                theme_name: session.theme_name.clone(),
                photo_url: session.photo_url.clone(),
                sessions: vec![session],
            }),
        }
    }

    groups
}

impl SessionView {
    /// How many people can still be booked.
    pub fn remaining_places(&self) -> u32 {
        self.max_persons.saturating_sub(self.booked_persons)
    }

    /// Whether the session can still take a booking.
    ///
    /// What is left is measured against the smallest party the workshop allows,
    /// not against zero: one place on a workshop attended in pairs is a place
    /// nobody can take, and offering it would send the visitor to a refusal.
    pub fn is_full(&self) -> bool {
        self.remaining_places() < self.min_persons
    }

    /// Sentence describing what is left, for display next to the session.
    pub fn availability_label(&self) -> String {
        // Asked of `is_full` rather than of the count, so the sentence and the
        // button can never disagree about the same session.
        if self.is_full() {
            return "Complet".to_owned();
        }

        match self.remaining_places() {
            1 => "1 place restante".to_owned(),
            remaining => format!("{remaining} places restantes"),
        }
    }

    /// Price with its unit, as shown to visitors.
    pub fn price_label(&self) -> String {
        // Drop the decimals on whole prices: "65 €" reads better than "65.00 €".
        if self.price.fract() == 0.0 {
            format!("{} €", self.price as i64)
        } else {
            format!("{:.2} €", self.price)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(max_persons: u32, booked_persons: u32) -> SessionView {
        SessionView {
            id: "651d1f0a0000000000000000".to_owned(),
            service_slug: "apres-midis-creatifs".to_owned(),
            service_label: "Ateliers après-midi créatifs (adultes)".to_owned(),
            service_description: "Des ateliers pour tous les niveaux.".to_owned(),
            service_path: "/services/apres-midis-creatifs".to_owned(),
            date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000001".to_owned(),
            theme_name: "Sculpture".to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000001?v=1".to_owned(),
            price: 65.0,
            max_persons,
            booked_persons,
            min_persons: 1,
        }
    }

    #[test]
    fn reports_remaining_places() {
        assert_eq!(session(8, 0).remaining_places(), 8);
        assert_eq!(session(8, 3).remaining_places(), 5);
        assert_eq!(session(8, 8).remaining_places(), 0);
    }

    #[test]
    fn treats_a_session_as_full_once_capacity_is_reached() {
        assert!(!session(8, 7).is_full());
        assert!(session(8, 8).is_full());
    }

    /// Overbooking can happen when two bookings land at the same moment; the
    /// session must read as full rather than as having negative room left.
    #[test]
    fn survives_being_overbooked() {
        let overbooked = session(8, 11);

        assert_eq!(overbooked.remaining_places(), 0);
        assert!(overbooked.is_full());
        assert_eq!(overbooked.availability_label(), "Complet");
    }

    #[test]
    fn spells_out_availability() {
        assert_eq!(session(8, 8).availability_label(), "Complet");
        assert_eq!(session(8, 7).availability_label(), "1 place restante");
        assert_eq!(session(8, 5).availability_label(), "3 places restantes");
    }

    /// The same session, on a workshop attended in pairs.
    fn in_pairs(max_persons: u32, booked_persons: u32) -> SessionView {
        SessionView { min_persons: 2, ..session(max_persons, booked_persons) }
    }

    /// One place on a workshop attended in pairs is a place nobody can take, and
    /// offering it would send the visitor to a refusal.
    ///
    /// This is also the only test that would notice `min_persons` regressing to
    /// zero: `remaining < 0` is false for every `u32`, so a zero would not relax
    /// the rule but switch it off.
    #[test]
    fn a_place_too_few_for_a_pair_reads_as_full() {
        let session = in_pairs(8, 7);

        assert_eq!(session.remaining_places(), 1, "there is a place left");
        assert!(session.is_full(), "but no booking could take it");
        assert_eq!(session.availability_label(), "Complet");
    }

    /// Only the last place is withheld: a pair still fits in two, and in three.
    #[test]
    fn a_pair_workshop_stays_open_while_a_pair_fits() {
        assert!(!in_pairs(8, 6).is_full());
        assert_eq!(in_pairs(8, 6).availability_label(), "2 places restantes");
        assert_eq!(in_pairs(8, 5).availability_label(), "3 places restantes");
    }

    #[test]
    fn drops_decimals_on_whole_prices() {
        let mut view = session(8, 0);
        assert_eq!(view.price_label(), "65 €");

        view.price = 62.5;
        assert_eq!(view.price_label(), "62.50 €");
    }

    /// One session under the named theme, with an id of its own so a group's
    /// contents can be told apart.
    fn dated(id: &str, theme_id: &str, theme_name: &str) -> SessionView {
        SessionView {
            id: id.to_owned(),
            theme_id: theme_id.to_owned(),
            theme_name: theme_name.to_owned(),
            ..session(8, 0)
        }
    }

    #[test]
    fn gathers_the_sessions_of_one_theme_under_one_group() {
        let groups = group_by_theme(vec![
            dated("a", "theme-1", "Aquarelle"),
            dated("b", "theme-1", "Aquarelle"),
        ]);

        assert_eq!(groups.len(), 1, "one theme should give one group: {groups:?}");
        assert_eq!(groups[0].theme_name, "Aquarelle");
        assert_eq!(groups[0].sessions.len(), 2, "both dates should be kept");
    }

    /// The list arrives sorted by date, so a theme takes the rank of its soonest
    /// session: what shows first is what happens first.
    #[test]
    fn a_theme_ranks_by_its_soonest_session() {
        let groups = group_by_theme(vec![
            dated("a", "theme-2", "Collage"),
            dated("b", "theme-1", "Aquarelle"),
            dated("c", "theme-2", "Collage"),
        ]);

        let order: Vec<&str> = groups.iter().map(|group| group.theme_name.as_str()).collect();

        assert_eq!(order, vec!["Collage", "Aquarelle"], "not the order they came in");
    }

    /// A workshop with nothing scheduled must give nothing to show, rather than
    /// an empty group standing under a heading.
    #[test]
    fn no_session_gives_no_group() {
        assert!(group_by_theme(Vec::new()).is_empty());
    }

    /// One session of the named workshop under the named theme. Both halves of the
    /// grouping key vary, which is what the tests below need to tell apart.
    fn offered(id: &str, slug: &str, theme_id: &str) -> SessionView {
        SessionView {
            id: id.to_owned(),
            service_slug: slug.to_owned(),
            service_label: format!("Atelier {slug}"),
            theme_id: theme_id.to_owned(),
            theme_name: format!("Thème {theme_id}"),
            ..session(8, 0)
        }
    }

    #[test]
    fn gathers_the_dates_of_one_workshop_and_theme_under_one_card() {
        let cards = group_by_service_and_theme(vec![
            offered("a", "aperos", "theme-1"),
            offered("b", "aperos", "theme-1"),
        ]);

        assert_eq!(cards.len(), 1, "one offer should give one card: {cards:?}");
        assert_eq!(cards[0].sessions.len(), 2, "both dates should be kept: {cards:?}");
        assert_eq!(cards[0].service_slug, "aperos");
        assert_eq!(cards[0].theme_id, "theme-1");
    }

    /// The whole of what tells this apart from [`group_by_theme`]. Without it a
    /// version keyed on the theme alone would pass every other test here, and the
    /// home page would merge two workshops that happen to share a theme.
    #[test]
    fn the_same_theme_on_two_workshops_gives_two_cards() {
        let cards = group_by_service_and_theme(vec![
            offered("a", "aperos", "theme-1"),
            offered("b", "parents-enfants", "theme-1"),
        ]);

        assert_eq!(cards.len(), 2, "two workshops are two offers: {cards:?}");
    }

    /// The other half of the key, for the same reason.
    #[test]
    fn the_same_workshop_on_two_themes_gives_two_cards() {
        let cards = group_by_service_and_theme(vec![
            offered("a", "aperos", "theme-1"),
            offered("b", "aperos", "theme-2"),
        ]);

        assert_eq!(cards.len(), 2, "two themes are two offers: {cards:?}");
    }

    /// The list arrives sorted by date, so a card takes the rank of its soonest
    /// date: what shows first is what happens first.
    #[test]
    fn a_card_ranks_by_its_soonest_date() {
        let cards = group_by_service_and_theme(vec![
            offered("a", "parents-enfants", "theme-2"),
            offered("b", "aperos", "theme-1"),
            offered("c", "parents-enfants", "theme-2"),
        ]);

        let order: Vec<&str> = cards.iter().map(|card| card.service_slug.as_str()).collect();

        assert_eq!(order, ["parents-enfants", "aperos"], "the cards were reordered");
    }

    #[test]
    fn no_session_gives_no_card() {
        assert!(group_by_service_and_theme(Vec::new()).is_empty());
    }

    #[test]
    fn keeps_the_soonest_dates_and_counts_the_rest() {
        let mut card = group_by_service_and_theme(vec![
            offered("a", "aperos", "theme-1"),
            offered("b", "aperos", "theme-1"),
            offered("c", "aperos", "theme-1"),
            offered("d", "aperos", "theme-1"),
        ])
        .remove(0);

        card.keep_soonest(3);

        let kept: Vec<&str> = card.sessions.iter().map(|date| date.id.as_str()).collect();
        assert_eq!(kept, ["a", "b", "c"], "the soonest three should be the ones kept");
        assert_eq!(card.further, 1, "the fourth should be counted, not dropped");
    }

    /// A card with fewer dates than it may list keeps them all and claims nothing.
    #[test]
    fn a_card_with_nothing_to_hide_counts_nothing() {
        let mut card = group_by_service_and_theme(vec![
            offered("a", "aperos", "theme-1"),
            offered("b", "aperos", "theme-1"),
        ])
        .remove(0);

        card.keep_soonest(3);

        assert_eq!(card.sessions.len(), 2, "both dates should stay");
        assert_eq!(card.further, 0, "there is nothing left over");
    }

    #[test]
    fn spells_out_the_dates_it_is_not_listing() {
        let mut card = group_by_service_and_theme(vec![offered("a", "aperos", "theme-1")]).remove(0);

        assert_eq!(card.further_label(), None, "nothing to announce");

        card.further = 1;
        assert_eq!(card.further_label().as_deref(), Some("+ 1 autre date"));

        card.further = 4;
        assert_eq!(card.further_label().as_deref(), Some("+ 4 autres dates"));
    }
}
