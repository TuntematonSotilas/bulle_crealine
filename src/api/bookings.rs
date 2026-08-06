//! Server functions over the bookings.

use leptos::prelude::*;

use crate::models::BookingView;

/// Records a booking and returns the label of the session it lands on.
///
/// Open to everyone, so every check runs here rather than in the browser: the
/// form's `required` attributes are a convenience, not a guarantee.
#[server]
pub async fn create_booking(
    session_id: String,
    name: String,
    email: String,
    phone: String,
    persons: u32,
    comment: String,
) -> Result<String, ServerFnError> {
    use bson::DateTime;

    use crate::api::log_failure;
    use crate::db::booking::{self, BookingDoc};
    use crate::db::{DbError, datetime, session};
    use crate::models::BookingRequest;

    let request = BookingRequest {
        session_id,
        name,
        email,
        phone,
        persons,
        comment,
    }
    .normalized();

    request
        .validate()
        .map_err(|problem| ServerFnError::new(problem.message()))?;

    let session_id = session::parse_id(&request.session_id)
        .map_err(|_| ServerFnError::new("Cette séance n'existe pas."))?;

    let session = session::find(session_id)
        .await
        .map_err(|error| log_failure("loading a session for a booking", error))?
        .ok_or_else(|| ServerFnError::new("Cette séance n'est plus proposée."))?;

    // Capacity is counted in people, not in bookings: three parties of two fill
    // six of the eight places.
    let booked = booking::booked_persons(session_id)
        .await
        .map_err(|error| log_failure("counting bookings before a booking", error))?;
    let remaining = session.max_persons.saturating_sub(booked);

    if remaining == 0 {
        return Err(ServerFnError::new("Cette séance est complète."));
    }
    if request.persons > remaining {
        return Err(ServerFnError::new(match remaining {
            1 => "Il ne reste qu'une place sur cette séance.".to_owned(),
            _ => format!("Il ne reste que {remaining} places sur cette séance."),
        }));
    }

    let document = BookingDoc {
        id: None,
        session_id,
        service_type: session.service_type,
        name: request.name,
        email: request.email,
        phone: request.phone,
        persons: request.persons,
        comment: request.comment,
        admin_comment: String::new(),
        created_at: DateTime::now(),
    };

    // The duplicate is caught by the unique index rather than by a prior read, so
    // two identical forms sent at once cannot both get through.
    match booking::insert(&document).await {
        Ok(_) => Ok(datetime::to_label(session.date)),
        Err(DbError::Duplicate) => Err(ServerFnError::new(
            "Une réservation existe déjà pour cette adresse sur cette séance.",
        )),
        Err(error) => Err(log_failure("inserting a booking", error)),
    }
}

/// Every booking, newest first, for the admin listing.
#[server]
pub async fn all_bookings() -> Result<Vec<BookingView>, ServerFnError> {
    use std::collections::HashMap;

    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{booking, datetime, session};

    require_admin()?;

    let bookings = booking::list_all()
        .await
        .map_err(|error| log_failure("listing bookings", error))?;

    // Sessions are fetched in one go and joined here: Mongo has no join, and one
    // query per row would not scale past a handful of bookings.
    let session_ids = bookings.iter().map(|booking| booking.session_id).collect();
    let sessions = session::find_many(session_ids)
        .await
        .map_err(|error| log_failure("loading the sessions of the bookings", error))?;
    let sessions: HashMap<_, _> = sessions
        .into_iter()
        .filter_map(|session| session.id.map(|id| (id, session)))
        .collect();

    Ok(bookings
        .into_iter()
        .map(|booking| {
            let session = sessions.get(&booking.session_id);

            BookingView {
                id: booking.id.map(|id| id.to_hex()).unwrap_or_default(),
                session_id: booking.session_id.to_hex(),
                service_type: booking.service_type,
                // A booking outlives the session it points at, so the label has to
                // cope with that session being gone.
                session_date_label: session
                    .map(|session| datetime::to_label(session.date))
                    .unwrap_or_else(|| "Séance supprimée".to_owned()),
                session_theme: session
                    .map(|session| session.theme.clone())
                    .unwrap_or_default(),
                name: booking.name,
                email: booking.email,
                phone: booking.phone,
                persons: booking.persons,
                comment: booking.comment,
                admin_comment: booking.admin_comment,
                created_label: datetime::to_short_label(booking.created_at),
            }
        })
        .collect())
}

/// Replaces the admin's note on a booking.
#[server]
pub async fn save_admin_comment(id: String, comment: String) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{booking, session};

    require_admin()?;

    let booking_id =
        session::parse_id(&id).map_err(|_| ServerFnError::new("Réservation inconnue."))?;

    booking::set_admin_comment(booking_id, comment.trim())
        .await
        .map_err(|error| log_failure("saving an admin comment", error))?;

    Ok(())
}
