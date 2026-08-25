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

use futures_util::TryStreamExt;
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

/// Unique index over the live bookings of one session, keyed by phone number.
const ACTIVE_BOOKING_INDEX: &str = "session_id_1_phone_key_1_active";

/// Index names from when the address carried a booking's identity. Dropped at
/// startup: with the address now optional they would file every blank-address
/// booking under the empty string and reject the second one.
///
/// The first is what Mongo auto-named the original; the second its successor
/// scoped to the live bookings.
const STALE_EMAIL_INDEXES: [&str; 2] = ["session_id_1_email_1", "session_id_1_email_1_active"];

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

    // Brings the documents written before soft deletion existed in line with the
    // field, so the partial filter below covers every booking. It has to test
    // `false` rather than `$ne: true`, because a partialFilterExpression accepts
    // no `$ne` — a document with no field would otherwise fall outside the index
    // and escape the constraint entirely.
    bookings
        .update_many(
            doc! { "is_deleted": { "$exists": false } },
            doc! { "$set": { "is_deleted": false, "deletion_comment": "" } },
        )
        .await?;

    // Same for the phone key: documents written while the address carried the
    // identity have none, and the index would file them all under the empty
    // string. Derived in Rust rather than in an aggregation pipeline, which would
    // take a chain of `$replaceAll` per separator, and only over the documents
    // still missing it — a no-op query from the second boot on.
    let unkeyed: Vec<BookingDoc> = bookings
        .find(doc! { "phone_key": { "$in": [null, ""] } })
        .await?
        .try_collect()
        .await?;

    for booking in unkeyed {
        let Some(id) = booking.id else { continue };

        bookings
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "phone_key": crate::models::phone_key(&booking.phone) } },
            )
            .await?;
    }

    // Turns "this number already booked this session" into a guarantee rather
    // than a check two simultaneous requests could both pass. Keyed on the phone
    // number because that is the field every booking now has.
    //
    // Scoped to the live bookings, so deleting one frees the number to book that
    // session again.
    let by_phone = IndexModel::builder()
        .keys(doc! { "session_id": 1, "phone_key": 1 })
        .options(
            IndexOptions::builder()
                .name(ACTIVE_BOOKING_INDEX.to_owned())
                .unique(true)
                .partial_filter_expression(doc! { "is_deleted": false })
                .build(),
        )
        .build();

    // Existing data can already hold two live bookings that key alike, and Mongo
    // then refuses to build the index. Reported rather than propagated: a failed
    // `init` takes the whole database offline, which is a far worse outcome than
    // running without this one guarantee.
    if let Err(error) = bookings.create_index(by_phone).await {
        if is_duplicate_key(&error) {
            eprintln!(
                "the unique booking index was not built: two live bookings on one session \
                 already share a phone number. Resolve them, then restart. ({error})"
            );
        } else {
            return Err(error.into());
        }
    }

    // Dropped whatever happened above: with the address optional these would
    // reject the second visitor who leaves it blank, both keying as the empty
    // string. Only when present, which makes this a no-op from the second boot on.
    for stale in STALE_EMAIL_INDEXES {
        if bookings.list_index_names().await?.iter().any(|name| name == stale) {
            bookings.drop_index(stale).await?;
        }
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
