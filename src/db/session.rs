//! The `sessions` collection: the workshop dates on offer.

use bson::oid::ObjectId;
use bson::{DateTime, doc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::service::ServiceDoc;
use crate::db::theme::ThemeSummaryDoc;
use crate::db::{DbError, datetime, sessions};
use crate::models::SessionView;

/// A session document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Slug of the kind of workshop, in [`crate::db::service`].
    ///
    /// A plain string rather than a foreign key, and deliberately still named
    /// `service_type`: that is the field, and the value, every session written
    /// while workshops were an enum already carries. Workshops became editable
    /// without a single stored document changing.
    pub service_type: String,
    /// French wall-clock time, stored verbatim; see [`crate::db::datetime`].
    pub date: DateTime,
    /// The theme this session is about, in [`crate::db::theme`].
    pub theme_id: ObjectId,
    pub price: f64,
    /// How many people the session can take in total.
    pub max_persons: u32,
}

impl SessionDoc {
    /// Turns the document into what the browser gets, given how many people are
    /// already booked on it and the theme it points at.
    ///
    /// `theme` and `service` are passed in rather than looked up here so that
    /// listing sessions resolves each of them in one round trip. Both fall back to
    /// a marker when missing: deleting either is refused while a session uses it,
    /// so that only happens if the collection was edited by hand.
    pub fn to_view(
        &self,
        booked_persons: u32,
        theme: Option<&ThemeSummaryDoc>,
        service: Option<&ServiceDoc>,
    ) -> SessionView {
        SessionView {
            id: self.id.map(|id| id.to_hex()).unwrap_or_default(),
            service_slug: self.service_type.clone(),
            service_label: service
                .map(|found| found.label.clone())
                .unwrap_or_else(|| "Atelier supprimé".to_owned()),
            service_description: service
                .map(|found| found.description.clone())
                .unwrap_or_default(),
            // Home rather than a link into nothing: a card whose workshop is gone
            // still shows a date worth reading, and "voir +" has to lead somewhere.
            service_path: service.map(ServiceDoc::to_view).map_or_else(
                || "/".to_owned(),
                |view| view.page_path(),
            ),
            date_label: datetime::to_label(self.date),
            date_input: datetime::to_input(self.date),
            theme_id: self.theme_id.to_hex(),
            theme_name: theme
                .map(|found| found.name.clone())
                .unwrap_or_else(|| "Thème supprimé".to_owned()),
            photo_url: theme.map(ThemeSummaryDoc::photo_url).unwrap_or_default(),
            price: self.price,
            max_persons: self.max_persons,
            booked_persons,
            // One, not zero, when the workshop is gone: the minimum is read as
            // `remaining < min_persons`, so a zero would not relax the rule but
            // switch off every guard standing on it.
            min_persons: service.map_or(1, |found| found.min_persons),
        }
    }
}

/// Reads a hex id coming from a form or a URL.
pub fn parse_id(id: &str) -> Result<ObjectId, DbError> {
    ObjectId::parse_str(id.trim()).map_err(|_| DbError::MalformedId)
}

/// Sessions of one kind that have not started yet, soonest first.
pub async fn list_upcoming(service_slug: &str) -> Result<Vec<SessionDoc>, DbError> {
    let filter = doc! {
        "service_type": service_slug,
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

/// The next `limit` upcoming sessions, whatever their kind, soonest first.
///
/// One query across all four kinds rather than four, and capped in the database
/// rather than in Rust: the `date` index that `crate::db::ensure_indexes` creates
/// at startup already holds them in this order, so Mongo walks it and stops at
/// `limit` instead of reading a calendar that only grows.
pub async fn list_next_upcoming(limit: i64) -> Result<Vec<SessionDoc>, DbError> {
    let found = sessions()?
        .find(doc! { "date": { "$gte": DateTime::now() } })
        .sort(doc! { "date": 1 })
        .limit(limit)
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// How many upcoming sessions each workshop-and-theme pair has, keyed by slug and
/// hex theme id.
///
/// Serves the "+ N autres dates" line on the home page, which the window that page
/// reads cannot answer on its own: dates past its horizon are real but unseen, and
/// a card would say "+ 1 autre date" for a workshop running weekly all year.
///
/// One round trip whatever the calendar holds. The `date` index serves the match,
/// and the grouping leaves one row per pair -- a few dozen, not one per session.
pub async fn count_upcoming_by_service_and_theme()
-> Result<std::collections::HashMap<(String, String), usize>, DbError> {
    let pipeline = vec![
        doc! { "$match": { "date": { "$gte": DateTime::now() } } },
        doc! { "$group": {
            "_id": { "service_type": "$service_type", "theme_id": "$theme_id" },
            "dates": { "$sum": 1 },
        } },
    ];

    let groups: Vec<bson::Document> = sessions()?.aggregate(pipeline).await?.try_collect().await?;

    let mut totals = std::collections::HashMap::new();
    for group in groups {
        let Ok(key) = group.get_document("_id") else {
            continue;
        };
        let (Ok(slug), Ok(theme_id)) = (key.get_str("service_type"), key.get_object_id("theme_id"))
        else {
            continue;
        };

        // The key is built the way a `SessionView` carries it, so the caller can
        // look a group up without converting anything.
        totals.insert(
            (slug.to_owned(), theme_id.to_hex()),
            group.get_i32("dates").unwrap_or_default().max(0) as usize,
        );
    }

    Ok(totals)
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

/// How many sessions point at a theme.
///
/// Gates deleting that theme: a session whose theme is gone has nothing to show.
pub async fn count_for_theme(theme_id: ObjectId) -> Result<u64, DbError> {
    Ok(sessions()?.count_documents(doc! { "theme_id": theme_id }).await?)
}

/// How many sessions are filed under a kind of workshop.
///
/// Gates deleting that workshop: the sessions would keep a slug nothing answers
/// to, and both the admin table and the booking page would lose their label.
pub async fn count_for_service(service_slug: &str) -> Result<u64, DbError> {
    Ok(sessions()?
        .count_documents(doc! { "service_type": service_slug })
        .await?)
}

/// The sessions using a theme, soonest first, to spell out what changing it affects.
pub async fn list_for_theme(theme_id: ObjectId) -> Result<Vec<SessionDoc>, DbError> {
    let found = sessions()?
        .find(doc! { "theme_id": theme_id })
        .sort(doc! { "date": 1 })
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
            "service_type": &session.service_type,
            "date": session.date,
            "theme_id": session.theme_id,
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
