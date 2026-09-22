//! Types shared by the server and the browser.
//!
//! Nothing here depends on Mongo: dates and prices arrive already formatted, so
//! the WASM bundle carries no date, timezone or BSON code. The documents actually
//! stored live in [`crate::db`].

pub mod booking;
pub mod service;
pub mod session;
pub mod theme;

pub use booking::{
    BookingContact, BookingProblem, BookingRequest, BookingView, MAX_PERSONS_PER_BOOKING,
    is_french_phone, phone_key,
};
pub use service::{
    SERVICE_ICONS, ServiceView, icon_label, is_valid_slug, section_prefix, section_title, slugify,
};
pub use session::{BookingOffer, HOME_SESSIONS, SessionView};
pub use theme::{AffectedSession, MAX_PHOTO_BYTES, MAX_PHOTO_LABEL, ThemeView};
