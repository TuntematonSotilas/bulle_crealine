use serde::{Deserialize, Serialize};

/// The kind of workshop a session belongs to.
///
/// The serde representation is the URL slug, so the value stored in Mongo, the
/// value posted by a form and the segment in `/booking/<slug>` are all the same
/// string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    CreatifsPourTous,
    ParentsEnfants,
    AperosCreatifs,
    HorsLesMurs,
    EnInstitution,
    Individuels,
}

impl ServiceType {
    /// Every variant, in the order the admin form lists them.
    pub const ALL: [Self; 6] = [
        Self::CreatifsPourTous,
        Self::ParentsEnfants,
        Self::AperosCreatifs,
        Self::HorsLesMurs,
        Self::EnInstitution,
        Self::Individuels,
    ];

    /// URL slug, and the value stored in Mongo.
    ///
    /// Must stay in step with the `serde` renaming above; a test guards that.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::CreatifsPourTous => "creatifs-pour-tous",
            Self::ParentsEnfants => "parents-enfants",
            Self::AperosCreatifs => "aperos-creatifs",
            Self::HorsLesMurs => "hors-les-murs",
            Self::EnInstitution => "en-institution",
            Self::Individuels => "individuels",
        }
    }

    /// Name shown to visitors.
    pub const fn label(self) -> &'static str {
        match self {
            Self::CreatifsPourTous => "Ateliers créatifs pour tous",
            Self::ParentsEnfants => "Ateliers parents-enfants",
            Self::AperosCreatifs => "Apéros créatifs (adultes)",
            Self::HorsLesMurs => "Ateliers hors les murs",
            Self::EnInstitution => "Ateliers en institution",
            Self::Individuels => "Ateliers individuels",
        }
    }

    /// Path of the public page describing this kind of workshop.
    pub const fn page_path(self) -> &'static str {
        match self {
            Self::CreatifsPourTous => "/services/creatifs-pour-tous",
            Self::ParentsEnfants => "/services/parents-enfants",
            Self::AperosCreatifs => "/services/aperos-creatifs",
            Self::HorsLesMurs => "/services/hors-les-murs",
            Self::EnInstitution => "/services/en-institution",
            Self::Individuels => "/services/individuels",
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

    #[test]
    fn page_paths_point_at_the_slug() {
        for kind in ServiceType::ALL {
            assert_eq!(kind.page_path(), format!("/services/{}", kind.slug()));
        }
    }
}
