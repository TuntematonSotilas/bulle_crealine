//! Server functions backing the booking pages and the admin area.
//!
//! Every function reserved for the admin opens with
//! [`require_admin`](crate::auth::require_admin): these endpoints live under
//! `/api`, which the page guard does not cover.

pub mod bookings;
pub mod sessions;

/// Shown when Mongo is unreachable or misconfigured.
///
/// Storage failures are never described to the visitor: the details go to the
/// server log, and the page says only that the operation could not go through.
#[cfg(feature = "ssr")]
pub const STORAGE_UNAVAILABLE: &str =
    "Les données ne sont pas accessibles pour le moment. Réessayez dans un instant.";

/// Logs a storage failure and turns it into a message fit for a visitor.
#[cfg(feature = "ssr")]
pub fn log_failure(context: &str, error: crate::db::DbError) -> leptos::prelude::ServerFnError {
    eprintln!("{context} failed: {error}");

    leptos::prelude::ServerFnError::new(STORAGE_UNAVAILABLE)
}
