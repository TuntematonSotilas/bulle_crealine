//! Server functions over the photos of a workshop run for a structure.
//!
//! [`service_photos`] is open to everyone, like [`crate::api::services::all_services`]:
//! the gallery is what a visitor came to see.
//!
//! [`add_service_photo`] is the second endpoint in this project that does not take
//! plain form fields -- a photo cannot travel through the urlencoded body an
//! `<ActionForm>` builds, so it reads a multipart stream. See
//! [`crate::api::themes::save_theme`], which it follows closely but for one
//! departure, marked where it matters.

use leptos::prelude::*;
// At file scope rather than inside the function: both sides of `add_service_photo`
// name these, the browser to build the body and the server to read it.
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

use crate::models::ServicePhotoView;

/// One workshop's gallery, oldest photo first.
///
/// Empty for a workshop that is not run for a structure, mirroring how
/// [`crate::api::sessions::upcoming_offer`] turns a `pro` workshop away: a gallery
/// only belongs to the pages that have one, and keeping that rule here means a page
/// cannot forget it. It is what makes a workshop moved out of the section stop
/// showing photos the admin can no longer reach.
#[server]
pub async fn service_photos(service: String) -> Result<Vec<ServicePhotoView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::{service as workshop, service_photo};
    use crate::models::MAX_SERVICE_PHOTOS;

    let Some(found) = workshop::find_by_slug(service.trim())
        .await
        .map_err(|error| log_failure("loading a workshop for its gallery", error))?
        .filter(|found| found.pro)
    else {
        return Ok(Vec::new());
    };

    let photos = service_photo::list_for_service(&found.slug)
        .await
        .map_err(|error| log_failure("listing the photos of a workshop", error))?;

    Ok(photos
        .iter()
        // Capped on the way out as well as on the way in: the ceiling is a
        // read-then-write (see `add_service_photo`), so two uploads racing each
        // other leave an unreachable sixth document rather than a six-up grid.
        .take(MAX_SERVICE_PHOTOS)
        .enumerate()
        .map(|(rank, photo)| ServicePhotoView {
            id: photo.id.to_hex(),
            url: photo.photo_url(),
            // Resolved here because the gallery component never sees the workshop.
            alt: format!("{} — photo {}", found.label, rank + 1),
        })
        .collect())
}

/// Adds one photo to a workshop.
///
/// Reads two multipart fields: `service`, the workshop's slug, and `photo`.
#[server(input = MultipartFormData)]
pub async fn add_service_photo(data: MultipartData) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::theme::detect_content_type;
    use crate::db::{service as workshop, service_photo};
    use crate::models::{MAX_PHOTO_LABEL, MAX_PHOTO_BYTES, MAX_SERVICE_PHOTOS};

    require_admin()?;

    let mut fields = data
        .into_inner()
        .ok_or_else(|| ServerFnError::new("Ce formulaire n'a pas pu être lu."))?;

    let mut service = String::new();
    let mut photo: Vec<u8> = Vec::new();

    while let Some(mut field) = fields
        .next_field()
        .await
        .map_err(|error| read_failure("reading the photo form", error))?
    {
        let field_name = field.name().unwrap_or_default().to_owned();

        match field_name.as_str() {
            // A field missing from this arm is not a compile error: it falls
            // through to the catch-all below and is silently dropped.
            "service" => {
                service = field
                    .text()
                    .await
                    .map_err(|error| read_failure("reading the workshop field", error))?;
            }
            "photo" => {
                // Read chunk by chunk so an oversized upload is turned down as soon
                // as it crosses the line, rather than after it has all been buffered.
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|error| read_failure("reading a workshop photo", error))?
                {
                    if photo.len() + chunk.len() > MAX_PHOTO_BYTES {
                        return Err(ServerFnError::new(format!(
                            "Cette photo est trop lourde : {MAX_PHOTO_LABEL} au maximum."
                        )));
                    }
                    photo.extend_from_slice(&chunk);
                }
            }
            // Ignored rather than refused: a browser may add fields of its own, and
            // turning the whole submission down over one would be unhelpful.
            _ => {}
        }
    }

    // The slug comes from a hidden field, but the form is an ordinary POST anyone
    // can forge, and a photo filed under a slug naming nothing would be invisible
    // and unreachable at once.
    let service = service.trim();
    let Some(found) = workshop::find_by_slug(service)
        .await
        .map_err(|error| log_failure("checking the workshop of a photo", error))?
        .filter(|found| found.pro)
    else {
        return Err(ServerFnError::new(
            "Les photos ne concernent que les ateliers de la rubrique « Autres Ateliers ».",
        ));
    };

    // Where this parts company with `save_theme`: there an empty part means "keep
    // the photo already stored", because an untouched file input still sends one.
    // Adding has nothing to keep, so an empty part is a submission with no file.
    if photo.is_empty() {
        return Err(ServerFnError::new("Choisissez une photo."));
    }

    let content_type = detect_content_type(&photo).ok_or_else(|| {
        ServerFnError::new("Ce fichier n'est pas une image (formats acceptés : PNG, JPEG, WebP).")
    })?;

    // Read-then-write, which this project avoids everywhere else by letting an
    // index refuse. No index expresses "at most five", so the check has to live
    // here; the gallery takes only the first five in case two uploads race.
    let held = service_photo::count_for_service(&found.slug)
        .await
        .map_err(|error| log_failure("counting the photos of a workshop", error))?;

    if held >= MAX_SERVICE_PHOTOS as u64 {
        return Err(ServerFnError::new(format!(
            "Cet atelier a déjà {MAX_SERVICE_PHOTOS} photos : supprimez-en une d'abord."
        )));
    }

    service_photo::insert(&found.slug, photo, content_type)
        .await
        .map(|_| ())
        .map_err(|error| log_failure("inserting a workshop photo", error))
}

/// Drops one photo.
#[server]
pub async fn delete_service_photo(id: String) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{service_photo, session};

    require_admin()?;

    let photo_id =
        session::parse_id(&id).map_err(|_| ServerFnError::new("Photo inconnue."))?;

    service_photo::delete(photo_id)
        .await
        .map_err(|error| log_failure("deleting a workshop photo", error))
}

/// Reports a malformed multipart body without describing it to the visitor.
#[cfg(feature = "ssr")]
fn read_failure(context: &str, error: impl std::fmt::Display) -> ServerFnError {
    eprintln!("{context} failed: {error}");

    ServerFnError::new("Ce formulaire n'a pas pu être lu. Réessayez.")
}
