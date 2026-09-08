use serde::{Deserialize, Serialize};

/// The kind of workshop a session belongs to.
///
/// The serde representation is the URL slug, so the value stored in Mongo, the
/// value posted by a form and the segment in `/booking/<slug>` are all the same
/// string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    ParentsEnfantsMoinsSix,
    ParentsEnfantsSixADouze,
    AperosCreatifs,
    ApresMidisCreatifs,
}

impl ServiceType {
    /// Every variant, in the order the admin form lists them.
    pub const ALL: [Self; 4] = [
        Self::ParentsEnfantsMoinsSix,
        Self::ParentsEnfantsSixADouze,
        Self::AperosCreatifs,
        Self::ApresMidisCreatifs,
    ];

    /// URL slug, and the value stored in Mongo.
    ///
    /// Must stay in step with the `serde` renaming above; a test guards that.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::ParentsEnfantsMoinsSix => "parents-enfants-moins-six",
            Self::ParentsEnfantsSixADouze => "parents-enfants-six-a-douze",
            Self::AperosCreatifs => "aperos-creatifs",
            Self::ApresMidisCreatifs => "apres-midis-creatifs"
        }
    }

    /// Name shown to visitors.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ParentsEnfantsMoinsSix => "Ateliers parents-enfants (moins de 6 ans)",
            Self::ParentsEnfantsSixADouze => "Ateliers parents-enfants (6 à 12 ans)",
            Self::AperosCreatifs => "Apéros créatifs (adultes)",
            Self::ApresMidisCreatifs => "Ateliers après-midi créatifs (adultes)",
        }
    }

    /// One or two sentences introducing this kind of workshop.
    ///
    /// Lives here rather than on the page that shows it so the home page's session
    /// cards and the service page itself cannot drift apart.
    pub const fn description(self) -> &'static str {
        match self {
            Self::ParentsEnfantsMoinsSix | Self::ParentsEnfantsSixADouze => {
                "Nos ateliers parents-enfants offrent un espace de création commune, \
                 favorisant un temps de partage loin des impératifs quotidiens."
            }
            Self::AperosCreatifs => {
                "Une soirée entre adultes autour d'un verre et d'un projet créatif, \
                 sans prérequis : on vient souffler et on repart avec sa création."
            }
            Self::ApresMidisCreatifs => {
                "Découvrez nos ateliers créatifs conçus pour tous les âges et tous les niveaux."
            }
        }
    }

    /// Path of the public page describing this kind of workshop.
    pub const fn page_path(self) -> &'static str {
        match self {
            Self::ParentsEnfantsMoinsSix => "/services/parents-enfants-moins-six",
            Self::ParentsEnfantsSixADouze => "/services/parents-enfants-six-a-douze",
            Self::AperosCreatifs => "/services/aperos-creatifs",
            Self::ApresMidisCreatifs => "/services/apres-midis-creatifs",
        }
    }

    /// Reads a variant back from its [`slug`](Self::slug).
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.slug() == slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_round_trips() {
        for kind in ServiceType::ALL {
            assert_eq!(ServiceType::from_slug(kind.slug()), Some(kind));
        }
    }

    #[test]
    fn rejects_an_unknown_slug() {
        assert_eq!(ServiceType::from_slug("does-not-exist"), None);
        assert_eq!(ServiceType::from_slug(""), None);
    }

    #[test]
    fn slugs_are_distinct() {
        let mut slugs: Vec<_> = ServiceType::ALL.iter().map(|kind| kind.slug()).collect();
        slugs.sort_unstable();
        let count = slugs.len();
        slugs.dedup();

        assert_eq!(slugs.len(), count, "two variants share a slug");
    }

    /// The slug is also the stored and posted value, so `slug()` and the serde
    /// renaming must not drift apart.
    #[test]
    fn slug_matches_the_serde_representation() {
        for kind in ServiceType::ALL {
            let serialized = serde_json::to_string(&kind).expect("a unit variant serializes");
            assert_eq!(serialized, format!("\"{}\"", kind.slug()));
        }
    }

    /// The home page prints this next to every session, so an empty one would
    /// leave a blank gap on a card rather than fail anywhere visible.
    #[test]
    fn every_kind_is_described() {
        for kind in ServiceType::ALL {
            let description = kind.description();

            assert!(!description.trim().is_empty(), "{kind:?} has no description");
            assert!(
                !description.contains("  "),
                "{kind:?} kept the indentation of its continued string literal: {description}"
            );
        }
    }

    #[test]
    fn page_paths_point_at_the_slug() {
        for kind in ServiceType::ALL {
            assert_eq!(kind.page_path(), format!("/services/{}", kind.slug()));
        }
    }
}
