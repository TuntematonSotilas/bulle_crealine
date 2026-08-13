//! The `themes` collection: what a session is about, and its photo.
//!
//! # Why the photo lives here
//!
//! The bytes sit in the document rather than on disk because the container is
//! rebuilt from its image on every deploy and carries no volume: a file written
//! next to the served assets would not survive. Mongo is the only storage that
//! outlives a deploy.
//!
//! The cost is that a naive `find` would drag every photo into memory, so reads
//! come in two shapes: [`ThemeSummaryDoc`] for anything that lists or joins, and
//! [`photo`] for the one route that actually serves an image.

use bson::oid::ObjectId;
use bson::spec::BinarySubtype;
use bson::{Binary, DateTime, doc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::{DbError, themes};
use crate::models::ThemeView;

/// A theme document, photo included.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub photo: Binary,
    /// Sniffed from the bytes by [`detect_content_type`], never taken from the
    /// browser, which is free to claim anything.
    pub content_type: String,
    /// When the photo last changed, which is what makes its URL unique.
    pub photo_updated_at: DateTime,
}

/// A theme document without its photo.
///
/// Everything but the image route reads this: it is what keeps the admin listing
/// from transferring megabytes to draw a table of names.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeSummaryDoc {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub photo_updated_at: DateTime,
}

impl ThemeSummaryDoc {
    pub fn to_view(&self) -> ThemeView {
        ThemeView {
            id: self.id.to_hex(),
            name: self.name.clone(),
            // The stamp lets the response be cached forever while a replaced photo
            // still shows up immediately, under its new URL.
            photo_url: format!(
                "/media/theme/{}?v={}",
                self.id.to_hex(),
                self.photo_updated_at.timestamp_millis()
            ),
        }
    }
}

/// Recognises the image formats we accept, by their leading bytes.
///
/// Sniffing rather than trusting the upload's declared type: the content type is
/// echoed back by the image route, and a caller that could choose it freely could
/// have a browser treat their bytes as something else entirely.
pub fn detect_content_type(bytes: &[u8]) -> Option<&'static str> {
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
    const GIF87: &[u8] = b"GIF87a";
    const GIF89: &[u8] = b"GIF89a";

    if bytes.starts_with(PNG) {
        return Some("image/png");
    }
    // JPEG: SOI marker, then any of the several possible segment markers.
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    // WebP is a RIFF container whose form type sits at offset 8.
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if bytes.starts_with(GIF87) || bytes.starts_with(GIF89) {
        return Some("image/gif");
    }

    None
}

/// Wraps raw bytes for storage.
fn to_binary(bytes: Vec<u8>) -> Binary {
    Binary { subtype: BinarySubtype::Generic, bytes }
}

/// Every theme, by name, without the photos.
pub async fn list_all() -> Result<Vec<ThemeSummaryDoc>, DbError> {
    let found = summaries()?
        .find(doc! {})
        .projection(doc! { "photo": 0 })
        .sort(doc! { "name": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// One theme, without its photo.
pub async fn find(id: ObjectId) -> Result<Option<ThemeSummaryDoc>, DbError> {
    Ok(summaries()?
        .find_one(doc! { "_id": id })
        .projection(doc! { "photo": 0 })
        .await?)
}

/// The named themes, without their photos, for resolving names in bulk.
pub async fn find_many(ids: Vec<ObjectId>) -> Result<Vec<ThemeSummaryDoc>, DbError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let found = summaries()?
        .find(doc! { "_id": { "$in": ids } })
        .projection(doc! { "photo": 0 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// The bytes of one theme's photo, and how to serve them.
pub async fn photo(id: ObjectId) -> Result<Option<(Vec<u8>, String)>, DbError> {
    let found = themes()?.find_one(doc! { "_id": id }).await?;

    Ok(found.map(|theme| (theme.photo.bytes, theme.content_type)))
}

pub async fn insert(name: &str, photo: Vec<u8>, content_type: &str) -> Result<ObjectId, DbError> {
    let document = ThemeDoc {
        id: None,
        name: name.to_owned(),
        photo: to_binary(photo),
        content_type: content_type.to_owned(),
        photo_updated_at: DateTime::now(),
    };

    let inserted = themes()?.insert_one(&document).await?;
    inserted.inserted_id.as_object_id().ok_or(DbError::MalformedId)
}

/// Renames a theme, and replaces its photo when a new one was uploaded.
///
/// `photo` being `None` means the form was submitted without choosing a file,
/// which keeps the current image rather than clearing it.
pub async fn update(
    id: ObjectId,
    name: &str,
    photo: Option<(Vec<u8>, String)>,
) -> Result<(), DbError> {
    let mut fields = doc! { "name": name };

    if let Some((bytes, content_type)) = photo {
        fields.insert("photo", to_binary(bytes));
        fields.insert("content_type", content_type);
        // Only stamped when the image changes: the stamp is what busts the cache of
        // a URL, and a rename has no reason to invalidate it.
        fields.insert("photo_updated_at", DateTime::now());
    }

    themes()?.update_one(doc! { "_id": id }, doc! { "$set": fields }).await?;
    Ok(())
}

pub async fn delete(id: ObjectId) -> Result<(), DbError> {
    themes()?.delete_one(doc! { "_id": id }).await?;
    Ok(())
}

/// The collection seen as summaries, so a projection can drop the photo.
fn summaries() -> Result<mongodb::Collection<ThemeSummaryDoc>, DbError> {
    Ok(themes()?.clone_with_type::<ThemeSummaryDoc>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_accepted_formats() {
        assert_eq!(detect_content_type(b"\x89PNG\r\n\x1a\nrest"), Some("image/png"));
        assert_eq!(detect_content_type(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00]), Some("image/jpeg"));
        assert_eq!(detect_content_type(b"RIFF\x24\x00\x00\x00WEBPVP8 "), Some("image/webp"));
        assert_eq!(detect_content_type(b"GIF89a\x01\x00"), Some("image/gif"));
    }

    /// Anything unrecognised has to be turned down: the sniffed type is echoed back
    /// as the `Content-Type` of the image route.
    #[test]
    fn turns_down_anything_else() {
        assert_eq!(detect_content_type(b""), None, "an empty upload");
        assert_eq!(detect_content_type(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"), None, "svg");
        assert_eq!(detect_content_type(b"%PDF-1.7"), None, "a pdf");
        assert_eq!(detect_content_type(b"just some text"), None, "plain text");
        assert_eq!(detect_content_type(&[0x00; 32]), None, "zeroed bytes");
    }

    /// A prefix long enough to look like the start of a signature must not pass:
    /// `starts_with` on a truncated file would otherwise read past nothing.
    #[test]
    fn turns_down_truncated_signatures() {
        assert_eq!(detect_content_type(b"\x89PNG"), None, "half a png signature");
        assert_eq!(detect_content_type(&[0xFF, 0xD8]), None, "half a jpeg signature");
        assert_eq!(detect_content_type(b"RIFF\x24\x00\x00\x00WEB"), None, "half a webp header");
        assert_eq!(detect_content_type(b"RIFF\x24\x00\x00\x00AVI "), None, "a riff that is not webp");
        assert_eq!(detect_content_type(b"GIF8"), None, "half a gif signature");
    }
}
