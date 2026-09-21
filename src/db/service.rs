//! The `services` collection: the kinds of workshop on offer.
//!
//! These used to be a compile-time enum. They are rows now, so the admin area can
//! add, edit and drop them without a deploy.
//!
//! What did *not* move is how a session and a booking refer to one: both still
//! store the slug, exactly the string the enum serialised to. That is what lets
//! this change land on an existing database without touching a single stored
//! document -- and it is why [`update`] never writes the slug.

use bson::oid::ObjectId;
use bson::doc;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::db::{DbError, services};
use crate::models::ServiceView;


/// A workshop document.
///
/// Every field added after the first release is `#[serde(default)]`: the
/// collection is seeded once and edited by hand from then on, so a document can
/// predate a field and must still read back rather than take the whole listing
/// down with it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The workshop's identity: what a session stores and what a URL carries.
    pub slug: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub age: String,
    #[serde(default)]
    pub steps: Vec<String>,
    /// Name of a Lucide icon, e.g. `"Palette"`. Empty means no picto.
    #[serde(default)]
    pub icon: String,
    /// Run for a structure rather than booked online; see [`ServiceView::pro`].
    #[serde(default)]
    pub pro: bool,
    #[serde(default)]
    pub position: i32,
}

impl ServiceDoc {
    pub fn to_view(&self) -> ServiceView {
        ServiceView {
            id: self.id.map(|id| id.to_hex()).unwrap_or_default(),
            slug: self.slug.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            age: self.age.clone(),
            steps: self.steps.clone(),
            icon: self.icon.clone(),
            pro: self.pro,
            position: self.position,
        }
    }
}

/// Every workshop, bookable ones first, then by rank and by name.
///
/// One order for the menu, the catalogue and the admin table, so the three cannot
/// disagree about where a workshop sits.
pub async fn list_all() -> Result<Vec<ServiceDoc>, DbError> {
    let found = services()?
        .find(doc! {})
        .sort(doc! { "pro": 1, "position": 1, "label": 1 })
        .await?
        .try_collect()
        .await?;

    Ok(found)
}

/// One workshop by id.
pub async fn find(id: ObjectId) -> Result<Option<ServiceDoc>, DbError> {
    Ok(services()?.find_one(doc! { "_id": id }).await?)
}

/// One workshop by the slug a URL or a stored session carries.
pub async fn find_by_slug(slug: &str) -> Result<Option<ServiceDoc>, DbError> {
    Ok(services()?.find_one(doc! { "slug": slug }).await?)
}

/// Stores a new workshop and returns its id.
pub async fn insert(service: &ServiceDoc) -> Result<ObjectId, DbError> {
    let inserted = services()?.insert_one(service).await?;

    inserted
        .inserted_id
        .as_object_id()
        .ok_or(DbError::MalformedId)
}

/// Overwrites everything about a workshop except its slug.
///
/// The slug is left alone on purpose: sessions and bookings are filed under it,
/// and changing it here would leave them pointing at a workshop that no longer
/// answers to that name.
pub async fn update(id: ObjectId, service: &ServiceDoc) -> Result<(), DbError> {
    let update = doc! {
        "$set": {
            "label": &service.label,
            "description": &service.description,
            "age": &service.age,
            "steps": &service.steps,
            "icon": &service.icon,
            "pro": service.pro,
            "position": service.position,
        }
    };

    services()?.update_one(doc! { "_id": id }, update).await?;

    Ok(())
}

/// Drops a workshop.
///
/// Whether that is allowed is decided a layer up, by counting the sessions filed
/// under its slug.
pub async fn delete(id: ObjectId) -> Result<(), DbError> {
    services()?.delete_one(doc! { "_id": id }).await?;

    Ok(())
}

/// Fills an empty collection with the workshops the site used to carry in code.
///
/// Runs at startup and does nothing once anything is stored, so it is a first-boot
/// migration rather than a reset. Keyed on "is the collection empty" rather than
/// on a stored marker, which would be one more thing to keep in step; the one case
/// that gets wrong is an admin who deletes every workshop on purpose and then
/// restarts, and that is the cheapest wrong answer on offer.
///
/// Takes the collection rather than reaching for [`services`]: this runs from
/// `ensure_indexes`, before the shared handle is published, so the global accessor
/// would report the database as unconfigured.
pub async fn seed_if_empty(collection: &mongodb::Collection<ServiceDoc>) -> Result<(), DbError> {
    if collection.count_documents(doc! {}).await? > 0 {
        return Ok(());
    }

    // Two instances booting together can both find the collection empty, and the
    // unique index on the slug then turns the second one down. That is the index
    // doing its job, not a failure worth reporting: a failed `init` would take the
    // whole database offline over a race whose outcome is already correct.
    match collection
        .insert_many(default_services())
        .await
        .map_err(DbError::from)
    {
        Ok(_) | Err(DbError::Duplicate) => Ok(()),
        Err(error) => Err(error),
    }
}

