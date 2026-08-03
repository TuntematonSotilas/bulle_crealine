//! Authentification de l'espace d'administration.
//!
//! Le site n'a qu'un seul administrateur, dont les identifiants sont fournis par
//! l'environnement ([`config::AdminConfig`]) : il n'y a donc ni inscription, ni
//! table d'utilisateurs, ni base de données à interroger.
//!
//! La connexion vérifie le mot de passe avec Argon2 puis dépose un cookie signé
//! ([`session`]). Ce cookie est ensuite revérifié à chaque requête, de deux
//! façons complémentaires :
//!
//! - [`middleware::admin_guard`] refuse les pages `/admin/*` avant même que
//!   Leptos ne les rende ;
//! - [`require_admin`] doit ouvrir toute server function réservée à
//!   l'administration, car les appels `/api/*` ne passent pas par le garde-fou
//!   ci-dessus.

use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod middleware;
#[cfg(feature = "ssr")]
pub mod session;
#[cfg(feature = "ssr")]
pub mod throttle;

/// Message affiché quand l'échec ne doit rien révéler de sa cause.
const GENERIC_FAILURE: &str = "La connexion a échoué. Réessayez.";

/// Chemin de la page de connexion, seule page `/admin` ouverte à tous.
pub const LOGIN_PATH: &str = "/admin/login";

/// Chemin de l'accueil de l'administration.
pub const ADMIN_PATH: &str = "/admin";

/// Ouvre une session d'administration et redirige vers `/admin`.
#[server]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use actix_web::http::header::{HeaderValue, SET_COOKIE};

    // Aucun `await` dans ce corps : la requête Actix est exposée à Leptos via un
    // `SendWrapper` lié au fil d'exécution courant, et la franchir ferait
    // paniquer les accès suivants.
    let request = expect_context::<leptos_actix::Request>();
    let response = expect_context::<leptos_actix::ResponseOptions>();
    let connection = request.connection_info();

    let config = config::AdminConfig::get().ok_or_else(|| {
        ServerFnError::new("L'administration n'est pas configurée sur ce serveur.")
    })?;

    // L'adresse d'origine est lue dans les en-têtes du proxy, donc falsifiable :
    // le compteur décourage une attaque naïve mais ne résiste pas à quelqu'un qui
    // fait tourner l'adresse annoncée. C'est bien Argon2, et lui seul, qui rend le
    // mot de passe coûteux à deviner ; l'usurpation permet en outre de bloquer
    // l'accès du véritable administrateur pendant la durée d'un blocage.
    let ip = connection.realip_remote_addr().unwrap_or("inconnue");

    if let Err(remaining) = throttle::check(ip) {
        return Err(ServerFnError::new(format!(
            "Trop de tentatives. Réessayez dans {} minutes.",
            remaining.as_secs() / 60 + 1
        )));
    }

    // Le mot de passe est vérifié même lorsque l'adresse ne correspond pas, afin
    // que le temps de réponse ne permette pas de deviner l'adresse attendue.
    // Argon2 est volontairement coûteux ; c'est le compteur ci-dessus qui borne
    // le nombre de vérifications qu'un visiteur peut déclencher.
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

    // `SameSite=Lax` empêche le navigateur d'envoyer ce cookie sur une requête
    // POST venant d'un autre site : la falsification de requête est écartée sans
    // jeton anti-CSRF supplémentaire.
    leptos_actix::redirect(ADMIN_PATH);
    Ok(())
}

/// Efface le cookie d'administration et renvoie vers la page de connexion.
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

/// Adresse de l'administrateur connecté, ou `None` si la requête n'est pas
/// authentifiée.
///
/// Sert au rendu côté client, quand la navigation interne de Leptos n'est pas
/// passée par [`middleware::admin_guard`]. Ce n'est pas une barrière de
/// sécurité : les données sensibles doivent être protégées par [`require_admin`]
/// dans la server function qui les expose.
#[server]
pub async fn admin_email() -> Result<Option<String>, ServerFnError> {
    Ok(current_admin())
}

/// Adresse de l'administrateur authentifié par la requête courante.
#[cfg(feature = "ssr")]
pub fn current_admin() -> Option<String> {
    use actix_web::http::header::COOKIE;

    let request = use_context::<leptos_actix::Request>()?;
    let cookie_header = request.headers().get(COOKIE)?.to_str().ok()?;
    session::authenticated_email(Some(cookie_header))
}

/// Exige une requête authentifiée, à placer en tête de toute server function
/// réservée à l'administration.
///
/// Renvoie l'adresse de l'administrateur, pour journalisation ou attribution.
#[cfg(feature = "ssr")]
pub fn require_admin() -> Result<String, ServerFnError> {
    current_admin().ok_or_else(|| ServerFnError::new("Accès réservé à l'administration."))
}

/// Calcule le hash Argon2 d'un mot de passe, au format PHC.
///
/// Utilisé par `examples/hash_password.rs` pour produire la valeur à placer dans
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

/// Traduit une erreur de server function en message affichable.
///
/// Seuls les messages que nous avons rédigés sont repris tels quels ; tout le
/// reste (panne réseau, erreur de sérialisation…) est remplacé par un message
/// générique, pour ne pas exposer d'interne au visiteur.
pub fn user_message(error: &ServerFnError) -> String {
    match error {
        ServerFnError::ServerError(message) => message.clone(),
        _ => GENERIC_FAILURE.to_owned(),
    }
}
