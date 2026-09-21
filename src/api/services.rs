//! Server functions over the kinds of workshop on offer.
//!
//! [`all_services`] is open to everyone, unlike the rest of this module: the
//! menu, the catalogue and every workshop page are built from it, so it carries
//! nothing a visitor could not already read on the site.

use leptos::prelude::*;

use crate::models::ServiceView;

/// Every workshop, bookable ones first, then by rank and by name.
#[server]
pub async fn all_services() -> Result<Vec<ServiceView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::service;

    let services = service::list_all()
        .await
        .map_err(|error| log_failure("listing workshops", error))?;

    Ok(services.iter().map(|found| found.to_view()).collect())
}

/// Creates a workshop, or updates the one `id` points at when it is not empty.
///
/// `steps` arrives as one line per step, which is how a textarea can stand in for
/// a list without a field-adding widget. Blank lines are dropped, so a trailing
/// newline does not become an empty step.
// One parameter per form field, which is what an `<ActionForm>` posts: gathering
// them into a struct would change the wire format the form builds.
#[allow(clippy::too_many_arguments)]
#[server]
pub async fn save_service(
    id: String,
    slug: String,
    label: String,
    description: String,
    age: String,
    icon: String,
    pro: String,
    steps: String,
    position: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::require_admin;
    use crate::db::service::{self, ServiceDoc};
    use crate::db::session;
    use crate::models::{SERVICE_ICONS, is_valid_slug};

    require_admin()?;

    let label = label.trim();
    if label.is_empty() {
        return Err(ServerFnError::new("Indiquez un nom de service."));
    }

    let description = description.trim();
    if description.is_empty() {
        return Err(ServerFnError::new("Indiquez une description."));
    }

    // Empty is allowed -- a workshop may have no picto -- but a name the icon
    // crate does not know would render an empty box on every page showing it.
    let icon = icon.trim();
    if !icon.is_empty() && !SERVICE_ICONS.iter().any(|(name, _)| *name == icon) {
        return Err(ServerFnError::new("Ce picto n'existe pas."));
    }

    let steps: Vec<String> = steps
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();

    let document = ServiceDoc {
        id: None,
        slug: slug.trim().to_owned(),
        label: label.to_owned(),
        description: description.to_owned(),
        age: age.trim().to_owned(),
        steps,
        icon: icon.to_owned(),
        pro: pro.trim() == "oui",
        position,
    };

    let id = id.trim();
    if id.is_empty() {
        if !is_valid_slug(&document.slug) {
            return Err(ServerFnError::new(
                "L'identifiant ne peut contenir que des minuscules, des chiffres et des tirets.",
            ));
        }

        return service::insert(&document)
            .await
            .map(|_| ())
            .map_err(|error| duplicate_or_failure("inserting a workshop", error));
    }

    let service_id =
        session::parse_id(id).map_err(|_| ServerFnError::new("Service inconnu."))?;

    // Moving a workshop into the section for structures takes away its booking
    // page, and the home page would go on offering "Réserver" on every date still
    // scheduled for it. Refused while any exists, rather than left to break.
    if document.pro {
        let scheduled = session::count_for_service(&document.slug)
            .await
            .map_err(|error| {
                crate::api::log_failure("counting the sessions of a workshop", error)
            })?;

        if scheduled > 0 {
            return Err(ServerFnError::new(format!(
                "{scheduled} séance(s) portent ce service : il ne peut pas passer en « {} » tant qu'elles existent.",
                crate::models::section_title(true)
            )));
        }
    }

    // The slug posted by an edit is ignored rather than refused: the form shows it
    // as text precisely because sessions and bookings are filed under it.
    service::update(service_id, &document)
        .await
        .map_err(|error| duplicate_or_failure("updating a workshop", error))
}

/// Drops a workshop, unless sessions are still filed under it.
///
/// Refused rather than cascaded, like a theme: those sessions store the slug, and
/// a session whose workshop is gone loses its name on the home page, in the admin
/// table and on the booking page at once.
#[server]
pub async fn delete_service(id: String) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{service, session};

    require_admin()?;

    let service_id = session::parse_id(&id).map_err(|_| ServerFnError::new("Service inconnu."))?;

    let found = service::find(service_id)
        .await
        .map_err(|error| log_failure("loading a workshop before deleting it", error))?
        .ok_or_else(|| ServerFnError::new("Service inconnu."))?;

    let used_by = session::count_for_service(&found.slug)
        .await
        .map_err(|error| log_failure("counting the sessions of a workshop", error))?;

    if used_by > 0 {
        return Err(ServerFnError::new(format!(
            "{used_by} séance(s) portent ce service : supprimez-les d'abord."
        )));
    }

    service::delete(service_id)
        .await
        .map_err(|error| log_failure("deleting a workshop", error))
}

/// How many sessions are filed under a workshop, to warn before an edit and to
/// explain why a deletion is refused.
#[server]
pub async fn service_session_count(id: String) -> Result<u64, ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{service, session};

    require_admin()?;

    let service_id = session::parse_id(&id).map_err(|_| ServerFnError::new("Service inconnu."))?;

    let Some(found) = service::find(service_id)
        .await
        .map_err(|error| log_failure("loading a workshop", error))?
    else {
        return Ok(0);
    };

    session::count_for_service(&found.slug)
        .await
        .map_err(|error| log_failure("counting the sessions of a workshop", error))
}

/// Turns the unique index on the slug into a sentence, and anything else into the
/// usual storage failure.
#[cfg(feature = "ssr")]
fn duplicate_or_failure(context: &str, error: crate::db::DbError) -> ServerFnError {
    use crate::api::log_failure;
    use crate::db::DbError;

    match error {
        DbError::Duplicate => ServerFnError::new("Un service utilise déjà cet identifiant."),
        other => log_failure(context, other),
    }
}
