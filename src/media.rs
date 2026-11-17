//! Serving the bytes stored in Mongo.
//!
//! Theme photos live in their document rather than on disk (see
//! [`crate::db::theme`]), so they need a route of their own; an `<img src>` cannot
//! call a server function. Workshop photos live in a collection of their own (see
//! [`crate::db::service_photo`]) and are served the same way, as are the workshop
//! PDFs (see [`crate::db::service_pdf`]).

use actix_web::http::header;
use actix_web::{HttpResponse, web};

/// How much of a request body the server accepts.
///
/// Actix defaults to 256 kB, which no photo would fit under. The real caps are
/// enforced while reading each upload — this only has to be wide enough not to cut
/// the largest legitimate one short, hence the room left for the other fields and
/// the multipart boundaries.
pub const MAX_BODY_BYTES: usize = largest_upload() + 64 * 1024;

/// The larger of the two files an admin can send: a photo or a PDF.
const fn largest_upload() -> usize {
    let photo = crate::models::MAX_PHOTO_BYTES;
    let pdf = crate::models::MAX_PDF_BYTES;

    if photo > pdf { photo } else { pdf }
}

/// `GET /media/theme/{id}` — one theme's photo.
///
/// Public on purpose: these images are meant to be shown to visitors, and an
/// `<img>` on an admin page would carry the session cookie anyway.
pub async fn theme_photo(id: web::Path<String>) -> HttpResponse {
    use crate::db::{session, theme};

    let Ok(theme_id) = session::parse_id(&id) else {
        return HttpResponse::NotFound().finish();
    };

    match theme::photo(theme_id).await {
        Ok(Some((bytes, content_type))) => HttpResponse::Ok()
            .content_type(content_type)
            // The URL carries a stamp that changes whenever the photo does, so a
            // cached copy can never be the stale one.
            .insert_header((header::CACHE_CONTROL, "public, max-age=31536000, immutable"))
            .body(bytes),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(error) => {
            eprintln!("serving a theme photo failed: {error}");
            HttpResponse::ServiceUnavailable().finish()
        }
    }
}

/// `GET /media/service-photo/{id}` — one photo of a workshop run for a structure.
///
/// Public for the same reason as [`theme_photo`]: these images are meant to be
/// shown to visitors.
pub async fn service_photo(id: web::Path<String>) -> HttpResponse {
    use crate::db::{service_photo, session};

    let Ok(photo_id) = session::parse_id(&id) else {
        return HttpResponse::NotFound().finish();
    };

    match service_photo::photo(photo_id).await {
        Ok(Some((bytes, content_type))) => HttpResponse::Ok()
            .content_type(content_type)
            // Truthful without a stamp here, unlike a theme's: the document behind
            // this id is never rewritten, so the bytes cannot change under it.
            .insert_header((header::CACHE_CONTROL, "public, max-age=31536000, immutable"))
            .body(bytes),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(error) => {
            eprintln!("serving a workshop photo failed: {error}");
            HttpResponse::ServiceUnavailable().finish()
        }
    }
}

/// `GET /media/service-pdf/{id}` — the PDF a workshop offers from the catalogue.
///
/// `inline`, so the browser opens it in its own reader rather than saving it; the
/// file name is what that reader offers when the visitor does save it. Cached like
/// a workshop photo, for the same reason: the document behind an id is never
/// rewritten.
pub async fn service_pdf(id: web::Path<String>) -> HttpResponse {
    use crate::db::{service_pdf, session};

    let Ok(pdf_id) = session::parse_id(&id) else {
        return HttpResponse::NotFound().finish();
    };

    match service_pdf::pdf(pdf_id).await {
        Ok(Some((bytes, slug))) => HttpResponse::Ok()
            .content_type("application/pdf")
            // The slug is safe inside the quotes: `is_valid_slug` allows only
            // lowercase letters, digits and hyphens.
            .insert_header((
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{slug}.pdf\""),
            ))
            .insert_header((header::CACHE_CONTROL, "public, max-age=31536000, immutable"))
            .body(bytes),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(error) => {
            eprintln!("serving a workshop PDF failed: {error}");
            HttpResponse::ServiceUnavailable().finish()
        }
    }
}
