//! Génère les valeurs à placer dans les variables d'environnement de
//! l'administration.
//!
//! ```text
//! cargo run --example hash_password --features ssr -- "mon mot de passe"
//! ```
//!
//! Le mot de passe n'est jamais réaffiché : seul son hash Argon2 l'est, et c'est
//! ce hash qui va dans `ADMIN_PASSWORD_HASH`.

use std::env;

fn main() {
    let Some(password) = env::args().nth(1) else {
        eprintln!(
            "usage : cargo run --example hash_password --features ssr -- \"<mot de passe>\"\n\
             \n\
             Pensez aux guillemets si le mot de passe contient des espaces."
        );
        std::process::exit(2);
    };

    if password.chars().count() < 12 {
        eprintln!(
            "Ce mot de passe fait {} caractères. Il en faut au moins 12 : c'est la seule\n\
             chose qui protège l'administration.",
            password.chars().count()
        );
        std::process::exit(2);
    }

    let hash = match bulle_crealine::auth::hash_password(&password) {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("le hachage a échoué : {error}");
            std::process::exit(1);
        }
    };

    println!("ADMIN_PASSWORD_HASH={hash}");
    println!("ADMIN_SESSION_SECRET={}", random_secret());
    println!();
    println!("Ajoutez ces deux lignes, ainsi que ADMIN_EMAIL, aux variables");
    println!("d'environnement du serveur. Ne les versionnez pas.");
}

/// Tire un secret de signature de 64 caractères hexadécimaux (256 bits).
fn random_secret() -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};

    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
