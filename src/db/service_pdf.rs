//! The `service_pdfs` collection: the PDF a workshop offers from the catalogue.
//!
//! # Why a collection of its own
//!
//! The bytes live in Mongo for the reason given in [`crate::db::theme`]: the
//! container is rebuilt from its image on every deploy and carries no volume. They
//! are not a field on the workshop because [`crate::db::service::list_all`] reads
//! whole documents, and every page draws the menu from it: each visit would drag
//! every PDF along.
//!
//! # One per workshop, never rewritten
//!
//! A unique index on the slug keeps it to one. Replacing a PDF deletes the old
//! document and inserts a new one, so an id always addresses the same bytes -- which
//! is what lets [`ServicePdfSummaryDoc::pdf_url`] carry no cache-busting stamp under
//! the route's year-long `immutable` header.

use bson::oid::ObjectId;
use bson::spec::BinarySubtype;
use bson::{Binary, DateTime, doc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::{DbError, service_pdfs};

/// One workshop's PDF, bytes included.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServicePdfDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Slug of the workshop it belongs to. `service::update` never rewrites a slug,
    /// so this cannot go stale.
    pub service_slug: String,
    pub pdf: Binary,
    pub uploaded_at: DateTime,
}

/// A PDF document without its bytes, which is what everything but the media route
/// reads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServicePdfSummaryDoc {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub service_slug: String,
    pub uploaded_at: DateTime,
}

impl ServicePdfSummaryDoc {
    /// Where [`crate::media::service_pdf`] serves this file.
    ///
    /// No stamp: a replacement is a new document under a new id, so the bytes behind
    /// this URL never change. A test pins the shape.
    pub fn pdf_url(&self) -> String {
        format!("/media/service-pdf/{}", self.id.to_hex())
    }
}

/// Wraps raw bytes for storage.
fn to_binary(bytes: Vec<u8>) -> Binary {
    Binary { subtype: BinarySubtype::Generic, bytes }
}

/// Every workshop's PDF, without the bytes. One read for the whole catalogue.
pub async fn list_all() -> Result<Vec<ServicePdfSummaryDoc>, DbError> {
    let found = summaries()?
        .find(doc! {})
        .projection(doc! { "pdf": 0 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// One workshop's PDF, without the bytes.
pub async fn find_for_service(
    service_slug: &str,
) -> Result<Option<ServicePdfSummaryDoc>, DbError> {
    Ok(summaries()?
        .find_one(doc! { "service_slug": service_slug })
        .projection(doc! { "pdf": 0 })
        .await?)
}

/// The bytes of one PDF, and the slug of the workshop it belongs to -- which names
/// the file when the visitor saves it.
pub async fn pdf(id: ObjectId) -> Result<Option<(Vec<u8>, String)>, DbError> {
    let found = service_pdfs()?.find_one(doc! { "_id": id }).await?;

    Ok(found.map(|found| (found.pdf.bytes, found.service_slug)))
}

/// Gives a workshop this PDF, dropping the one it had.
///
/// Delete then insert rather than an update, so the replacement gets an id of its
/// own (see the module notes). No transaction: a failure between the two leaves the
/// workshop with no PDF, which the admin sees and can fix by sending it again.
pub async fn replace_for_service(service_slug: &str, pdf: Vec<u8>) -> Result<(), DbError> {
    delete_for_service(service_slug).await?;

    let document = ServicePdfDoc {
        id: None,
        service_slug: service_slug.to_owned(),
        pdf: to_binary(pdf),
        uploaded_at: DateTime::now(),
    };

    service_pdfs()?.insert_one(&document).await?;
    Ok(())
}

/// Drops a workshop's PDF, if it has one. Also what deleting the workshop does.
pub async fn delete_for_service(service_slug: &str) -> Result<(), DbError> {
    service_pdfs()?
        .delete_many(doc! { "service_slug": service_slug })
        .await?;
    Ok(())
}

/// The collection seen as summaries, so a projection can drop the bytes.
fn summaries() -> Result<mongodb::Collection<ServicePdfSummaryDoc>, DbError> {
    Ok(service_pdfs()?.clone_with_type::<ServicePdfSummaryDoc>())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The route serves these with a year-long `immutable` header, which is only
    /// truthful while the bytes behind an id cannot change.
    #[test]
    fn the_url_carries_no_cache_busting_stamp() {
        let summary = ServicePdfSummaryDoc {
            id: ObjectId::parse_str("651d1f0a0000000000000001").expect("a valid id"),
            service_slug: "en-institution".to_owned(),
            uploaded_at: DateTime::now(),
        };

        let url = summary.pdf_url();

        assert_eq!(url, "/media/service-pdf/651d1f0a0000000000000001");
        assert!(!url.contains('?'), "a stamp would mean the bytes can change: {url}");
    }

    /// The bytes are served back verbatim, so they must survive BSON untouched.
    #[test]
    fn a_pdf_survives_the_trip_through_bson() {
        let pdf = b"%PDF-1.7\n\x00\xFF%%EOF".to_vec();

        let document = ServicePdfDoc {
            id: None,
            service_slug: "en-institution".to_owned(),
            pdf: to_binary(pdf.clone()),
            uploaded_at: DateTime::now(),
        };

        let stored = bson::to_document(&document).expect("a PDF should serialize");
        let read: ServicePdfDoc = bson::from_document(stored).expect("and read back");

        assert_eq!(read.pdf.bytes, pdf, "the bytes changed on the way");
    }
}
