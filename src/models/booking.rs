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
    MissingEmail,
    MalformedEmail,
    MissingPhone,
    NoPersons,
    TooManyPersons,
}

impl BookingProblem {
    /// Sentence shown to the visitor.
    pub const fn message(self) -> &'static str {
        match self {
            Self::NoSession => "Choisissez une séance.",
            Self::MissingName => "Indiquez votre nom.",
            Self::MissingEmail => "Indiquez votre adresse e-mail.",
            Self::MalformedEmail => "Cette adresse e-mail semble incorrecte.",
            Self::MissingPhone => "Indiquez votre numéro de téléphone.",
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
        if self.email.is_empty() {
            return Err(BookingProblem::MissingEmail);
        }
        if !looks_like_an_email(&self.email) {
            return Err(BookingProblem::MalformedEmail);
        }
        if self.phone.is_empty() {
            return Err(BookingProblem::MissingPhone);
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
            ("email", BookingRequest { email: String::new(), ..request() }, BookingProblem::MissingEmail),
            ("phone", BookingRequest { phone: String::new(), ..request() }, BookingProblem::MissingPhone),
        ];

        for (field, form, expected) in cases {
            assert_eq!(form.validate(), Err(expected), "missing {field} slipped through");
        }
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
