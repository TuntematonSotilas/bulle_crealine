//! Authentication of the admin area.
//!
//! The site has a single administrator, whose credentials come from the
//! environment ([`config::AdminConfig`]): there is no sign-up, no user table and
//! no database to query.
//!
//! Logging in checks the password with Argon2, then drops a signed cookie
//! ([`session`]). That cookie is re-checked on every request, in two
//! complementary ways:
//!
//! - [`middleware::admin_guard`] turns away `/admin/*` pages before Leptos even
//!   renders them;
//! - [`require_admin`] must open every server function reserved for the admin,
//!   because `/api/*` calls do not go through the guard above.

use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod middleware;
#[cfg(feature = "ssr")]
pub mod session;
#[cfg(feature = "ssr")]
pub mod throttle;

/// Message shown when a failure must not reveal anything about its cause.
const GENERIC_FAILURE: &str = "La connexion a échoué. Réessayez.";

/// Path of the login page, the only `/admin` page open to everyone.
pub const LOGIN_PATH: &str = "/admin/login";

/// Path of the admin home page.
pub const ADMIN_PATH: &str = "/admin";

/// Grants admin access and redirects to `/admin`.
#[server]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use actix_web::http::header::{HeaderValue, SET_COOKIE};

    // No `await` in this body: the Actix request is exposed to Leptos through a
    // `SendWrapper` tied to the current thread, and crossing one would make the
    // accesses that follow panic.
    let request = expect_context::<leptos_actix::Request>();
    let response = expect_context::<leptos_actix::ResponseOptions>();
    let connection = request.connection_info();

    let config = config::AdminConfig::get().ok_or_else(|| {
        ServerFnError::new("L'administration n'est pas configurée sur ce serveur.")
    })?;

    // The origin address is read from the proxy headers, so it can be forged: the
    // counter discourages a naive attack but does not hold up against someone
    // rotating the address they announce. It is Argon2, and Argon2 alone, that
    // makes the password expensive to guess; forging the address also lets an
    // attacker lock the real administrator out for the length of a lockout.
    let ip = connection.realip_remote_addr().unwrap_or("inconnue");

    if let Err(remaining) = throttle::check(ip) {
        return Err(ServerFnError::new(format!(
            "Trop de tentatives. Réessayez dans {} minutes.",
            remaining.as_secs() / 60 + 1
        )));
    }

    // The password is checked even when the address does not match, so that the
    // response time gives no way to guess the expected address. Argon2 is
    // deliberately costly; it is the counter above that bounds how many checks a
    // visitor can trigger.
    let email_matches = email.trim().to_lowercase() == config.email;
    let password_matches = session::verify_password(&password, &config.password_hash);

    if !(email_matches && password_matches) {
        throttle::record_failure(ip);
        return Err(ServerFnError::new("Adresse ou mot de passe incorrect."));
    }

    throttle::record_success(ip);

    let cookie = session::grant_cookie(config, connection.scheme() == "https");
    let cookie = HeaderValue::from_str(&cookie).map_err(|_| ServerFnError::new(GENERIC_FAILURE))?;
    response.append_header(SET_COOKIE, cookie);

    // `SameSite=Lax` keeps the browser from sending this cookie on a POST coming
    // from another site: request forgery is ruled out without an extra CSRF
    // token.
    leptos_actix::redirect(ADMIN_PATH);
    Ok(())
}

/// Clears the admin cookie and sends the visitor back to the login page.
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use actix_web::http::header::{HeaderValue, SET_COOKIE};

    let request = expect_context::<leptos_actix::Request>();
    let response = expect_context::<leptos_actix::ResponseOptions>();

    let cookie = session::revoke_cookie(request.connection_info().scheme() == "https");
    let cookie = HeaderValue::from_str(&cookie)
        .map_err(|_| ServerFnError::new("La déconnexion a échoué."))?;
    response.append_header(SET_COOKIE, cookie);

    leptos_actix::redirect(LOGIN_PATH);
    Ok(())
}

/// Address of the logged-in admin, or `None` when the request is not
/// authenticated.
///
/// Used for client-side rendering, when Leptos's in-app navigation has not gone
/// through [`middleware::admin_guard`]. This is not a security barrier: sensitive
/// data must be protected by [`require_admin`] inside the server function that
/// exposes it.
#[server]
pub async fn admin_email() -> Result<Option<String>, ServerFnError> {
    Ok(current_admin())
}

/// Address of the admin authenticated by the current request.
#[cfg(feature = "ssr")]
pub fn current_admin() -> Option<String> {
    use actix_web::http::header::COOKIE;

    let request = use_context::<leptos_actix::Request>()?;
    let cookie_header = request.headers().get(COOKIE)?.to_str().ok()?;
    session::authenticated_email(Some(cookie_header))
}

/// Requires an authenticated request; belongs at the top of every server function
/// reserved for the admin.
///
/// Returns the admin's address, for logging or attribution.
#[cfg(feature = "ssr")]
pub fn require_admin() -> Result<String, ServerFnError> {
    current_admin().ok_or_else(|| ServerFnError::new("Accès réservé à l'administration."))
}

/// Computes the Argon2 hash of a password, in PHC format.
///
/// Used by `examples/hash_password.rs` to produce the value that goes into
/// `ADMIN_PASSWORD_HASH`.
#[cfg(feature = "ssr")]
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::password_hash::{SaltString, rand_core::OsRng};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Turns a server function error into something displayable.
///
/// Only the messages we wrote ourselves are passed through as they are;
/// everything else (network failure, serialization error…) is replaced by a
/// generic message, so as not to expose internals to the visitor.
pub fn user_message(error: &ServerFnError) -> String {
    match error {
        ServerFnError::ServerError(message) => message.clone(),
        _ => GENERIC_FAILURE.to_owned(),
    }
}
