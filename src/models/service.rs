use serde::{Deserialize, Serialize};

/// A kind of workshop, as the browser sees it.
///
/// Workshops used to be a compile-time enum. They are now rows the admin area
/// creates, edits and drops, so every page that names one reads it from here.
///
/// The identity is [`slug`](Self::slug), not the id: it is what `sessions` and
/// `bookings` store, what `/booking/<slug>` carries, and what the public pages
/// resolve. That is why the admin form lets a slug be chosen once and never
/// changed -- renaming it would orphan every session already recorded under it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceView {
    /// Hex form of the Mongo `ObjectId`, which is what the admin form posts back.
    pub id: String,
    /// URL segment, and the value stored on a session and on a booking.
    pub slug: String,
    /// Name shown to visitors.
    pub label: String,
    /// One or two sentences introducing this kind of workshop.
    pub description: String,
    /// Who the workshop is for, e.g. `"De 0 à 6 ans"`. May be empty.
    pub age: String,
    /// How a session unfolds, one line per step. May be empty.
    pub steps: Vec<String>,
    /// Name of the Lucide icon standing for this workshop, e.g. `"Palette"`.
    ///
    /// Stored as the name rather than as a drawing: the icons ship with the
    /// binary, so only the reference has to travel. Empty means no picto.
    pub icon: String,
    /// Whether this workshop is run for a structure rather than booked online.
    ///
    /// Decides both which menu section it appears under and whether it is
    /// bookable at all: a structure agrees on its dates directly, so there is no
    /// session to offer and no `/booking/<slug>` to reach.
    pub pro: bool,
    /// Rank inside its section; ties fall back to the label.
    pub position: i32,
}

impl ServiceView {
    /// Path of the public page describing this workshop.
    pub fn page_path(&self) -> String {
        format!("{}/{}", section_prefix(self.pro), self.slug)
    }

    /// Where to book it, or `None` when it is run for a structure.
    pub fn booking_path(&self) -> Option<String> {
        (!self.pro).then(|| format!("/booking/{}", self.slug))
    }
}

/// Route prefix of a section: the two are separate so a visitor can tell from the
/// URL alone whether a workshop is one they can sign up for.
pub fn section_prefix(pro: bool) -> &'static str {
    if pro { "/pro" } else { "/services" }
}

/// Heading the menu and the catalogue file a workshop under.
pub fn section_title(pro: bool) -> &'static str {
    if pro { "Autres Ateliers" } else { "Ateliers à domicile" }
}

/// Whether a slug is safe to put in a URL and to store.
///
/// Deliberately narrow: lowercase ASCII, digits and single inner hyphens. A slug
/// travels through a route segment, a Mongo filter and an `href`, and anything
/// needing escaping in one of the three is more trouble than it is worth.
pub fn is_valid_slug(candidate: &str) -> bool {
    !candidate.is_empty()
        && !candidate.starts_with('-')
        && !candidate.ends_with('-')
        && !candidate.contains("--")
        && candidate.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

/// Turns a label into a slug, to prefill the field rather than to replace it.
///
/// Accented letters are folded rather than dropped, so "Apéros créatifs" gives
/// "aperos-creatifs" and not "ap-ros-cr-atifs".
pub fn slugify(label: &str) -> String {
    let mut slug = String::with_capacity(label.len());

    for character in label.chars() {
        match fold(character) {
            // One hyphen per run of anything unusable, and never a leading one.
            "" => {
                if !slug.is_empty() && !slug.ends_with('-') {
                    slug.push('-');
                }
            }
            folded => slug.push_str(folded),
        }
    }

    slug.trim_end_matches('-').to_owned()
}

/// The ASCII a character contributes to a slug, or `""` when it contributes none.
///
/// The accented letters are spelled out rather than computed: stripping diacritics
/// properly would mean pulling in Unicode normalization for the handful of letters
/// French actually uses.
///
/// Lowercased through `char::to_lowercase` rather than `to_ascii_lowercase`, which
/// leaves `'À'` and `'Œ'` untouched and would drop every accented capital.
fn fold(character: char) -> &'static str {
    let lowered = character.to_lowercase().next().unwrap_or(character);

    match lowered {
        'a' => "a",
        'b' => "b",
        'c' => "c",
        'd' => "d",
        'e' => "e",
        'f' => "f",
        'g' => "g",
        'h' => "h",
        'i' => "i",
        'j' => "j",
        'k' => "k",
        'l' => "l",
        'm' => "m",
        'n' => "n",
        'o' => "o",
        'p' => "p",
        'q' => "q",
        'r' => "r",
        's' => "s",
        't' => "t",
        'u' => "u",
        'v' => "v",
        'w' => "w",
        'x' => "x",
        'y' => "y",
        'z' => "z",
        '0' => "0",
        '1' => "1",
        '2' => "2",
        '3' => "3",
        '4' => "4",
        '5' => "5",
        '6' => "6",
        '7' => "7",
        '8' => "8",
        '9' => "9",
        'à' | 'â' | 'ä' => "a",
        'ç' => "c",
        'é' | 'è' | 'ê' | 'ë' => "e",
        'î' | 'ï' => "i",
        'ô' | 'ö' => "o",
        'ù' | 'û' | 'ü' => "u",
        'ÿ' => "y",
        'œ' => "oe",
        'æ' => "ae",
        _ => "",
    }
}

