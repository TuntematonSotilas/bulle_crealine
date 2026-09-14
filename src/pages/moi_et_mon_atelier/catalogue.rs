use icons::ArrowRight;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::models::{ServiceProType, ServiceType};

/// One workshop on show, drawn from either family.
///
/// The two are separate types on purpose -- only [`ServiceType`] is bookable --
/// but they carry the same three pieces of wording, so flattening them here lets
/// one card renderer serve both.
struct Workshop {
    label: &'static str,
    /// Shown on the card; empty when the group's intro already covers it.
    description: &'static str,
    path: &'static str,
}

impl Workshop {
    /// Takes its wording straight from the model, so the catalogue cannot drift
    /// from the home page's cards or from the workshop's own page.
    const fn bookable(kind: ServiceType) -> Self {
        Self {
            label: kind.label(),
            description: kind.description(),
            path: kind.page_path(),
        }
    }

    /// The workshops run for structures, which no one books online.
    const fn pro(kind: ServiceProType) -> Self {
        Self {
            label: kind.label(),
            description: kind.description(),
            path: kind.page_path(),
        }
    }
}

/// A run of workshops under one heading, mirroring how the menu files them.
struct Group {
    title: &'static str,
    workshops: &'static [Workshop],
}

/// The two workshops run with parents and children share one description word for
/// word, so it is the group that carries it. Printing it on both cards, side by
/// side, would read as a mistake.
const PARENTS_ENFANTS: [Workshop; 2] = [
    Workshop::bookable(ServiceType::ParentsEnfantsMoinsSix),
    Workshop::bookable(ServiceType::ParentsEnfantsSixADouze),
];

const ADULTES: [Workshop; 2] = [
    Workshop::bookable(ServiceType::AperosCreatifs),
    Workshop::bookable(ServiceType::ApresMidisCreatifs),
];
 
const PROFESSIONNELS: [Workshop; 3] = [
    Workshop::pro(ServiceProType::EnInstitution),
    Workshop::pro(ServiceProType::HorsLesMurs),
    Workshop::pro(ServiceProType::Individuels),
];

/// Grouped as the menu groups them, so a visitor meets the same shape twice.
const GROUPS: [Group; 3] = [
    Group {
        title: "Ateliers parents-enfants",
        workshops: &PARENTS_ENFANTS,
    },
    Group {
        title: "Ateliers adultes",
        workshops: &ADULTES,
    },
    Group {
        title: "Ateliers pour les structures",
        workshops: &PROFESSIONNELS,
    },
];

/// Renders the "Catalogue des ateliers" page.
#[component]
pub fn CataloguePage() -> impl IntoView {
    let groups = GROUPS
        .iter()
        .map(|group| {
            let cards = group
                .workshops
                .iter()
                .map(|workshop| {
                    view! {
                        <li class="flex flex-col gap-2 p-6 rounded-[2rem] border shadow-sm border-border bg-card">
                            <h3 class="text-lg font-semibold text-heading">{workshop.label}</h3>
                            {(!workshop.description.is_empty())
                                .then(|| {
                                    view! {
                                        <p class="text-sm leading-relaxed text-muted-foreground">
                                            {workshop.description}
                                        </p>
                                    }
                                })}
                            <a
                                href=workshop.path
                                class="inline-flex gap-1 items-center pt-1 mt-auto text-sm font-medium underline-offset-4 text-primary hover:underline"
                            >
                                "voir +"
                                <ArrowRight class="w-4 h-4"/>
                            </a>
                        </li>
                    }
                })
                .collect::<Vec<_>>();

            view! {
                <section class="space-y-4">
                    <h2 class="text-2xl font-semibold tracking-tight text-heading">
                        {group.title}
                    </h2>
                    <ul class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">{cards}</ul>
                </section>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <Title text="Catalogue des ateliers — Bulle Créaline"/>

        <div class="mx-auto space-y-10 max-w-6xl py-6">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Catalogue des ateliers"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
                <p class="mx-auto max-w-2xl text-lg leading-8 text-muted-foreground">
                    "Tous les ateliers proposés, à la maison, entre adultes ou au sein de votre structure."
                </p>
            </div>

            <div class="space-y-12">{groups}</div>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// The cards carry links, and a link resolves `aria-current` against the
    /// current location, which server-side is the request being rendered.
    fn page_html() -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            provide_context(RequestUrl::new("/moi-et-mon-atelier/catalogue"));

            view! {
                <Router>
                    <CataloguePage/>
                </Router>
            }
            .to_html()
        })
    }

    /// The page exists to be exhaustive: every workshop the menu offers has to be
    /// named here, the three run for structures included.
    #[test]
    fn every_workshop_is_named() {
        let html = page_html();

        for kind in ServiceType::ALL {
            assert!(html.contains(kind.label()), "{kind:?} is not named: {html}");
        }

        for kind in ServiceProType::ALL {
            assert!(html.contains(kind.label()), "{kind:?} is not named: {html}");
        }
    }

    /// A catalogue entry that leads nowhere is a dead end; each one points either
    /// at the workshop's page or, for the three still to come, at their future one.
    #[test]
    fn every_workshop_leads_somewhere() {
        let html = page_html();

        for kind in ServiceType::ALL {
            let href = format!("href=\"{}\"", kind.page_path());
            assert!(html.contains(&href), "{kind:?} has no link: {html}");
        }

        for kind in ServiceProType::ALL {
            let href = format!("href=\"{}\"", kind.page_path());
            assert!(html.contains(&href), "{kind:?} has no link: {html}");
        }
    }

    /// The four modelled workshops take their wording from `ServiceType`. Copying
    /// it into the page would let the catalogue drift from the home page's cards
    /// and from the workshop's own page, which this catches.
    #[test]
    fn the_modelled_wording_comes_from_the_model() {
        let html = page_html();

        for kind in [ServiceType::AperosCreatifs, ServiceType::ApresMidisCreatifs] {
            assert!(
                html.contains(kind.description()),
                "{kind:?} does not show the model's description: {html}"
            );
        }

        for kind in ServiceProType::ALL {
            assert!(
                html.contains(kind.description()),
                "{kind:?} does not show the model's description: {html}"
            );
        }
    }

    /// Both parents-and-children workshops return the very same sentence, so the
    /// group says it once. Twice, side by side, reads as a mistake.
    #[test]
    fn each_card_carries_its_own_description() {
        let html = page_html();
        let shared = ServiceType::ParentsEnfantsMoinsSix.description();

        assert_eq!(
            ServiceType::ParentsEnfantsSixADouze.description(),
            shared,
            "the two descriptions diverged, so this test no longer proves anything"
        );

        // Twice, once per card. The model returns the same sentence for both age
        // brackets, and showing it on each card is the accepted cost of leaving
        // `ServiceType` alone; a third occurrence would mean it also leaked into
        // a heading or an intro.
        assert_eq!(
            html.matches(shared).count(),
            2,
            "the description should appear once per card: {html}"
        );
    }

    /// Grouped as the menu groups them, and in the same order, so a visitor meets
    /// one taxonomy rather than two.
    #[test]
    fn the_groups_follow_the_order_of_the_menu() {
        let html = page_html();

        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle:?} is missing: {html}"))
        };

        let order = [
            at("Ateliers parents-enfants"),
            at("Ateliers adultes"),
            at("Ateliers pour les structures"),
        ];

        assert!(
            order.windows(2).all(|pair| pair[0] < pair[1]),
            "the groups are out of order: {order:?}"
        );
        assert_eq!(html.matches("<section").count(), 3, "not three groups: {html}");
    }
}
