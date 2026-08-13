use serde::{Deserialize, Serialize};

/// Largest photo accepted, in bytes.
///
/// Well under Mongo's 16 MB document ceiling. It lives here rather than next to the
/// document it bounds so that the form and the server quote the same figure: the db
/// layer is server-only, and the page could not read a constant from it.
pub const MAX_PHOTO_BYTES: usize = 5 * 1024 * 1024;

/// [`MAX_PHOTO_BYTES`] as it is worded to the admin.
pub const MAX_PHOTO_LABEL: &str = "5 Mo";

/// A workshop theme as the browser sees it.
///
/// Carries the photo as a URL, never as bytes: the image itself is served by
/// `GET /media/theme/{id}`, so listing the themes stays cheap and the WASM bundle
/// never handles binary data.
///
/// Not to be confused with the light/dark theme, which is
/// [`crate::components::hooks::use_theme_mode`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeView {
    /// Hex form of the Mongo `ObjectId`.
    pub id: String,
    pub name: String,
    /// Where to fetch the photo, already carrying the cache-busting stamp.
    pub photo_url: String,
}

/// One line of "here is what changing this theme affects".
///
/// Deliberately minimal, like [`crate::models::BookingContact`]: enough to
/// recognise a session, nothing more.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AffectedSession {
    /// Human label, e.g. `"dimanche 5 juillet 2026 à 14h00"`.
    pub date_label: String,
    /// Label of the kind of workshop, e.g. `"Apéros créatifs (adultes)"`.
    pub service_label: String,
}
