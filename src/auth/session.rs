//! Admin access carried by a signed cookie.
//!
//! No state is kept on the server: the cookie itself holds the admin's address
//! and its expiry date, sealed with an HMAC-SHA256. Every request is therefore
//! authenticated end to end, with no database and no session table to maintain.
//!
//! The price of this self-contained format is that a cookie cannot be
//! invalidated remotely before it expires, hence the short lifetime set by
//! [`SESSION_TTL`].

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::config::{AdminConfig, SESSION_TTL};

type HmacSha256 = Hmac<Sha256>;

/// Name of the cookie carrying the token.
pub const COOKIE_NAME: &str = "bc_admin";

/// Payload format version, so it can be evolved later on.
const PAYLOAD_VERSION: &str = "v1";

/// Checks a password against an Argon2 hash in PHC format.
///
/// Returns `false` on an unreadable hash rather than propagating the error: an
/// invalid configuration must never let a login through.
pub fn verify_password(password: &str, phc_hash: &str) -> bool {
    match PasswordHash::new(phc_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Builds the `Set-Cookie` header granting access to the admin area.
///
/// `secure` must mirror the scheme of the request being served: the `Secure`
/// flag would stop the cookie from working over HTTP on localhost.
pub fn grant_cookie(config: &AdminConfig, secure: bool) -> String {
    let expires_at = now_unix() + SESSION_TTL.as_secs();
    let value = sign(&config.secret, &config.email, expires_at);
    format!(
        "{COOKIE_NAME}={value}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        SESSION_TTL.as_secs(),
        if secure { "; Secure" } else { "" }
    )
}

/// Builds the `Set-Cookie` header clearing the token from the browser.
pub fn revoke_cookie(secure: bool) -> String {
    format!(
        "{COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    )
}

/// Returns the address of the admin authenticated by the `Cookie` header.
///
/// Returns `None` when the cookie is absent, tampered with, expired, or when the
/// address it carries is no longer the configured one — that last case revokes
/// every token in circulation at once whenever `ADMIN_EMAIL` changes.
pub fn authenticated_email(cookie_header: Option<&str>) -> Option<String> {
    let config = AdminConfig::get()?;
    let value = cookie_value(cookie_header?, COOKIE_NAME)?;
    let email = verify(&config.secret, value)?;
    (email == config.email).then_some(email)
}

/// Seals `email` and `expires_at` into a `<payload>.<signature>` cookie value.
fn sign(secret: &[u8], email: &str, expires_at: u64) -> String {
    let payload = B64.encode(format!("{PAYLOAD_VERSION}|{expires_at}|{email}"));
    let signature = B64.encode(mac(secret, payload.as_bytes()));
    format!("{payload}.{signature}")
}

/// Checks the signature and expiry of a cookie value, and extracts the address
/// from it.
fn verify(secret: &[u8], value: &str) -> Option<String> {
    let (payload, signature) = value.split_once('.')?;

    // `verify_slice` compares in constant time, which keeps the check from
    // becoming an oracle on the bytes of the signature.
    let mut hmac = HmacSha256::new_from_slice(secret).ok()?;
    hmac.update(payload.as_bytes());
    hmac.verify_slice(&B64.decode(signature).ok()?).ok()?;

    let decoded = String::from_utf8(B64.decode(payload).ok()?).ok()?;
    let mut fields = decoded.splitn(3, '|');
    let version = fields.next()?;
    let expires_at: u64 = fields.next()?.parse().ok()?;
    let email = fields.next()?;

    (version == PAYLOAD_VERSION && expires_at > now_unix()).then(|| email.to_owned())
}

fn mac(secret: &[u8], message: &[u8]) -> Vec<u8> {
    let mut hmac =
        HmacSha256::new_from_slice(secret).expect("HmacSha256 takes a key of any length");
    hmac.update(message);
    hmac.finalize().into_bytes().to_vec()
}

/// Extracts one cookie's value from a raw `Cookie` header.
///
/// Saves us from enabling actix-web's `cookies` feature, which we would need for
/// this single read alone.
fn cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key.trim() == name).then(|| value.trim())
    })
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"a-test-secret-long-enough-for-hmac";

    #[test]
    fn accepts_a_cookie_it_signed() {
        let value = sign(SECRET, "admin@example.com", now_unix() + 60);
        assert_eq!(verify(SECRET, &value).as_deref(), Some("admin@example.com"));
    }

    #[test]
    fn rejects_an_expired_cookie() {
        let value = sign(SECRET, "admin@example.com", now_unix() - 1);
        assert!(verify(SECRET, &value).is_none());
    }

    #[test]
    fn rejects_a_cookie_signed_with_another_secret() {
        let value = sign(
            b"another-secret-just-as-long-as-the-real-one",
            "admin@example.com",
            now_unix() + 60,
        );
        assert!(verify(SECRET, &value).is_none());
    }

    #[test]
    fn rejects_a_tampered_payload() {
        // An expiry pushed back by hand must be rejected: the signature no
        // longer covers the payload.
        let value = sign(SECRET, "admin@example.com", now_unix() + 60);
        let (_, signature) = value.split_once('.').unwrap();
        let forged = B64.encode(format!(
            "{PAYLOAD_VERSION}|{}|admin@example.com",
            now_unix() + 99_999
        ));
        assert!(verify(SECRET, &format!("{forged}.{signature}")).is_none());
    }

    #[test]
    fn rejects_a_malformed_value() {
        for value in ["", ".", "no-dot-at-all", "aaaa.bbbb"] {
            assert!(
                verify(SECRET, value).is_none(),
                "{value:?} should have been rejected"
            );
        }
    }

    #[test]
    fn reads_the_cookie_among_others() {
        let header = "theme=dark; bc_admin=abc.def; other=1";
        assert_eq!(cookie_value(header, COOKIE_NAME), Some("abc.def"));
        assert_eq!(cookie_value("theme=dark", COOKIE_NAME), None);
    }

    #[test]
    fn verifies_an_argon2_password() {
        let phc = crate::auth::hash_password("correct horse battery staple").unwrap();

        assert!(verify_password("correct horse battery staple", &phc));
        assert!(!verify_password("the wrong password", &phc));
        assert!(!verify_password("correct horse battery staple", "not-a-phc-hash"));
    }
}
