//! Types shared by the server and the browser.
//!
//! Nothing here depends on Mongo: dates and prices arrive already formatted, so
//! the WASM bundle carries no date, timezone or BSON code. The documents actually
//! stored live in [`crate::db`].

pub mod booking;
pub mod service_type;
pub mod session;
pub mod theme;

pub use booking::{
    BookingContact, BookingProblem, BookingRequest, BookingView, MAX_PERSONS_PER_BOOKING,
    phone_key,
};
pub use service_type::ServiceType;
pub use session::{HOME_SESSIONS, SessionView};
pub use theme::{AffectedSession, MAX_PHOTO_BYTES, MAX_PHOTO_LABEL, ThemeView};
