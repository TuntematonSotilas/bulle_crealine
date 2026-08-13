//! Server functions over the workshop themes.
//!
//! [`save_theme`] is the one endpoint in this project that does not take plain form
//! fields: a photo cannot travel through the urlencoded body an `<ActionForm>`
//! builds, so it reads a multipart stream instead.

use leptos::prelude::*;
// At file scope rather than inside the function: both sides of `save_theme` name
// these, the browser to build the body and the server to read it.
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

use crate::models::{AffectedSession, ThemeView};

/// Every theme, for the admin listing and for the session form's picker.
///
/// Photos are not included; each view carries the URL to fetch one.
#[server]
pub async fn all_themes() -> Result<Vec<ThemeView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::theme;

    require_admin()?;

    let themes = theme::list_all()
        .await
        .map_err(|error| log_failure("listing themes", error))?;

    Ok(themes.iter().map(|found| found.to_view()).collect())
}

/// Creates a theme, or updates the one `id` points at when it is not empty.
///
/// Reads three multipart fields: `id`, `name` and `photo`. A creation requires a
/// photo; an update without one keeps the current image.
#[server(input = MultipartFormData)]
pub async fn save_theme(data: MultipartData) -> Result<(), ServerFnError> {
    use crate::auth::require_admin;
    use crate::db::session;
    use crate::db::theme::{self, detect_content_type};
    use crate::models::{MAX_PHOTO_BYTES, MAX_PHOTO_LABEL};

    require_admin()?;

    let mut fields = data
        .into_inner()
        .ok_or_else(|| ServerFnError::new("Ce formulaire n'a pas pu être lu."))?;

    let mut id = String::new();
    let mut name = String::new();
    let mut photo: Vec<u8> = Vec::new();

    while let Some(mut field) = fields
        .next_field()
        .await
        .map_err(|error| read_failure("reading the theme form", error))?
    {
        let field_name = field.name().unwrap_or_default().to_owned();

        match field_name.as_str() {
            "id" | "name" => {
                let text = field
                    .text()
                    .await
                    .map_err(|error| read_failure("reading a theme field", error))?;

                if field_name == "id" {
                    id = text;
                } else {
                    name = text;
                }
            }
            "photo" => {
                // Read chunk by chunk so an oversized upload is turned down as soon as
                // it crosses the line, rather than after it has all been buffered.
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|error| read_failure("reading the theme photo", error))?
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

    let name = name.trim();
    if name.is_empty() {
        return Err(ServerFnError::new("Indiquez un nom de thème."));
    }

    // An untouched file input still sends an empty part, which is how "keep the
    // current photo" reaches us.
    let uploaded = if photo.is_empty() {
        None
    } else {
        let content_type = detect_content_type(&photo).ok_or_else(|| {
            ServerFnError::new("Ce fichier n'est pas une image (formats acceptés : PNG, JPEG, WebP).")
        })?;

        Some((photo, content_type.to_owned()))
    };

    let id = id.trim();
    if id.is_empty() {
        let Some((bytes, content_type)) = uploaded else {
            return Err(ServerFnError::new("Choisissez une photo pour ce thème."));
        };

        return theme::insert(name, bytes, &content_type)
            .await
            .map(|_| ())
            .map_err(|error| duplicate_or_failure("inserting a theme", error));
    }

    let theme_id =
        session::parse_id(id).map_err(|_| ServerFnError::new("Thème inconnu."))?;

    theme::update(theme_id, name, uploaded)
        .await
        .map_err(|error| duplicate_or_failure("updating a theme", error))
}

/// Drops a theme, unless a session still uses it.
///
/// Unlike a session — whose deletion deliberately leaves bookings behind, so the
/// admin can still reach the people who signed up — a theme has nothing to leave
/// behind: a session without a theme has nothing to show. So this refuses.
#[server]
pub async fn delete_theme(id: String) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{session, theme};

    require_admin()?;

    let theme_id = session::parse_id(&id).map_err(|_| ServerFnError::new("Thème inconnu."))?;

    let used_by = session::count_for_theme(theme_id)
        .await
        .map_err(|error| log_failure("counting the sessions of a theme", error))?;

    if used_by > 0 {
        return Err(ServerFnError::new(format!(
            "{used_by} séance(s) utilisent ce thème : changez leur thème ou supprimez-les d'abord."
        )));
    }

    theme::delete(theme_id)
        .await
        .map_err(|error| log_failure("deleting a theme", error))
}

/// The sessions a theme is used by, to spell out what changing it affects.
#[server]
pub async fn theme_sessions(id: String) -> Result<Vec<AffectedSession>, ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{datetime, session};

    require_admin()?;

    let theme_id = session::parse_id(&id).map_err(|_| ServerFnError::new("Thème inconnu."))?;

    let sessions = session::list_for_theme(theme_id)
        .await
        .map_err(|error| log_failure("listing the sessions of a theme", error))?;

    Ok(sessions
        .iter()
        .map(|found| AffectedSession {
            date_label: datetime::to_label(found.date),
            service_label: found.service_type.label().to_owned(),
        })
        .collect())
}

/// Reports a malformed multipart body without describing it to the visitor.
#[cfg(feature = "ssr")]
fn read_failure(context: &str, error: impl std::fmt::Display) -> ServerFnError {
    eprintln!("{context} failed: {error}");

    ServerFnError::new("Ce formulaire n'a pas pu être lu. Réessayez.")
}

/// Turns the unique index on the name into a sentence, and anything else into the
/// usual storage failure.
#[cfg(feature = "ssr")]
fn duplicate_or_failure(context: &str, error: crate::db::DbError) -> ServerFnError {
    use crate::api::log_failure;
    use crate::db::DbError;

    match error {
        DbError::Duplicate => ServerFnError::new("Un thème porte déjà ce nom."),
        other => log_failure(context, other),
    }
}
