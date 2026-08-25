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
    /// Whether the admin removed this booking. Deleting is logical: the document
    /// stays, but it drops out of the capacity count and of the contacts to warn.
    ///
    /// Defaulted because documents written before soft deletion existed carry no
    /// such field, and a missing bool would otherwise fail to deserialize.
    #[serde(default)]
    pub is_deleted: bool,
    /// Why the admin removed it. Required at deletion, empty while active.
    #[serde(default)]
    pub deletion_comment: String,
}

/// Matches the bookings that still count.
///
/// `$ne` rather than `false` so the documents predating the field are treated as
/// active, exactly as [`BookingDoc::is_deleted`]'s default does on read.
fn active() -> bson::Document {
    doc! { "is_deleted": { "$ne": true } }
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

    // Deleted bookings free their seats, so they are filtered out here rather
    // than subtracted afterwards.
    let pipeline = vec![
        doc! { "$match": {
            "session_id": { "$in": session_ids },
            "is_deleted": { "$ne": true },
        } },
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

/// Live bookings on one session, oldest first.
///
/// Feeds the warning shown before a session is changed or dropped, so a deleted
/// booking is left out: there is nobody left to warn.
pub async fn list_for_session(session_id: ObjectId) -> Result<Vec<BookingDoc>, DbError> {
    let mut filter = active();
    filter.insert("session_id", session_id);

    let found = bookings()?
        .find(filter)
        .sort(doc! { "created_at": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// Every booking, deleted ones included, newest first.
///
/// The admin listing shows both, split into two tables, so the split happens
/// once the documents are read rather than over two queries.
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

/// Marks a booking as deleted, recording why.
///
/// Returns `false` when no booking carries that id, so the caller can tell a
/// stale form apart from a deletion that landed.
pub async fn soft_delete(id: ObjectId, reason: &str) -> Result<bool, DbError> {
    let outcome = bookings()?
        .update_one(
            doc! { "_id": id },
            doc! { "$set": { "is_deleted": true, "deletion_comment": reason } },
        )
        .await?;

    Ok(outcome.matched_count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bookings stored before soft deletion existed carry neither new field.
    /// Without the `serde` defaults every one of them would fail to read, taking
    /// the whole admin listing down with it.
    #[test]
    fn a_document_predating_soft_deletion_reads_as_active() {
        let legacy = doc! {
            "_id": ObjectId::new(),
            "session_id": ObjectId::new(),
            "service_type": "aperos-creatifs",
            "name": "Alice Martin",
            "email": "alice@example.com",
            "phone": "06 12 34 56 78",
            "persons": 2i64,
            "comment": "",
            "admin_comment": "",
            "created_at": DateTime::now(),
        };

        let booking: BookingDoc =
            bson::from_document(legacy).expect("a legacy booking should still deserialize");

        assert!(!booking.is_deleted, "a legacy booking must count as active");
        assert_eq!(booking.deletion_comment, "");
    }

    /// The filter has to treat "no field" as active too, or the documents above
    /// would drop out of the capacity count the moment the field was introduced.
    #[test]
    fn the_active_filter_admits_a_missing_flag() {
        let filter = active();
        let condition = filter
            .get_document("is_deleted")
            .expect("the filter should constrain is_deleted");

        assert_eq!(condition.get_bool("$ne"), Ok(true));
    }
}