/// The icons the admin can pick from, as `(name, what it looks like)`.
///
/// A hand-written shortlist rather than the 1635 the icon crate ships: they
/// cannot be enumerated at runtime -- `IconType` derives no iterator -- and a
/// list that long would make a worse picker than no picker at all.
///
/// The name is what goes in [`ServiceView::icon`] and must match the icon crate's
/// spelling exactly; a test checks each one still resolves.
pub const SERVICE_ICONS: [(&str, &str); 29] = [
    ("Palette", "Palette"),
    ("Brush", "Pinceau"),
    ("PaintBucket", "Pot de peinture"),
    ("Pencil", "Crayon"),
    ("Scissors", "Ciseaux"),
    ("Shapes", "Formes"),
    ("Sparkles", "Étincelles"),
    ("Baby", "Bébé"),
    ("Users", "Groupe"),
    ("UsersRound", "Petit groupe"),
    ("HandHeart", "Main et cœur"),
    ("Heart", "Cœur"),
    ("Smile", "Sourire"),
    ("Wine", "Verre"),
    ("Coffee", "Tasse"),
    ("House", "Maison"),
    ("Building2", "Bâtiment"),
    ("Car", "Voiture"),
    ("Flower2", "Fleur"),
    ("Leaf", "Feuille"),
    ("Sun", "Soleil"),
    ("Star", "Étoile"),
    ("BookOpen", "Livre ouvert"),
    ("Gift", "Cadeau"),
    ("Music", "Musique"),
    ("Camera", "Appareil photo"),
    ("Drama", "Masques"),
    ("Puzzle", "Puzzle"),
    ("Accessibility", "Accessibilité"),
];

/// The French wording of an icon, for the dropdown's closed state.
pub fn icon_label(name: &str) -> Option<&'static str> {
    SERVICE_ICONS
        .iter()
        .find(|(icon, _)| *icon == name)
        .map(|(_, label)| *label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(pro: bool) -> ServiceView {
        ServiceView {
            id: "651d1f0a0000000000000001".to_owned(),
            slug: "aperos-creatifs".to_owned(),
            label: "Apéros créatifs".to_owned(),
            description: "Une soirée entre adultes.".to_owned(),
            age: "À partir de 18 ans".to_owned(),
            steps: vec!["Accueil".to_owned()],
            icon: "Wine".to_owned(),
            pro,
            position: 0,
        }
    }

    /// The two sections live under different prefixes, so a URL says on its own
    /// whether the workshop behind it can be booked.
    #[test]
    fn the_page_path_follows_the_section() {
        assert_eq!(service(false).page_path(), "/services/aperos-creatifs");
        assert_eq!(service(true).page_path(), "/pro/aperos-creatifs");
    }

    /// A workshop run for a structure agrees on its dates directly: offering a
    /// booking link would send a visitor to a page with nothing to pick.
    #[test]
    fn only_a_bookable_workshop_offers_a_booking_path() {
        assert_eq!(
            service(false).booking_path().as_deref(),
            Some("/booking/aperos-creatifs")
        );
        assert_eq!(service(true).booking_path(), None);
    }

    #[test]
    fn accepts_the_slugs_already_in_use() {
        for slug in [
            "aperos-creatifs",
            "parents-enfants-moins-six",
            "en-institution",
            "atelier2",
        ] {
            assert!(is_valid_slug(slug), "{slug} should be accepted");
        }
    }

    /// Anything needing escaping in a URL, in an `href` or in a Mongo filter has
    /// to be turned down at the door rather than handled everywhere downstream.
    #[test]
    fn rejects_anything_that_would_need_escaping() {
        for slug in [
            "",
            "-leading",
            "trailing-",
            "double--hyphen",
            "Majuscule",
            "avec espace",
            "accentué",
            "slash/inside",
            "point.inside",
        ] {
            assert!(!is_valid_slug(slug), "{slug:?} should be refused");
        }
    }

    /// The prefill is only useful if what it produces passes the check that
    /// follows it; an accented label is the ordinary case here, not the edge one.
    #[test]
    fn slugify_produces_a_valid_slug() {
        let cases = [
            ("Apéros créatifs", "aperos-creatifs"),
            (
                "Ateliers parents-enfants (moins de 6 ans)",
                "ateliers-parents-enfants-moins-de-6-ans",
            ),
            ("Ateliers hors les murs", "ateliers-hors-les-murs"),
            // Accented capitals: `to_ascii_lowercase` leaves these alone, and they
            // would then fall through as unusable and vanish from the slug.
            ("Œuvre collective", "oeuvre-collective"),
            ("ÉVEIL À L'ART", "eveil-a-l-art"),
        ];

        for (label, expected) in cases {
            let slug = slugify(label);

            assert_eq!(slug, expected, "{label:?}");
            assert!(is_valid_slug(&slug), "{label:?} produced an invalid slug: {slug}");
        }
    }

    /// Nothing usable must not produce a slug that then fails the check: the form
    /// asks again rather than posting an empty segment.
    #[test]
    fn slugify_gives_up_rather_than_producing_something_invalid() {
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("!!!"), "");
    }

    /// The names are what reach the icon crate, so a typo here would render an
    /// empty `<svg>` on a page rather than fail anywhere visible.
    #[test]
    fn every_offered_icon_resolves() {
        use std::str::FromStr;

        for (name, label) in SERVICE_ICONS {
            assert!(
                icons::common::IconType::from_str(name).is_ok(),
                "{name} is not an icon the crate knows"
            );
            assert!(!label.is_empty(), "{name} has no wording");
        }
    }

    #[test]
    fn icons_are_offered_once_each() {
        let mut names: Vec<_> = SERVICE_ICONS.iter().map(|(name, _)| *name).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), count, "an icon is offered twice");
    }

    #[test]
    fn icon_label_answers_for_what_is_offered_and_only_that() {
        assert_eq!(icon_label("Wine"), Some("Verre"));
        assert_eq!(icon_label("NotAnIcon"), None);
    }
}
