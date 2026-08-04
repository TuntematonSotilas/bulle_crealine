//! Actix guard over the admin pages.

use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{COOKIE, LOCATION};
use actix_web::middleware::Next;
use actix_web::{Error, HttpResponse};

use crate::auth::{ADMIN_PATH, LOGIN_PATH, session};

/// Sends unauthenticated requests back to the login page.
///
/// Runs before Leptos renders anything, so an admin page is never built nor sent
/// to an unauthenticated visitor.
///
/// It does not cover server functions, served under `/api`: each of those has to
/// call [`crate::auth::require_admin`] itself.
pub async fn admin_guard<B>(
    request: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<EitherBody<B>>, Error>
where
    B: MessageBody + 'static,
{
    if is_protected(request.path()) && !is_authenticated(&request) {
        let redirect = HttpResponse::Found()
            .insert_header((LOCATION, LOGIN_PATH))
            .finish();
        return Ok(request.into_response(redirect).map_into_right_body());
    }

    Ok(next.call(request).await?.map_into_left_body())
}

/// True for the admin pages that require authentication.
fn is_protected(path: &str) -> bool {
    let path = path.trim_end_matches('/');
    // Compare the whole prefix, so as not to catch a public page whose path
    // happens to start with the same letters.
    let is_admin_page = path == ADMIN_PATH || path.starts_with(&format!("{ADMIN_PATH}/"));

    is_admin_page && path != LOGIN_PATH
}

fn is_authenticated(request: &ServiceRequest) -> bool {
    let cookie_header = request
        .headers()
        .get(COOKIE)
        .and_then(|value| value.to_str().ok());

    session::authenticated_email(cookie_header).is_some()
}

#[cfg(test)]
mod tests {
    use super::is_protected;

    #[test]
    fn protects_admin_pages() {
        assert!(is_protected("/admin"));
        assert!(is_protected("/admin/"));
        assert!(is_protected("/admin/ateliers"));
    }

    #[test]
    fn lets_the_login_page_through() {
        assert!(!is_protected("/admin/login"));
        assert!(!is_protected("/admin/login/"));
    }

    #[test]
    fn does_not_spill_onto_the_public_site() {
        for path in ["/", "/administration", "/admin-secret", "/qui-suis-je"] {
            assert!(!is_protected(path), "{path} is not an admin page");
        }
    }
}
