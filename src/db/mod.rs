//! MongoDB access.
//!
//! One [`Database`] handle is opened at startup and shared for the process's
//! lifetime; the driver pools connections behind it, so cloning the handle per
//! request is free.
//!
//! Like the admin credentials, the connection is optional: without
//! `MONGODB_URI` the public site still serves, and every booking or admin query
//! reports that storage is unavailable rather than bringing the process down.

pub mod booking;
pub mod datetime;
pub mod session;
pub mod theme;

use std::env;
use std::sync::OnceLock;

use mongodb::options::{IndexOptions, ServerApi, ServerApiVersion};
use mongodb::{Client, Collection, Database, IndexModel, bson::doc};

use crate::db::booking::BookingDoc;
use crate::db::session::SessionDoc;
use crate::db::theme::ThemeDoc;

/// Database holding the sessions and the bookings.
const DEFAULT_DATABASE: &str = "bulle_crealine_db";

/// Sessions on offer.
const SESSIONS: &str = "sessions";

/// Bookings made by visitors.
const BOOKINGS: &str = "bookings";

/// What the sessions are about: a name and a photo.
const THEMES: &str = "themes";

/// Unique index over the live bookings of one session.
const ACTIVE_BOOKING_INDEX: &str = "session_id_1_email_1_active";

/// The name Mongo gave the unqualified index this replaced.
const LEGACY_BOOKING_INDEX: &str = "session_id_1_email_1";

static DATABASE: OnceLock<Database> = OnceLock::new();

/// Anything that can go wrong while reaching Mongo.
#[derive(Debug)]
pub enum DbError {
    /// `MONGODB_URI` is absent or empty.
    NotConfigured,
    /// The cluster refused the connection, or a query failed.
    Mongo(mongodb::error::Error),
    /// An id coming from a form is not a valid `ObjectId`.
    MalformedId,
    /// A unique index turned the write down.
    Duplicate,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => write!(f, "MONGODB_URI is missing or empty"),
            Self::Mongo(error) => write!(f, "mongodb: {error}"),
            Self::MalformedId => write!(f, "malformed identifier"),
            Self::Duplicate => write!(f, "a unique index rejected the write"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<mongodb::error::Error> for DbError {
    fn from(error: mongodb::error::Error) -> Self {
        if is_duplicate_key(&error) {
            Self::Duplicate
        } else {
            Self::Mongo(error)
        }
    }
}

/// Whether Mongo rejected a write because a unique index already held the value.
fn is_duplicate_key(error: &mongodb::error::Error) -> bool {
    use mongodb::error::{ErrorKind, WriteFailure};

    /// Mongo's code for a duplicate key.
    const DUPLICATE_KEY: i32 = 11_000;

    match error.kind.as_ref() {
        ErrorKind::Write(WriteFailure::WriteError(write_error)) => {
            write_error.code == DUPLICATE_KEY
        }
        ErrorKind::InsertMany(insert_error) => insert_error
            .write_errors
            .as_ref()
            .is_some_and(|errors| errors.iter().any(|error| error.code == DUPLICATE_KEY)),
        _ => false,
    }
}

/// Connects to the cluster, checks it answers, and creates the indexes.
///
/// Meant to be called once at startup: a bad URI is then reported before the
/// first visitor rather than on their first booking.
pub async fn init() -> Result<&'static Database, DbError> {
    if let Some(existing) = DATABASE.get() {
        return Ok(existing);
    }

    let uri = match env::var("MONGODB_URI") {
        Ok(uri) if !uri.trim().is_empty() => uri,
        _ => return Err(DbError::NotConfigured),
    };
    let name = match env::var("MONGODB_DATABASE") {
        Ok(name) if !name.trim().is_empty() => name.trim().to_owned(),
        _ => DEFAULT_DATABASE.to_owned(),
    };

    let mut options = mongodb::options::ClientOptions::parse(uri.trim()).await?;
    // Pin the server API so a cluster upgrade cannot change how our queries behave.
    options.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

    let database = Client::with_options(options)?.database(&name);

    // `with_options` does not talk to the cluster, so without this ping a bad URI
    // or a wrong password would only surface on the first real query.
    database.run_command(doc! { "ping": 1 }).await?;

    ensure_indexes(&database).await?;

    Ok(DATABASE.get_or_init(|| database))
}

/// The handle opened at startup, or `None` when storage is unavailable.
pub fn get() -> Option<&'static Database> {
    DATABASE.get()
}

