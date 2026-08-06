//! Conversions between BSON dates and the strings the UI uses.
//!
//! # Timezone
//!
//! Session dates are wall-clock times in France, stored verbatim: the admin types
//! `2026-07-05T14:00` and that is exactly what lands in Mongo, tagged UTC even
//! though it means 14:00 in Paris. Display therefore always shows what was typed,
//! with no timezone database involved.
//!
//! The one place this shows through is the "upcoming sessions" filter, which
//! compares against a real UTC clock. A stored value reads 1 h (winter) or 2 h
//! (summer) ahead of the instant it denotes, so a session drops off the public
//! list that long after it actually started — which is the forgiving direction.

use bson::DateTime;

const MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

const WEEKDAYS: [&str; 7] = [
    "lundi",
    "mardi",
    "mercredi",
    "jeudi",
    "vendredi",
    "samedi",
    "dimanche",
];

/// Reads the value of an `<input type="datetime-local">`, e.g. `2026-07-05T14:00`.
///
/// The instant is taken as a French wall-clock time and stored as-is; see the
/// module docs.
pub fn parse_input(value: &str) -> Option<DateTime> {
    let value = value.trim();

    // Browsers append seconds as soon as the field carries a `step`, so accept
    // both `…T14:00` and `…T14:00:30`.
    let rfc3339 = match value.len() {
        16 => format!("{value}:00Z"),
        19 => format!("{value}Z"),
        _ => return None,
    };

    DateTime::parse_rfc3339_str(rfc3339).ok()
}

/// Renders a date back into the `<input type="datetime-local">` format.
pub fn to_input(date: DateTime) -> String {
    let at = date.to_time_0_3();

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}",
        at.year(),
        u8::from(at.month()),
        at.day(),
        at.hour(),
        at.minute(),
    )
}

/// Renders a session date in French, e.g. `dimanche 5 juillet 2026 à 14h00`.
pub fn to_label(date: DateTime) -> String {
    let at = date.to_time_0_3();
    let weekday = WEEKDAYS[at.weekday().number_days_from_monday() as usize];
    let month = MONTHS[u8::from(at.month()) as usize - 1];

    format!(
        "{weekday} {} {month} {} à {:02}h{:02}",
        at.day(),
        at.year(),
        at.hour(),
        at.minute(),
    )
}

/// Renders a timestamp compactly, e.g. `05/07/2026 à 14h00`.
///
/// Used for bookkeeping dates, where the weekday adds nothing.
pub fn to_short_label(date: DateTime) -> String {
    let at = date.to_time_0_3();

    format!(
        "{:02}/{:02}/{:04} à {:02}h{:02}",
        at.day(),
        u8::from(at.month()),
        at.year(),
        at.hour(),
        at.minute(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5 July 2026 is a Sunday.
    const SUNDAY_AFTERNOON: &str = "2026-07-05T14:00";

    #[test]
    fn parses_an_input_value() {
        let parsed = parse_input(SUNDAY_AFTERNOON).expect("a well-formed input value");

        assert_eq!(to_input(parsed), SUNDAY_AFTERNOON);
    }

    #[test]
    fn parses_an_input_value_carrying_seconds() {
        let parsed = parse_input("2026-07-05T14:00:30").expect("seconds are allowed");

        assert_eq!(to_input(parsed), SUNDAY_AFTERNOON);
    }

    #[test]
    fn tolerates_surrounding_whitespace() {
        assert!(parse_input("  2026-07-05T14:00 ").is_some());
    }

    #[test]
    fn rejects_a_malformed_input_value() {
        for value in [
            "",
            "2026-07-05",
            "05/07/2026T14:00",
            "2026-13-05T14:00",
            "2026-07-32T14:00",
            "2026-07-05T25:00",
            "not-a-date-at-all",
        ] {
            assert!(
                parse_input(value).is_none(),
                "{value:?} should have been rejected"
            );
        }
    }

    /// RFC 3339 allows a space where `datetime-local` writes a `T`. Browsers only
    /// ever send the `T`, but accepting both costs nothing and yields the same
    /// instant, so the leniency is left in rather than guarded against.
    #[test]
    fn also_accepts_a_space_before_the_time() {
        let parsed = parse_input("2026-07-05 14:00").expect("a space separator is tolerated");

        assert_eq!(to_input(parsed), SUNDAY_AFTERNOON);
    }

    #[test]
    fn writes_a_french_label() {
        let date = parse_input(SUNDAY_AFTERNOON).expect("a well-formed input value");

        assert_eq!(to_label(date), "dimanche 5 juillet 2026 à 14h00");
    }

    #[test]
    fn names_every_weekday_and_month() {
        // 5 January 2026 is a Monday, so a week from there walks every weekday.
        let expected_weekdays = [
            "lundi",
            "mardi",
            "mercredi",
            "jeudi",
            "vendredi",
            "samedi",
            "dimanche",
        ];
        for (offset, weekday) in expected_weekdays.iter().enumerate() {
            let day = 5 + offset;
            let date = parse_input(&format!("2026-01-{day:02}T09:00")).expect("a valid date");
            assert!(
                to_label(date).starts_with(weekday),
                "{} should start with {weekday}",
                to_label(date)
            );
        }

        for (index, month) in MONTHS.iter().enumerate() {
            let date = parse_input(&format!("2026-{:02}-15T09:00", index + 1))
                .expect("the 15th exists in every month");
            assert!(
                to_label(date).contains(month),
                "{} should contain {month}",
                to_label(date)
            );
        }
    }

    #[test]
    fn writes_a_short_label() {
        let date = parse_input(SUNDAY_AFTERNOON).expect("a well-formed input value");

        assert_eq!(to_short_label(date), "05/07/2026 à 14h00");
    }

    #[test]
    fn pads_single_digit_times() {
        let date = parse_input("2026-07-05T09:05").expect("a well-formed input value");

        assert_eq!(to_label(date), "dimanche 5 juillet 2026 à 09h05");
        assert_eq!(to_short_label(date), "05/07/2026 à 09h05");
    }
}
