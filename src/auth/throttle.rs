//! Limitation des tentatives de connexion, par adresse IP.
//!
//! Argon2 ralentit déjà fortement une attaque par force brute, mais rien
//! n'empêcherait de la mener en parallèle. Un compteur en mémoire suffit ici :
//! le site tourne sur une instance unique, et perdre les compteurs à un
//! redémarrage est sans conséquence puisque le mot de passe reste, lui, hors
//! d'atteinte.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Nombre d'échecs consécutifs toléré avant blocage.
const MAX_FAILURES: u32 = 5;

/// Durée du blocage, et délai au bout duquel un compteur inactif est oublié.
const LOCKOUT: Duration = Duration::from_secs(15 * 60);

/// Au-delà, on purge les compteurs périmés : sans cela, des requêtes forgeant
/// l'IP d'origine feraient grossir la table indéfiniment.
const PRUNE_THRESHOLD: usize = 1024;

static FAILURES: OnceLock<Mutex<HashMap<String, Attempts>>> = OnceLock::new();

#[derive(Debug)]
struct Attempts {
    count: u32,
    last: Instant,
}

/// Vérifie qu'`ip` a encore le droit d'essayer.
///
/// Renvoie le temps d'attente restant si le blocage est en cours.
pub fn check(ip: &str) -> Result<(), Duration> {
    let mut failures = lock();
    match failures.get(ip) {
        Some(attempts) if attempts.count >= MAX_FAILURES => {
            match LOCKOUT.checked_sub(attempts.last.elapsed()) {
                Some(remaining) => Err(remaining),
                // Blocage écoulé : on repart de zéro.
                None => {
                    failures.remove(ip);
                    Ok(())
                }
            }
        }
        _ => Ok(()),
    }
}

/// Comptabilise un échec de connexion.
pub fn record_failure(ip: &str) {
    let mut failures = lock();

    if failures.len() >= PRUNE_THRESHOLD {
        failures.retain(|_, attempts| attempts.last.elapsed() < LOCKOUT);
    }

    failures
        .entry(ip.to_owned())
        .and_modify(|attempts| {
            // Une série d'échecs interrompue assez longtemps ne compte plus.
            attempts.count = if attempts.last.elapsed() < LOCKOUT {
                attempts.count.saturating_add(1)
            } else {
                1
            };
            attempts.last = Instant::now();
        })
        .or_insert(Attempts {
            count: 1,
            last: Instant::now(),
        });
}

/// Efface le compteur après une connexion réussie.
pub fn record_success(ip: &str) {
    lock().remove(ip);
}

fn lock() -> std::sync::MutexGuard<'static, HashMap<String, Attempts>> {
    let failures = FAILURES.get_or_init(|| Mutex::new(HashMap::new()));
    // Un poison ne peut venir que d'une panique sous le verrou ; les compteurs
    // restent exploitables et les perdre serait plus grave que de les reprendre.
    failures.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_the_tolerated_number_of_failures() {
        let ip = "203.0.113.1";
        for _ in 0..MAX_FAILURES - 1 {
            record_failure(ip);
            assert!(check(ip).is_ok());
        }

        record_failure(ip);
        assert!(check(ip).is_err(), "the lockout should have kicked in");

        record_success(ip);
        assert!(check(ip).is_ok(), "a success should have lifted the lockout");
    }

    #[test]
    fn counts_addresses_separately() {
        let blocked = "203.0.113.2";
        for _ in 0..MAX_FAILURES {
            record_failure(blocked);
        }

        assert!(check(blocked).is_err());
        assert!(check("203.0.113.3").is_ok());
    }
}
