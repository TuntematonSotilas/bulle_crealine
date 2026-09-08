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

/// The next few sessions on offer, every kind of workshop mixed together and
/// soonest first, for the home page.
///
/// Open to everyone, like [`upcoming_sessions`].
#[server]
pub async fn next_sessions() -> Result<Vec<SessionView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::session;
    use crate::models::HOME_SESSIONS;

    let upcoming = session::list_next_upcoming(HOME_SESSIONS as i64)
        .await
        .map_err(|error| log_failure("listing the next upcoming sessions", error))?;

    // Soonest first out of the query, and `with_booked_persons` keeps that order,
    // so the page can render the list as it comes.
    with_booked_persons(upcoming).await
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
    theme_id: String,
    price: f64,
    max_persons: u32,
) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::session::{self, SessionDoc};
    use crate::db::{booking, datetime, theme};
    use crate::models::ServiceType;

    require_admin()?;

    let service_type = ServiceType::from_slug(service.trim())
        .ok_or_else(|| ServerFnError::new("Choisissez un type d'atelier."))?;
    let date = datetime::parse_input(&date)
        .ok_or_else(|| ServerFnError::new("Cette date n'est pas valide."))?;

    // Checked against the collection, not just parsed: a stale dropdown could still
    // post a theme that has since been deleted, and the listing would then show the
    // session with no theme at all.
    let theme_id = session::parse_id(theme_id.trim())
        .map_err(|_| ServerFnError::new("Choisissez un thème."))?;
    if theme::find(theme_id)
        .await
        .map_err(|error| log_failure("checking the theme of a session", error))?
        .is_none()
    {
        return Err(ServerFnError::new("Ce thème n'existe plus."));
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
        theme_id,
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

/// Resolves how many people each session carries and what its theme is called,
/// in one round trip each rather than one per session.
#[cfg(feature = "ssr")]
async fn with_booked_persons(
    sessions: Vec<crate::db::session::SessionDoc>,
) -> Result<Vec<SessionView>, ServerFnError> {
    use std::collections::HashMap;

    use crate::api::log_failure;
    use crate::db::{booking, theme};

    let ids = sessions.iter().filter_map(|session| session.id).collect();
    let booked = booking::booked_persons_by_session(ids)
        .await
        .map_err(|error| log_failure("summing bookings per session", error))?;

    let mut theme_ids: Vec<_> = sessions.iter().map(|session| session.theme_id).collect();
    theme_ids.sort_unstable();
    theme_ids.dedup();

    // Kept whole rather than reduced to names: a session view carries the theme's
    // photo URL as well, and that is built from the stamp on the document.
    let themes: HashMap<_, _> = theme::find_many(theme_ids)
        .await
        .map_err(|error| log_failure("resolving the themes of a list of sessions", error))?
        .into_iter()
        .map(|found| (found.id, found))
        .collect();

    Ok(sessions
        .iter()
        .map(|session| {
            let taken = session
                .id
                .and_then(|id| booked.get(&id).copied())
                .unwrap_or(0);

            session.to_view(taken, themes.get(&session.theme_id))
        })
        .collect())
}
