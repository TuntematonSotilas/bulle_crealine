use serde::{Deserialize, Serialize};

use crate::models::ServiceView;

/// How many upcoming sessions the home page lists, every kind of workshop mixed
/// together and soonest first.
///
/// The home page is a shop window, not the calendar: past the next handful of
/// dates a visitor is better served by a workshop's own page, which the "voir +"
/// link on each card points at. The cap is global rather than per kind, so a
/// heavily scheduled workshop can take the whole list -- which is the point, those
/// really are the next dates on offer.
pub const HOME_SESSIONS: usize = 6;

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

    /// Whether the session can still take anyone.
    pub fn is_full(&self) -> bool {
        self.remaining_places() == 0
    }

    /// Sentence describing what is left, for display next to the session.
    pub fn availability_label(&self) -> String {
        match self.remaining_places() {
            0 => "Complet".to_owned(),
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
}
