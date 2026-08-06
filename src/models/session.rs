use serde::{Deserialize, Serialize};

use crate::models::ServiceType;

/// A session as the browser sees it.
///
/// Dates arrive already formatted, so the WASM bundle needs neither a date nor a
/// timezone library. Capacity arrives already resolved, so a page can tell
/// whether a session is full without a second round trip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionView {
    /// Hex form of the Mongo `ObjectId`.
    pub id: String,
    pub service_type: ServiceType,
    /// Human label, e.g. `"dimanche 5 juillet 2026 à 14h00"`.
    pub date_label: String,
    /// `"2026-07-05T14:00"`, ready for an `<input type="datetime-local">`.
    pub date_input: String,
    pub theme: String,
    pub price: f64,
    /// How many people the session can take in total.
    pub max_persons: u32,
    /// How many people are already booked, summed over every booking.
    pub booked_persons: u32,
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
            service_type: ServiceType::CreatifsPourTous,
            date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme: "Sculpture".to_owned(),
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
}
