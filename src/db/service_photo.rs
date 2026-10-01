//! The `service_photos` collection: what a workshop run for a structure looks like.
//!
//! # Why a collection of its own
//!
//! The bytes live in Mongo for the reason given in [`crate::db::theme`]: the
//! container is rebuilt from its image on every deploy and carries no volume. But
//! five photos of [`crate::models::MAX_PHOTO_BYTES`] would be 25 MB against a 16 MB
//! document ceiling, so they cannot be a field on the workshop. One document per
//! photo, filed under the workshop's slug.
//!
//! # Write once
//!
//! A photo document is inserted and deleted, never rewritten -- the admin adds and
//! removes, there is no "replace this one". That is what lets [`photo_url`] carry no
//! cache-busting stamp where a theme's does.
//!
//! No field here is `#[serde(default)]`, unlike every other document in this layer:
//! nothing predates this collection, which is born with its fields. Anything added
//! later will need one.

use bson::oid::ObjectId;
use bson::spec::BinarySubtype;
use bson::{Binary, DateTime, doc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::{DbError, service_photos};

/// One photo of a workshop, bytes included.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServicePhotoDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Slug of the workshop it belongs to, which is how a session and a theme refer
    /// to one too. `service::update` never rewrites a slug, so this cannot go stale.
    pub service_slug: String,
    pub photo: Binary,
    /// Sniffed from the bytes by [`crate::db::theme::detect_content_type`], never
    /// taken from the browser, which is free to claim anything.
    pub content_type: String,
    /// When it was added, which is the order the gallery shows them in.
    pub uploaded_at: DateTime,
}

/// A photo document without its bytes.
///
/// Everything but the image route reads this: it is what keeps a gallery of five
/// from transferring 25 MB to draw five thumbnails.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServicePhotoSummaryDoc {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub service_slug: String,
    pub uploaded_at: DateTime,
}

impl ServicePhotoSummaryDoc {
    /// Where [`crate::media::service_photo`] serves this image.
    ///
    /// No stamp, unlike a theme's: this document is never rewritten, and an
    /// `ObjectId` is never reused, so the URL addresses bytes that cannot change.
    /// That is what makes the route's year-long `immutable` header truthful.
    ///
    /// The day someone adds a "replace this photo", that header turns into a
    /// year-long poisoning and this has to grow a stamp. A test pins the shape.
    pub fn photo_url(&self) -> String {
        format!("/media/service-photo/{}", self.id.to_hex())
    }
}

/// Wraps raw bytes for storage.
fn to_binary(bytes: Vec<u8>) -> Binary {
    Binary { subtype: BinarySubtype::Generic, bytes }
}

/// One workshop's photos, oldest first, without the bytes.
///
/// `_id` breaks the tie: two photos uploaded in the same millisecond would
/// otherwise swap places between two loads of the same page.
pub async fn list_for_service(
    service_slug: &str,
) -> Result<Vec<ServicePhotoSummaryDoc>, DbError> {
    let found = summaries()?
        .find(doc! { "service_slug": service_slug })
        .projection(doc! { "photo": 0 })
        .sort(doc! { "uploaded_at": 1, "_id": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// How many photos a workshop carries, to cap the gallery and to explain a refusal.
pub async fn count_for_service(service_slug: &str) -> Result<u64, DbError> {
    Ok(service_photos()?
        .count_documents(doc! { "service_slug": service_slug })
        .await?)
}

/// The bytes of one photo, and how to serve them.
pub async fn photo(id: ObjectId) -> Result<Option<(Vec<u8>, String)>, DbError> {
    let found = service_photos()?.find_one(doc! { "_id": id }).await?;

    Ok(found.map(|found| (found.photo.bytes, found.content_type)))
}

pub async fn insert(
    service_slug: &str,
    photo: Vec<u8>,
    content_type: &str,
) -> Result<ObjectId, DbError> {
    let document = ServicePhotoDoc {
        id: None,
        service_slug: service_slug.to_owned(),
        photo: to_binary(photo),
        content_type: content_type.to_owned(),
        uploaded_at: DateTime::now(),
    };

    let inserted = service_photos()?.insert_one(&document).await?;
    inserted.inserted_id.as_object_id().ok_or(DbError::MalformedId)
}

pub async fn delete(id: ObjectId) -> Result<(), DbError> {
    service_photos()?.delete_one(doc! { "_id": id }).await?;
    Ok(())
}

/// Drops every photo of a workshop, which is what deleting the workshop does.
pub async fn delete_for_service(service_slug: &str) -> Result<(), DbError> {
    service_photos()?
        .delete_many(doc! { "service_slug": service_slug })
        .await?;
    Ok(())
}

/// The collection seen as summaries, so a projection can drop the bytes.
fn summaries() -> Result<mongodb::Collection<ServicePhotoSummaryDoc>, DbError> {
    Ok(service_photos()?.clone_with_type::<ServicePhotoSummaryDoc>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> ServicePhotoSummaryDoc {
        ServicePhotoSummaryDoc {
            id: ObjectId::parse_str("651d1f0a0000000000000001").expect("a valid id"),
            service_slug: "en-institution".to_owned(),
            uploaded_at: DateTime::now(),
        }
    }

    /// The route serves these with a year-long `immutable` header, which is only
    /// truthful while the bytes behind an id cannot change. Should a "replace this
    /// photo" ever land, this test is what should stop it going out unnoticed.
    #[test]
    fn the_url_carries_no_cache_busting_stamp() {
        let url = summary().photo_url();

        assert_eq!(url, "/media/service-photo/651d1f0a0000000000000001");
        assert!(!url.contains('?'), "a stamp would mean the bytes can change: {url}");
    }

    /// The bytes make the round trip untouched: they are served back verbatim, and
    /// a `Binary` that re-read as anything else would serve a broken image.
    #[test]
    fn a_photo_survives_the_trip_through_bson() {
        let png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0xFF];

        let document = ServicePhotoDoc {
            id: None,
            service_slug: "en-institution".to_owned(),
            photo: to_binary(png.clone()),
            content_type: "image/png".to_owned(),
            uploaded_at: DateTime::now(),
        };

        let stored = bson::to_document(&document).expect("a photo should serialize");
        let read: ServicePhotoDoc =
            bson::from_document(stored).expect("and read back");

        assert_eq!(read.photo.bytes, png, "the bytes changed on the way");
        assert_eq!(read.content_type, "image/png");
        assert_eq!(read.service_slug, "en-institution");
    }
}
