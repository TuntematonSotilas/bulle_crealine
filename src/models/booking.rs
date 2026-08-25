use serde::{Deserialize, Serialize};

use crate::models::ServiceType;

/// Largest party a single booking may declare.
///
/// Not a business rule so much as a guard: a form is free to post any number, and
/// a typo of `100` should not silently fill a workshop.
pub const MAX_PERSONS_PER_BOOKING: u32 = 20;

/// What a visitor fills in on the booking page, once trimmed and checked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingRequest {
    pub session_id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub persons: u32,
    /// Free-text note from the visitor; may be empty.
    pub comment: String,
}

/// Why a booking form was turned down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookingProblem {
    NoSession,
    MissingName,
    /// Only ever raised for a non-empty address: giving none is allowed.
    MalformedEmail,
    MissingPhone,
    /// Digits are what [`phone_key`] keeps, and a number without any of them
    /// would key as the empty string and collide with every other such number.
    MalformedPhone,
    NoPersons,
    TooManyPersons,
}

impl BookingProblem {
    /// Sentence shown to the visitor.
    pub const fn message(self) -> &'static str {
        match self {
            Self::NoSession => "Choisissez une séance.",
            Self::MissingName => "Indiquez votre nom.",
            Self::MalformedEmail => "Cette adresse e-mail semble incorrecte.",
            Self::MissingPhone => "Indiquez votre numéro de téléphone.",
            Self::MalformedPhone => "Ce numéro de téléphone semble incorrect.",
            Self::NoPersons => "Il faut au moins une personne.",
            Self::TooManyPersons => "Contactez-nous directement pour un groupe de cette taille.",
        }
    }
}

impl BookingRequest {
    /// Trims every field and lowercases the address, so that the same person
    /// typing `Alice@Example.com ` twice trips the duplicate check.
    pub fn normalized(&self) -> Self {
        Self {
            session_id: self.session_id.trim().to_owned(),
            name: self.name.trim().to_owned(),
            email: self.email.trim().to_lowercase(),
            phone: self.phone.trim().to_owned(),
            persons: self.persons,
            comment: self.comment.trim().to_owned(),
        }
    }

    /// Checks a normalized request.
    ///
    /// The email test stays deliberately loose — a shape check, not an attempt to
    /// decide whether the address exists.
    pub fn validate(&self) -> Result<(), BookingProblem> {
        if self.session_id.is_empty() {
            return Err(BookingProblem::NoSession);
        }
        if self.name.is_empty() {
            return Err(BookingProblem::MissingName);
        }
        // The address is optional: the phone number is what we reach people on,
        // and what tells two bookings apart. Its shape is still checked, but only
        // once one was actually given.
        if !self.email.is_empty() && !looks_like_an_email(&self.email) {
            return Err(BookingProblem::MalformedEmail);
        }
        if self.phone.is_empty() {
            return Err(BookingProblem::MissingPhone);
        }
        if phone_key(&self.phone).is_empty() {
            return Err(BookingProblem::MalformedPhone);
        }
        if self.persons == 0 {
            return Err(BookingProblem::NoPersons);
        }
        if self.persons > MAX_PERSONS_PER_BOOKING {
            return Err(BookingProblem::TooManyPersons);
        }

        Ok(())
    }
}

/// Canonical form of a phone number, and a booking's identity on a session.
///
/// Digits only, so `"06 12 34 56 78"`, `"06.12.34.56.78"` and `"0612345678"` all
/// land on one key. Without this the unique index would compare what was typed,
/// and the same number spelled two ways would slip past it.
///
/// Deliberately short of full E.164 normalization: `"+33 6 12 34 56 78"` keys as
/// `"33612345678"` and so does not meet its national spelling. Reconciling the
/// two needs a country to assume, which this form never asks for.
pub fn phone_key(phone: &str) -> String {
    phone.chars().filter(char::is_ascii_digit).collect()
}

/// Exactly one `@`, something on each side, and a dot in the domain.
fn looks_like_an_email(candidate: &str) -> bool {
    let Some((local, domain)) = candidate.split_once('@') else {
        return false;
    };

    !local.is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
        && domain.contains('.')
        && !candidate.contains(char::is_whitespace)
}

/// A booking as listed in the admin area.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingView {
    /// Hex form of the Mongo `ObjectId`.
    pub id: String,
    pub session_id: String,
    pub service_type: ServiceType,
    /// Label of the booked session, or a stand-in when that session was deleted.
    pub session_date_label: String,
    pub session_theme: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub persons: u32,
    /// Note left by the visitor; read-only for the admin.
    pub comment: String,
    /// Note the admin keeps on this booking; the only editable field.
    pub admin_comment: String,
    /// When the booking came in, already formatted.
    pub created_label: String,
    /// Whether the admin removed this booking; decides which of the two admin
    /// tables it lands in.
    pub is_deleted: bool,
    /// Why the admin removed it. Empty unless [`Self::is_deleted`].
    pub deletion_comment: String,
}

