//! Serving the bytes stored in Mongo.
//!
//! Theme photos live in their document rather than on disk (see
//! [`crate::db::theme`]), so they need a route of their own; an `<img src>` cannot
//! call a server function.

use actix_web::http::header;
use actix_web::{HttpResponse, web};

/// How much of a request body the server accepts.
///
/// Actix defaults to 256 kB, which no photo would fit under. The real cap is
/// enforced while reading the upload — this only has to be wide enough not to cut
/// a legitimate one short, hence the room left for the other fields and the
/// multipart boundaries.
pub const MAX_BODY_BYTES: usize = crate::models::MAX_PHOTO_BYTES + 64 * 1024;

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
