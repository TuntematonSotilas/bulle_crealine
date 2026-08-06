//! The `bookings` collection: the reservations visitors made.

use std::collections::HashMap;

use bson::oid::ObjectId;
use bson::{DateTime, doc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::{DbError, bookings};
use crate::models::{BookingContact, ServiceType};

/// A booking document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookingDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The session booked. Kept even if that session is later deleted.
    pub session_id: ObjectId,
    /// Copied from the session so a booking stays readable on its own.
    pub service_type: ServiceType,
    pub name: String,
    /// Lowercased; half of the unique index that rules out duplicates.
    pub email: String,
    pub phone: String,
    pub persons: u32,
    /// Note left by the visitor.
    pub comment: String,
    /// Note kept by the admin; the only field the admin area edits.
    pub admin_comment: String,
    pub created_at: DateTime,
}

impl BookingDoc {
    /// Reduces the booking to the contact details shown in the admin warning.
    pub fn to_contact(&self) -> BookingContact {
        BookingContact {
            name: self.name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            persons: self.persons,
        }
    }
}

/// How many people are booked on one session.
pub async fn booked_persons(session_id: ObjectId) -> Result<u32, DbError> {
    Ok(booked_persons_by_session(vec![session_id])
        .await?
        .get(&session_id)
        .copied()
        .unwrap_or(0))
}

/// How many people are booked on each of the given sessions.
///
/// Summed by the server in one round trip, so listing sessions does not fire one
/// query per row.
pub async fn booked_persons_by_session(
    session_ids: Vec<ObjectId>,
) -> Result<HashMap<ObjectId, u32>, DbError> {
    if session_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let pipeline = vec![
        doc! { "$match": { "session_id": { "$in": session_ids } } },
        doc! { "$group": { "_id": "$session_id", "persons": { "$sum": "$persons" } } },
    ];

    let groups: Vec<bson::Document> = bookings()?
        .aggregate(pipeline)
        .await?
        .try_collect()
        .await?;

    let mut totals = HashMap::new();
    for group in groups {
        let Ok(session_id) = group.get_object_id("_id") else {
            continue;
        };

        // `$sum` hands back an Int32 or an Int64 depending on what it added up.
        let persons = group
            .get_i32("persons")
            .map(i64::from)
            .or_else(|_| group.get_i64("persons"))
            .unwrap_or(0);

        totals.insert(session_id, persons.clamp(0, i64::from(u32::MAX)) as u32);
    }

    Ok(totals)
}

/// Stores a booking.
///
/// Returns [`DbError::Duplicate`] when this address already booked this session,
/// which the unique index decides rather than a prior read.
pub async fn insert(booking: &BookingDoc) -> Result<ObjectId, DbError> {
    let inserted = bookings()?.insert_one(booking).await?;

    inserted
        .inserted_id
        .as_object_id()
        .ok_or(DbError::MalformedId)
}

/// Bookings on one session, oldest first.
pub async fn list_for_session(session_id: ObjectId) -> Result<Vec<BookingDoc>, DbError> {
    let found = bookings()?
        .find(doc! { "session_id": session_id })
        .sort(doc! { "created_at": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// Every booking, newest first, for the admin listing.
pub async fn list_all() -> Result<Vec<BookingDoc>, DbError> {
    let found = bookings()?
        .find(doc! {})
        .sort(doc! { "created_at": -1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// Replaces the admin's note on a booking.
pub async fn set_admin_comment(id: ObjectId, comment: &str) -> Result<(), DbError> {
    bookings()?
        .update_one(
            doc! { "_id": id },
            doc! { "$set": { "admin_comment": comment } },
        )
        .await?;

    Ok(())
}
