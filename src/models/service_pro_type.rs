/// A workshop run for a structure rather than booked online.
///
/// Deliberately separate from [`ServiceType`](crate::models::ServiceType), which
/// it mirrors method for method. That one is serialised into Mongo, resolves
/// `/booking/:service` through `from_slug`, and its `ALL` fills the dropdown of
/// the admin's session form; a workshop nobody can book online has no business
/// appearing in any of the three.
///
/// No `serde` derive for the same reason: nothing stores these, so there is no
/// representation to keep in step with the slug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ServiceProType {
    EnInstitution,
    HorsLesMurs,
    Individuels,
}

impl ServiceProType {
    /// Every variant, in the order the menu and the catalogue list them.
    pub const ALL: [Self; 3] = [Self::EnInstitution, Self::HorsLesMurs, Self::Individuels];

    /// URL slug, and the last segment of [`page_path`](Self::page_path).
    pub const fn slug(self) -> &'static str {
        match self {
            Self::EnInstitution => "en-institution",
            Self::HorsLesMurs => "hors-les-murs",
            Self::Individuels => "individuels",
        }
    }

    /// Name shown to visitors.
    pub const fn label(self) -> &'static str {
        match self {
            Self::EnInstitution => "Ateliers en institution",
            Self::HorsLesMurs => "Ateliers hors les murs",
            Self::Individuels => "Ateliers individuels",
        }
    }

    /// One or two sentences introducing this kind of workshop.
    ///
    /// Lives here rather than on the page that shows it so the catalogue and the
    /// workshop's own page, once it exists, cannot drift apart.
    pub const fn description(self) -> &'static str {
        match self {
            Self::EnInstitution => {
                "Des ateliers créatifs menés au sein de votre structure, \
                 adaptés au rythme et aux capacités de chaque groupe."
            }
            Self::HorsLesMurs => {
                "L'atelier se déplace : le matériel et l'accompagnement viennent \
                 jusqu'à vous, sur le lieu de votre choix."
            }
            Self::Individuels => {
                "Un accompagnement en tête-à-tête, au rythme de la personne, \
                 pour qui préfère créer sans le groupe."
            }
        }
    }

    /// Path of the public page describing this kind of workshop.
    ///
    /// None of these pages is built yet, so they currently land on the catch-all.
    pub const fn page_path(self) -> &'static str {
        match self {
            Self::EnInstitution => "/pro/en-institution",
            Self::HorsLesMurs => "/pro/hors-les-murs",
            Self::Individuels => "/pro/individuels",
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
        for kind in ServiceProType::ALL {
            assert_eq!(ServiceProType::from_slug(kind.slug()), Some(kind));
        }
    }

    #[test]
    fn rejects_an_unknown_slug() {
        assert_eq!(ServiceProType::from_slug("aperos-creatifs"), None);
        assert_eq!(ServiceProType::from_slug(""), None);
    }

    #[test]
    fn slugs_are_distinct() {
        let mut slugs: Vec<_> = ServiceProType::ALL.iter().map(|kind| kind.slug()).collect();
        slugs.sort_unstable();
        slugs.dedup();

        assert_eq!(slugs.len(), ServiceProType::ALL.len(), "two kinds share a slug");
    }

    /// A continued string literal keeps the indentation of the following line
    /// unless it is written with a trailing `\`, which shows up as a run of
    /// spaces in the middle of a sentence.
    #[test]
    fn every_kind_is_described() {
        for kind in ServiceProType::ALL {
            let description = kind.description();

            assert!(!description.is_empty(), "{kind:?} has no description");
            assert!(
                !description.contains("  "),
                "{kind:?} has a run of spaces: {description:?}"
            );
        }
    }

    #[test]
    fn page_paths_point_at_the_slug() {
        for kind in ServiceProType::ALL {
            assert_eq!(kind.page_path(), format!("/pro/{}", kind.slug()));
        }
    }

    /// The two families must not overlap, or a slug would name two workshops and
    /// `/pro/x` would describe something already reachable at `/services/x`.
    #[test]
    fn no_slug_is_shared_with_a_bookable_workshop() {
        use crate::models::ServiceType;

        for pro in ServiceProType::ALL {
            assert_eq!(
                ServiceType::from_slug(pro.slug()),
                None,
                "{pro:?} shares its slug with a bookable workshop"
            );
        }
    }
}
