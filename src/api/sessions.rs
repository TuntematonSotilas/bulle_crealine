//! Server functions over the sessions on offer.

use leptos::prelude::*;

use crate::models::{BookingContact, SessionView};

/// Upcoming sessions of one kind, for the public booking page.
///
/// Open to everyone: it exposes nothing a visitor cannot already read on the
/// matching service page.
#[server]
pub async fn upcoming_sessions(service: String) -> Result<Vec<SessionView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::session;
    use crate::models::ServiceType;

    let service_type = ServiceType::from_slug(service.trim())
        .ok_or_else(|| ServerFnError::new("Ce type d'atelier n'existe pas."))?;

    let sessions = session::list_upcoming(service_type)
        .await
        .map_err(|error| log_failure("listing upcoming sessions", error))?;

    with_booked_persons(sessions).await
}

/// Every session, for the admin listing.
#[server]
pub async fn all_sessions() -> Result<Vec<SessionView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::session;

    require_admin()?;

    let sessions = session::list_all()
        .await
        .map_err(|error| log_failure("listing sessions", error))?;

    with_booked_persons(sessions).await
}

/// Creates a session, or updates the one `id` points at when it is not empty.
#[server]
pub async fn save_session(
    id: String,
    service: String,
    date: String,
    theme: String,
    price: f64,
    max_persons: u32,
) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::session::{self, SessionDoc};
    use crate::db::{booking, datetime};
    use crate::models::ServiceType;

    require_admin()?;

    let service_type = ServiceType::from_slug(service.trim())
        .ok_or_else(|| ServerFnError::new("Choisissez un type d'atelier."))?;
    let date = datetime::parse_input(&date)
        .ok_or_else(|| ServerFnError::new("Cette date n'est pas valide."))?;
    let theme = theme.trim().to_owned();

    if theme.is_empty() {
        return Err(ServerFnError::new("Indiquez un thème."));
    }
    if !price.is_finite() || price < 0.0 {
        return Err(ServerFnError::new("Ce prix n'est pas valide."));
    }
    if max_persons == 0 {
        return Err(ServerFnError::new(
            "Une séance doit accepter au moins une personne.",
        ));
    }

    let document = SessionDoc {
        id: None,
        service_type,
        date,
        theme,
        price,
        max_persons,
    };

    let id = id.trim();
    if id.is_empty() {
        session::insert(&document)
            .await
            .map_err(|error| log_failure("inserting a session", error))?;

        return Ok(());
    }

    let session_id = session::parse_id(id).map_err(|_| ServerFnError::new("Séance inconnue."))?;

    // Shrinking a session below what is already booked would leave it silently
    // overbooked, so refuse rather than let the two figures contradict each other.
    let booked = booking::booked_persons(session_id)
        .await
        .map_err(|error| log_failure("counting bookings before an update", error))?;
    if max_persons < booked {
        return Err(ServerFnError::new(format!(
            "{booked} personnes sont déjà inscrites : la capacité ne peut pas descendre en dessous."
        )));
    }

    session::update(session_id, &document)
        .await
        .map_err(|error| log_failure("updating a session", error))?;

    Ok(())
}

/// Drops a session.
///
/// Bookings on it are kept on purpose, so the admin can still reach the people
/// who had signed up.
#[server]
pub async fn delete_session(id: String) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::session;

    require_admin()?;

    let session_id = session::parse_id(&id).map_err(|_| ServerFnError::new("Séance inconnue."))?;

    session::delete(session_id)
        .await
        .map_err(|error| log_failure("deleting a session", error))?;

    Ok(())
}

/// Who booked one session, to warn the admin before a risky change.
#[server]
pub async fn session_contacts(id: String) -> Result<Vec<BookingContact>, ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{booking, session};

    require_admin()?;

    let session_id = session::parse_id(&id).map_err(|_| ServerFnError::new("Séance inconnue."))?;

    let bookings = booking::list_for_session(session_id)
        .await
        .map_err(|error| log_failure("listing the bookings of a session", error))?;

    Ok(bookings
        .iter()
        .map(|booking| booking.to_contact())
        .collect())
}

/// Resolves how many people each session already carries, in one round trip.
#[cfg(feature = "ssr")]
async fn with_booked_persons(
    sessions: Vec<crate::db::session::SessionDoc>,
) -> Result<Vec<SessionView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::booking;

    let ids = sessions.iter().filter_map(|session| session.id).collect();
    let booked = booking::booked_persons_by_session(ids)
        .await
        .map_err(|error| log_failure("summing bookings per session", error))?;

    Ok(sessions
        .iter()
        .map(|session| {
            let taken = session
                .id
                .and_then(|id| booked.get(&id).copied())
                .unwrap_or(0);

            session.to_view(taken)
        })
        .collect())
}