/// How a session of a workshop run at the atelier unfolds.
///
/// Was copied word for word into three page components; seeded onto the four
/// bookable workshops so the admin can now tell them apart.
fn default_steps() -> Vec<String> {
    [
        "Accueil et présentation de l'atelier",
        "Petit exercice créatif simple",
        "Explication du thème et du matériel",
        "Découverte des matériaux et des techniques par les participants",
        "Choix du projet par le participant (accompagnement possible)",
        "Réalisation du projet",
        "Temps de partage et d'échange autour des créations",
        "Clôture de la séance et prise de retours",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// The seven workshops the site carried as two enums, with the wording, the slugs
/// and the order they already had.
///
/// The slugs matter most: they are what every session and booking already stored
/// points at, so changing one here would orphan real data.
fn default_services() -> Vec<ServiceDoc> {
    /// Both parents-and-children workshops shared one sentence, and still do.
    const PARENTS_ENFANTS: &str = "Nos ateliers parents-enfants offrent un espace de création \
                                   commune, favorisant un temps de partage loin des impératifs \
                                   quotidiens.";

    let bookable = |slug: &str, label: &str, description: &str, age: &str, icon: &str, position| {
        ServiceDoc {
            id: None,
            slug: slug.to_owned(),
            label: label.to_owned(),
            description: description.to_owned(),
            age: age.to_owned(),
            steps: default_steps(),
            icon: icon.to_owned(),
            pro: false,
            position,
        }
    };

    // No steps: how a workshop for a structure unfolds is agreed with that
    // structure, so seeding the atelier's own run-through would be a guess.
    let pro = |slug: &str, label: &str, description: &str, icon: &str, position| ServiceDoc {
        id: None,
        slug: slug.to_owned(),
        label: label.to_owned(),
        description: description.to_owned(),
        age: String::new(),
        steps: Vec::new(),
        icon: icon.to_owned(),
        pro: true,
        position,
    };

    vec![
        bookable(
            "parents-enfants-moins-six",
            "Ateliers parents-enfants (moins de 6 ans)",
            PARENTS_ENFANTS,
            "De 0 à 6 ans",
            "Baby",
            0,
        ),
        bookable(
            "parents-enfants-six-a-douze",
            "Ateliers parents-enfants (6 à 12 ans)",
            PARENTS_ENFANTS,
            "De 6 à 12 ans",
            "Users",
            1,
        ),
        bookable(
            "aperos-creatifs",
            "Apéros créatifs (adultes)",
            "Une soirée entre adultes autour d'un verre et d'un projet créatif, sans prérequis : \
             on vient souffler et on repart avec sa création.",
            "À partir de 18 ans",
            "Wine",
            2,
        ),
        bookable(
            "apres-midis-creatifs",
            "Ateliers après-midi créatifs (adultes)",
            "Découvrez nos ateliers créatifs conçus pour tous les âges et tous les niveaux.",
            "À partir de 13 ans",
            "Palette",
            3,
        ),
        pro(
            "en-institution",
            "Ateliers en institution",
            "Des ateliers créatifs menés au sein de votre structure, adaptés au rythme et aux \
             capacités de chaque groupe.",
            "Building2",
            0,
        ),
        pro(
            "hors-les-murs",
            "Ateliers hors les murs",
            "L'atelier se déplace : le matériel et l'accompagnement viennent jusqu'à vous, sur le \
             lieu de votre choix.",
            "Car",
            1,
        ),
        pro(
            "individuels",
            "Ateliers individuels",
            "Un accompagnement en tête-à-tête, au rythme de la personne, pour qui préfère créer \
             sans le groupe.",
            "HandHeart",
            2,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SERVICE_ICONS, is_valid_slug};

    /// These slugs are already stored on every session and booking the site has
    /// taken. A change here would leave those rows pointing at nothing.
    #[test]
    fn the_seed_keeps_the_slugs_the_enums_used() {
        let slugs: Vec<_> = default_services()
            .iter()
            .map(|service| service.slug.clone())
            .collect();

        assert_eq!(
            slugs,
            [
                "parents-enfants-moins-six",
                "parents-enfants-six-a-douze",
                "aperos-creatifs",
                "apres-midis-creatifs",
                "en-institution",
                "hors-les-murs",
                "individuels",
            ]
        );
    }

    /// A seeded row goes straight into the collection without passing through the
    /// form's checks, so it has to satisfy them on its own.
    #[test]
    fn every_seeded_workshop_would_pass_the_form() {
        for service in default_services() {
            assert!(is_valid_slug(&service.slug), "{} is not a valid slug", service.slug);
            assert!(!service.label.trim().is_empty(), "{} has no label", service.slug);
            assert!(
                !service.description.trim().is_empty(),
                "{} has no description",
                service.slug
            );
            assert!(
                SERVICE_ICONS.iter().any(|(name, _)| *name == service.icon),
                "{} uses an icon the picker does not offer: {}",
                service.slug,
                service.icon
            );
        }
    }

    /// A continued string literal keeps the indentation of the following line
    /// unless it is written with a trailing `\`, which shows up as a run of spaces
    /// in the middle of a sentence.
    #[test]
    fn no_seeded_wording_kept_its_indentation() {
        for service in default_services() {
            assert!(
                !service.description.contains("  "),
                "{} has a run of spaces: {:?}",
                service.slug,
                service.description
            );
        }
    }

    /// The listing sorts on `(pro, position)`, so two workshops sharing both would
    /// swap places between two loads of the same page.
    #[test]
    fn the_seed_ranks_each_section_without_a_tie() {
        for pro in [false, true] {
            let mut ranks: Vec<_> = default_services()
                .iter()
                .filter(|service| service.pro == pro)
                .map(|service| service.position)
                .collect();

            let count = ranks.len();
            ranks.sort_unstable();
            ranks.dedup();

            assert_eq!(ranks.len(), count, "two workshops share a rank (pro = {pro})");
        }
    }

    /// The four workshops run at the atelier follow one run-through; the ones run
    /// for a structure have none to seed.
    #[test]
    fn only_the_bookable_workshops_are_seeded_with_steps() {
        for service in default_services() {
            assert_eq!(
                service.steps.is_empty(),
                service.pro,
                "{} has the wrong kind of run-through",
                service.slug
            );
        }
    }
}