/// The sessions collection, or [`DbError::NotConfigured`].
pub fn sessions() -> Result<Collection<SessionDoc>, DbError> {
    Ok(get().ok_or(DbError::NotConfigured)?.collection(SESSIONS))
}

/// The bookings collection, or [`DbError::NotConfigured`].
pub fn bookings() -> Result<Collection<BookingDoc>, DbError> {
    Ok(get().ok_or(DbError::NotConfigured)?.collection(BOOKINGS))
}

/// The themes collection, or [`DbError::NotConfigured`].
pub fn themes() -> Result<Collection<ThemeDoc>, DbError> {
    Ok(get().ok_or(DbError::NotConfigured)?.collection(THEMES))
}

/// Creates the indexes the queries rely on.
///
/// Creating an index that already exists with the same shape is a no-op, so this
/// is safe to run on every boot.
async fn ensure_indexes(database: &Database) -> Result<(), DbError> {
    let sessions: Collection<SessionDoc> = database.collection(SESSIONS);
    let bookings: Collection<BookingDoc> = database.collection(BOOKINGS);
    let themes: Collection<ThemeDoc> = database.collection(THEMES);

    // Serves the public listing: sessions of one kind, upcoming first.
    sessions
        .create_index(
            IndexModel::builder()
                .keys(doc! { "service_type": 1, "date": 1 })
                .build(),
        )
        .await?;

    // Turns "this address already booked this session" into a guarantee rather
    // than a check that two simultaneous requests could both pass.
    //
    // Scoped to the live bookings: once a booking is deleted the address is free
    // to book that session again. The partial filter tests `false` rather than
    // `$ne: true` because a partialFilterExpression accepts no `$ne`, hence the
    // backfill just above — a legacy document with no field would otherwise fall
    // outside the index and escape the constraint entirely.
    bookings
        .update_many(
            doc! { "is_deleted": { "$exists": false } },
            doc! { "$set": { "is_deleted": false, "deletion_comment": "" } },
        )
        .await?;

    bookings
        .create_index(
            IndexModel::builder()
                .keys(doc! { "session_id": 1, "email": 1 })
                .options(
                    IndexOptions::builder()
                        .name(ACTIVE_BOOKING_INDEX.to_owned())
                        .unique(true)
                        .partial_filter_expression(doc! { "is_deleted": false })
                        .build(),
                )
                .build(),
        )
        .await?;

    // The unqualified index this replaces would still refuse a second booking
    // after the first was deleted. Dropped only once the partial one is in place,
    // so the uniqueness guarantee is never lifted, and only when actually there,
    // which makes this a no-op from the second boot on.
    if bookings
        .list_index_names()
        .await?
        .iter()
        .any(|name| name == LEGACY_BOOKING_INDEX)
    {
        bookings.drop_index(LEGACY_BOOKING_INDEX).await?;
    }

    // Serves the admin listing, newest booking first.
    bookings
        .create_index(IndexModel::builder().keys(doc! { "created_at": -1 }).build())
        .await?;

    // Two themes with the same name would be indistinguishable in the dropdown that
    // picks one for a session, so let the index refuse rather than check-then-write.
    themes
        .create_index(
            IndexModel::builder()
                .keys(doc! { "name": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    // Answers "is any session still using this theme?", which gates its deletion.
    sessions
        .create_index(IndexModel::builder().keys(doc! { "theme_id": 1 }).build())
        .await?;

    Ok(())
}
