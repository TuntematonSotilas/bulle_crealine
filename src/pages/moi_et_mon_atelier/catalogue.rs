use icons::ArrowRight;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::services::all_services;
use crate::components::ui::service_icon::ServiceIcon;
use crate::models::{ServiceView, section_title};

/// Renders the "Catalogue des ateliers" page.
///
/// Built from the collection rather than from a list written here, so a workshop
/// added in the admin area shows up without a deploy -- and so the catalogue can
/// no longer drift from the menu, both reading the same rows in the same order.
#[component]
pub fn CataloguePage() -> impl IntoView {
    let services = Resource::new(|| (), |()| async move { all_services().await });

    view! {
        <Title text="Catalogue des ateliers — Bulle Créaline"/>

        <div class="py-6 mx-auto space-y-10 max-w-6xl">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Catalogue des ateliers"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
                <p class="mx-auto max-w-2xl text-lg leading-8 text-muted-foreground">
                    "Tous les ateliers proposés, à la maison, entre adultes ou au sein de votre structure."
                </p>
            </div>

            <Transition fallback=|| {
                view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
            }>
                {move || Suspend::new(async move {
                    // A storage failure leaves the heading and nothing under it: a
                    // visitor came for the workshops, and the failure is already
                    // logged server-side.
                    services.await.ok().map(|list| view! { <Sections services=list/> })
                })}
            </Transition>
        </div>
    }
}

/// The two sections the menu also uses, bookable workshops first.
#[component]
fn Sections(services: Vec<ServiceView>) -> impl IntoView {
    let sections = [false, true]
        .into_iter()
        .filter_map(|pro| {
            let group: Vec<_> = services
                .iter()
                .filter(|service| service.pro == pro)
                .cloned()
                .collect();

            // A heading over nothing is worse than no heading: an admin who
            // deletes every workshop for structures should not be left with an
            // empty "Autres Ateliers".
            (!group.is_empty()).then(|| {
                view! {
                    <section class="space-y-4">
                        <h2 class="text-2xl font-semibold tracking-tight text-heading">
                            {section_title(pro)}
                        </h2>
                        <ul class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
                            {group
                                .into_iter()
                                .map(|service| view! { <WorkshopCard service=service/> })
                                .collect::<Vec<_>>()}
                        </ul>
                    </section>
                }
            })
        })
        .collect::<Vec<_>>();

    if sections.is_empty() {
        return Either::Left(view! {
            <p class="text-center text-muted-foreground">
                "Aucun atelier n'est proposé pour le moment."
            </p>
        });
    }

    Either::Right(view! { <div class="space-y-12">{sections}</div> })
}

#[component]
fn WorkshopCard(service: ServiceView) -> impl IntoView {
    view! {
        <li class="flex flex-col gap-2 p-6 rounded-[2rem] border shadow-sm border-border bg-card">
            <div class="flex gap-2 items-center">
                <ServiceIcon
                    name=service.icon.clone()
                    class="w-5 h-5 shrink-0 text-heading-soft"
                />
                <h3 class="text-lg font-semibold text-heading">{service.label.clone()}</h3>
            </div>
            {(!service.description.is_empty())
                .then(|| {
                    view! {
                        <p class="text-sm leading-relaxed text-muted-foreground">
                            {service.description.clone()}
                        </p>
                    }
                })}
            <a
                href=service.page_path()
                class="inline-flex gap-1 items-center pt-1 mt-auto text-sm font-medium underline-offset-4 text-primary hover:underline"
            >
                "voir +"
                <ArrowRight class="w-4 h-4"/>
            </a>
        </li>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn service(slug: &str, label: &str, pro: bool, position: i32) -> ServiceView {
        ServiceView {
            id: format!("651d1f0a00000000000000{position:02}"),
            slug: slug.to_owned(),
            label: label.to_owned(),
            description: format!("Description de {label}."),
            age: String::new(),
            steps: Vec::new(),
            icon: "Palette".to_owned(),
            pro,
            position,
        }
    }

    /// Bookable ones first, then those run for structures: the order the server
    /// sorts them into, which is also the order the menu shows.
    fn services() -> Vec<ServiceView> {
        vec![
            service("aperos-creatifs", "Apéros créatifs", false, 0),
            service("apres-midis-creatifs", "Après-midis créatifs", false, 1),
            service("en-institution", "Ateliers en institution", true, 0),
        ]
    }

    fn sections_html(services: Vec<ServiceView>) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The cards carry links, and a link resolves `aria-current` against the
            // current location, which server-side is the request being rendered.
            provide_context(RequestUrl::new("/moi-et-mon-atelier/catalogue"));

            view! {
                <Router>
                    <Sections services=services/>
                </Router>
            }
            .to_html()
        })
    }

    /// The page exists to be exhaustive: every workshop on offer is named, with
    /// its own description and a way through to its page.
    #[test]
    fn every_workshop_is_named_described_and_linked() {
        let html = sections_html(services());

        for service in services() {
            assert!(html.contains(&service.label), "{} is not named: {html}", service.slug);
            assert!(
                html.contains(&service.description),
                "{} has no description: {html}",
                service.slug
            );
            assert!(
                html.contains(&format!("href=\"{}\"", service.page_path())),
                "{} has no link: {html}",
                service.slug
            );
        }
    }

    /// Two sections, in the menu's order, so a visitor meets one taxonomy rather
    /// than two -- and workshops for structures point at `/pro/`, not `/services/`.
    #[test]
    fn the_sections_follow_the_menu() {
        let html = sections_html(services());

        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle:?} is missing: {html}"))
        };

        assert!(
            at(section_title(false)) < at(section_title(true)),
            "the sections are out of order: {html}"
        );
        assert_eq!(html.matches("<section").count(), 2, "not two sections: {html}");
        assert!(html.contains(r#"href="/pro/en-institution""#), "{html}");
    }

    /// An admin who deletes every workshop of one kind should not be left with a
    /// heading standing over nothing.
    #[test]
    fn a_section_with_no_workshop_is_left_out() {
        let html = sections_html(vec![service("aperos-creatifs", "Apéros créatifs", false, 0)]);

        assert!(html.contains(section_title(false)), "the filled section shows: {html}");
        assert!(!html.contains(section_title(true)), "the empty one does not: {html}");
        assert_eq!(html.matches("<section").count(), 1, "not one section: {html}");
    }

    /// Nothing at all is a state the collection can genuinely be in, and it must
    /// read as an answer rather than as a page that failed to load.
    #[test]
    fn says_so_when_nothing_is_on_offer() {
        let html = sections_html(vec![]);

        assert!(html.contains("Aucun atelier"), "{html}");
        assert_eq!(html.matches("<section").count(), 0, "{html}");
    }
}
