//! Jeton d'administration porté par un cookie signé.
//!
//! Aucun état n'est conservé côté serveur : le cookie contient lui-même
//! l'adresse de l'administrateur et sa date d'expiration, scellées par un HMAC
//! SHA-256. Chaque requête est donc authentifiée de bout en bout sans base de
//! données ni table de sessions à entretenir.
//!
//! La contrepartie de ce format auto-porté est qu'un cookie émis ne peut pas
//! être invalidé à distance avant son expiration, d'où la durée de vie courte
//! fixée par [`SESSION_TTL`].

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::config::{AdminConfig, SESSION_TTL};

type HmacSha256 = Hmac<Sha256>;

/// Nom du cookie portant le jeton.
pub const COOKIE_NAME: &str = "bc_admin";

/// Version du format de charge utile, pour pouvoir le faire évoluer plus tard.
const PAYLOAD_VERSION: &str = "v1";

/// Vérifie un mot de passe contre un hash Argon2 au format PHC.
///
/// Renvoie `false` sur un hash illisible plutôt que de propager l'erreur : une
/// configuration invalide ne doit jamais laisser passer une connexion.
pub fn verify_password(password: &str, phc_hash: &str) -> bool {
    match PasswordHash::new(phc_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Construit l'en-tête `Set-Cookie` accordant l'accès à l'administration.
///
/// `secure` doit refléter le schéma de la requête en cours : le drapeau
/// `Secure` empêcherait le cookie de fonctionner en HTTP sur localhost.
pub fn grant_cookie(config: &AdminConfig, secure: bool) -> String {
    let expires_at = now_unix() + SESSION_TTL.as_secs();
    let value = sign(&config.secret, &config.email, expires_at);
    format!(
        "{COOKIE_NAME}={value}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        SESSION_TTL.as_secs(),
        if secure { "; Secure" } else { "" }
    )
}

/// Construit l'en-tête `Set-Cookie` effaçant le jeton côté navigateur.
pub fn revoke_cookie(secure: bool) -> String {
    format!(
        "{COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    )
}

/// Renvoie l'adresse de l'administrateur authentifié par l'en-tête `Cookie`.
///
/// Renvoie `None` si le cookie est absent, altéré, expiré, ou si l'adresse
/// qu'il porte n'est plus celle configurée — ce dernier cas révoque d'un coup
/// tous les jetons en circulation lorsque `ADMIN_EMAIL` change.
pub fn authenticated_email(cookie_header: Option<&str>) -> Option<String> {
    let config = AdminConfig::get()?;
    let value = cookie_value(cookie_header?, COOKIE_NAME)?;
    let email = verify(&config.secret, value)?;
    (email == config.email).then_some(email)
}

/// Scelle `email` et `expires_at` en une valeur de cookie `<charge>.<signature>`.
fn sign(secret: &[u8], email: &str, expires_at: u64) -> String {
    let payload = B64.encode(format!("{PAYLOAD_VERSION}|{expires_at}|{email}"));
    let signature = B64.encode(mac(secret, payload.as_bytes()));
    format!("{payload}.{signature}")
}

/// Contrôle la signature et l'expiration d'une valeur de cookie, et en extrait
/// l'adresse.
fn verify(secret: &[u8], value: &str) -> Option<String> {
    let (payload, signature) = value.split_once('.')?;

    // `verify_slice` compare en temps constant, ce qui évite de transformer la
    // vérification en oracle sur les octets de la signature.
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
        HmacSha256::new_from_slice(secret).expect("HmacSha256 accepte une clé de toute longueur");
    hmac.update(message);
    hmac.finalize().into_bytes().to_vec()
}

/// Extrait la valeur d'un cookie d'un en-tête `Cookie` brut.
///
/// Évite d'activer la fonctionnalité `cookies` d'actix-web, dont nous n'aurions
/// besoin que pour cette seule lecture.
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
        assert!(!verify_password("mauvais mot de passe", &phc));
        assert!(!verify_password(
            "correct horse battery staple",
            "pas-un-hash-phc"
        ));
    }
}