/// Who to warn before a session is changed or dropped.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingContact {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub persons: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> BookingRequest {
        BookingRequest {
            session_id: "651d1f0a0000000000000000".to_owned(),
            name: "Alice Martin".to_owned(),
            email: "alice@example.com".to_owned(),
            phone: "06 12 34 56 78".to_owned(),
            persons: 2,
            comment: String::new(),
        }
    }

    #[test]
    fn accepts_a_complete_request() {
        assert_eq!(request().validate(), Ok(()));
    }

    #[test]
    fn accepts_an_empty_comment_but_not_an_empty_phone() {
        let mut form = request();
        form.comment = String::new();
        assert_eq!(form.validate(), Ok(()));

        form.phone = String::new();
        assert_eq!(form.validate(), Err(BookingProblem::MissingPhone));
    }

    #[test]
    fn rejects_missing_fields() {
        let cases = [
            ("session", BookingRequest { session_id: String::new(), ..request() }, BookingProblem::NoSession),
            ("name", BookingRequest { name: String::new(), ..request() }, BookingProblem::MissingName),
            ("phone", BookingRequest { phone: String::new(), ..request() }, BookingProblem::MissingPhone),
        ];

        for (field, form, expected) in cases {
            assert_eq!(form.validate(), Err(expected), "missing {field} slipped through");
        }
    }

    /// The phone number carries the requirement the address used to.
    #[test]
    fn accepts_a_booking_without_an_email() {
        let form = BookingRequest { email: String::new(), ..request() };

        assert_eq!(form.validate(), Ok(()));
    }

    /// Optional is not unchecked: a typed address still has to look like one, or
    /// it would be stored unusable.
    #[test]
    fn still_rejects_a_malformed_email_when_one_is_given() {
        let form = BookingRequest { email: "not-an-address".to_owned(), ..request() };

        assert_eq!(form.validate(), Err(BookingProblem::MalformedEmail));
    }

    /// Whitespace is not an address: normalization has to turn it into the empty,
    /// accepted case rather than leave it to trip the shape check.
    #[test]
    fn a_blank_email_normalizes_to_no_email() {
        let form = BookingRequest { email: "   ".to_owned(), ..request() };
        let clean = form.normalized();

        assert_eq!(clean.email, "");
        assert_eq!(clean.validate(), Ok(()));
    }

    /// One number spelled several ways has to reach one key, or the unique index
    /// would let the same person book a session twice.
    #[test]
    fn one_number_spelled_differently_gives_one_key() {
        let canonical = phone_key("0612345678");

        for spelling in ["06 12 34 56 78", "06.12.34.56.78", "06-12-34-56-78", " 0612345678 "] {
            assert_eq!(phone_key(spelling), canonical, "{spelling:?} should match");
        }
    }

    /// A number without a digit would key as the empty string, and every such
    /// booking would then look like a repeat of the first.
    #[test]
    fn rejects_a_phone_number_carrying_no_digit() {
        let form = BookingRequest { phone: "à rappeler".to_owned(), ..request() };

        assert_eq!(form.validate(), Err(BookingProblem::MalformedPhone));
        assert!(phone_key("à rappeler").is_empty());
    }

    #[test]
    fn rejects_a_party_size_out_of_range() {
        let none = BookingRequest { persons: 0, ..request() };
        assert_eq!(none.validate(), Err(BookingProblem::NoPersons));

        let crowd = BookingRequest { persons: MAX_PERSONS_PER_BOOKING + 1, ..request() };
        assert_eq!(crowd.validate(), Err(BookingProblem::TooManyPersons));

        let limit = BookingRequest { persons: MAX_PERSONS_PER_BOOKING, ..request() };
        assert_eq!(limit.validate(), Ok(()));
    }

    #[test]
    fn rejects_a_malformed_email() {
        for email in [
            "no-at-sign",
            "@example.com",
            "alice@",
            "alice@example",
            "alice@.com",
            "alice@example.",
            "alice@@example.com",
            "ali ce@example.com",
        ] {
            let form = BookingRequest { email: email.to_owned(), ..request() };
            assert_eq!(
                form.validate(),
                Err(BookingProblem::MalformedEmail),
                "{email:?} should have been rejected"
            );
        }
    }

    #[test]
    fn normalization_trims_and_lowercases_the_email() {
        let messy = BookingRequest {
            session_id: "  651d1f0a0000000000000000 ".to_owned(),
            name: "  Alice Martin  ".to_owned(),
            email: "  Alice@Example.COM ".to_owned(),
            phone: " 06 12 34 56 78 ".to_owned(),
            persons: 2,
            comment: "  myComment  ".to_owned(),
        };

        let clean = messy.normalized();

        assert_eq!(clean.session_id, "651d1f0a0000000000000000");
        assert_eq!(clean.name, "Alice Martin");
        assert_eq!(clean.email, "alice@example.com");
        assert_eq!(clean.phone, "06 12 34 56 78");
        assert_eq!(clean.comment, "myComment");
    }

    /// A blank field is only whitespace away from an empty one, and the browser
    /// happily posts spaces past a `required` attribute.
    #[test]
    fn normalization_turns_blank_fields_into_rejected_ones() {
        let blank = BookingRequest { name: "   ".to_owned(), ..request() };

        assert_eq!(blank.normalized().validate(), Err(BookingProblem::MissingName));
    }
}
