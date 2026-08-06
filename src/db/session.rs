//! The `sessions` collection: the workshop dates on offer.

use bson::oid::ObjectId;
use bson::{DateTime, doc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::{DbError, datetime, sessions};
use crate::models::{ServiceType, SessionView};

/// A session document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub service_type: ServiceType,
    /// French wall-clock time, stored verbatim; see [`crate::db::datetime`].
    pub date: DateTime,
    pub theme: String,
    pub price: f64,
    /// How many people the session can take in total.
    pub max_persons: u32,
}

impl SessionDoc {
    /// Turns the document into what the browser gets, given how many people are
    /// already booked on it.
    pub fn to_view(&self, booked_persons: u32) -> SessionView {
        SessionView {
            id: self.id.map(|id| id.to_hex()).unwrap_or_default(),
            service_type: self.service_type,
            date_label: datetime::to_label(self.date),
            date_input: datetime::to_input(self.date),
            theme: self.theme.clone(),
            price: self.price,
            max_persons: self.max_persons,
            booked_persons,
        }
    }
}

/// Reads a hex id coming from a form or a URL.
pub fn parse_id(id: &str) -> Result<ObjectId, DbError> {
    ObjectId::parse_str(id.trim()).map_err(|_| DbError::MalformedId)
}

/// Sessions of one kind that have not started yet, soonest first.
pub async fn list_upcoming(service_type: ServiceType) -> Result<Vec<SessionDoc>, DbError> {
    let filter = doc! {
        "service_type": service_type.slug(),
        "date": { "$gte": DateTime::now() },
    };

    let found = sessions()?
        .find(filter)
        .sort(doc! { "date": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// Every session, soonest first, for the admin listing.
pub async fn list_all() -> Result<Vec<SessionDoc>, DbError> {
    let found = sessions()?
        .find(doc! {})
        .sort(doc! { "date": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// One session by id.
pub async fn find(id: ObjectId) -> Result<Option<SessionDoc>, DbError> {
    Ok(sessions()?.find_one(doc! { "_id": id }).await?)
}

/// Sessions matching the given ids, for labelling a list of bookings.
pub async fn find_many(ids: Vec<ObjectId>) -> Result<Vec<SessionDoc>, DbError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let found = sessions()?
        .find(doc! { "_id": { "$in": ids } })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// Stores a new session and returns its id.
pub async fn insert(session: &SessionDoc) -> Result<ObjectId, DbError> {
    let inserted = sessions()?.insert_one(session).await?;

    inserted
        .inserted_id
        .as_object_id()
        .ok_or(DbError::MalformedId)
}

/// Overwrites the mutable fields of an existing session.
pub async fn update(id: ObjectId, session: &SessionDoc) -> Result<(), DbError> {
    let update = doc! {
        "$set": {
            "service_type": session.service_type.slug(),
            "date": session.date,
            "theme": &session.theme,
            "price": session.price,
            "max_persons": session.max_persons,
        }
    };

    sessions()?.update_one(doc! { "_id": id }, update).await?;

    Ok(())
}

/// Drops a session.
///
/// Bookings pointing at it are deliberately kept: they record who signed up, and
/// the admin still needs to reach those people. The admin listing shows them with
/// a "deleted session" marker.
pub async fn delete(id: ObjectId) -> Result<(), DbError> {
    sessions()?.delete_one(doc! { "_id": id }).await?;

    Ok(())
}
