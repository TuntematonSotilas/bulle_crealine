use std::env;
use std::sync::OnceLock;
use std::time::Duration;

/// Durée de vie d'une session.
///
/// Volontairement courte : le cookie est auto-porteur (aucun état côté serveur),
/// donc une session émise ne peut pas être révoquée avant son expiration.
pub const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);

/// Longueur minimale acceptée pour le secret de signature.
const MIN_SECRET_LEN: usize = 32;

static CONFIG: OnceLock<AdminConfig> = OnceLock::new();

/// Identifiants de l'unique compte d'administration, lus dans l'environnement.
#[derive(Debug)]
pub struct AdminConfig {
    /// Adresse acceptée à la connexion.
    pub email: String,
    /// Hash Argon2 du mot de passe, au format PHC.
    pub password_hash: String,
    /// Clé HMAC utilisée pour signer les cookies de session.
    pub secret: Vec<u8>,
}

/// Raison pour laquelle l'administration n'a pas pu être configurée.
#[derive(Debug)]
pub enum ConfigError {
    /// Une variable d'environnement obligatoire est absente ou vide.
    Missing(&'static str),
    /// `ADMIN_SESSION_SECRET` est trop court pour servir de clé HMAC.
    SecretTooShort { len: usize },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(name) => {
                write!(f, "la variable d'environnement {name} est absente ou vide")
            }
            Self::SecretTooShort { len } => write!(
                f,
                "ADMIN_SESSION_SECRET fait {len} caractères, il en faut au moins {MIN_SECRET_LEN}"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl AdminConfig {
    /// Lit et valide la configuration, puis la mémorise pour les appels suivants.
    ///
    /// À appeler au démarrage du serveur afin que l'exploitant voie tout de suite
    /// une éventuelle configuration incomplète. Un second appel ne relit pas
    /// l'environnement et renvoie la configuration déjà validée.
    pub fn init() -> Result<&'static Self, ConfigError> {
        if let Some(existing) = CONFIG.get() {
            return Ok(existing);
        }

        // Les valeurs sont systématiquement rognées : un retour à la ligne collé
        // par erreur dans un panneau d'hébergeur rendrait le hash illisible.
        let email = required("ADMIN_EMAIL")?.trim().to_lowercase();
        let password_hash = required("ADMIN_PASSWORD_HASH")?.trim().to_owned();
        let secret = required("ADMIN_SESSION_SECRET")?.trim().to_owned();

        if secret.len() < MIN_SECRET_LEN {
            return Err(ConfigError::SecretTooShort { len: secret.len() });
        }

        let config = Self {
            email,
            password_hash,
            secret: secret.into_bytes(),
        };

        // `init` n'est appelé qu'au démarrage : en cas de course, la première
        // valeur écrite fait foi et les deux sont équivalentes.
        Ok(CONFIG.get_or_init(|| config))
    }

    /// Configuration validée au démarrage, ou `None` si l'administration est
    /// désactivée faute de configuration valable.
    pub fn get() -> Option<&'static Self> {
        CONFIG.get()
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(ConfigError::Missing(name)),
    }
}
