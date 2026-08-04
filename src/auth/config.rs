use std::env;
use std::sync::OnceLock;
use std::time::Duration;

/// How long a granted access stays valid.
///
/// Deliberately short: the cookie is self-contained (no server-side state), so a
/// cookie that has been handed out cannot be revoked before it expires.
pub const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);

/// Shortest signing secret we accept.
const MIN_SECRET_LEN: usize = 32;

static CONFIG: OnceLock<AdminConfig> = OnceLock::new();

/// Credentials of the one and only admin account, read from the environment.
#[derive(Debug)]
pub struct AdminConfig {
    /// The address accepted at login.
    pub email: String,
    /// Argon2 hash of the password, in PHC format.
    pub password_hash: String,
    /// HMAC key used to sign access cookies.
    pub secret: Vec<u8>,
}

/// Why the admin area could not be configured.
#[derive(Debug)]
pub enum ConfigError {
    /// A required environment variable is missing or empty.
    Missing(&'static str),
    /// `ADMIN_SESSION_SECRET` is too short to act as an HMAC key.
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
    /// Reads and validates the configuration, then memoizes it for later calls.
    ///
    /// Meant to be called when the server starts, so that an incomplete
    /// configuration is reported right away. A second call does not re-read the
    /// environment and returns the configuration already validated.
    pub fn init() -> Result<&'static Self, ConfigError> {
        if let Some(existing) = CONFIG.get() {
            return Ok(existing);
        }

        // Values are always trimmed: a newline pasted by accident into a hosting
        // provider's dashboard would make the hash unreadable.
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

        // `init` only runs at startup: should two calls race, the first value
        // written wins and both are equivalent anyway.
        Ok(CONFIG.get_or_init(|| config))
    }

    /// The configuration validated at startup, or `None` when the admin area is
    /// disabled for lack of a usable configuration.
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
