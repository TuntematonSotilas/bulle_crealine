use std::collections::HashMap;

use icons::FileText;
use leptos::either::Either;
use leptos::prelude::*;

use crate::api::catalogue::catalogue_pdfs;
use crate::api::services::all_services;
use crate::components::seo::PageMeta;
use crate::components::ui::service_icon::ServiceIcon;
use crate::models::{ServicePdfLink, ServiceView, section_title};

/// What a workshop with no PDF yet shows in place of the link.
const NO_PDF: &str = "PDF bientôt disponible";

/// Renders the "Catalogue des ateliers" page: every workshop, each with a link to
/// its PDF.
///
/// Built from the collection rather than from a list written here, so a workshop
/// added in the admin area shows up without a deploy -- and so the catalogue can
/// no longer drift from the menu, both reading the same rows in the same order.
#[component]
pub fn CataloguePage() -> impl IntoView {
    let services = Resource::new(|| (), |()| async move { all_services().await });

    // Created here, beside the one above, rather than inside the `Suspend` below: a
    // resource born while its parent suspense is already resolving misses the
    // server's render altogether.
    let pdfs = Resource::new(|| (), |()| async move { catalogue_pdfs().await });

    view! {
        <PageMeta
            title="Catalogue des ateliers — Bulle Créaline (E.I)"
            description="Tous les ateliers de Bulle Créaline à Feurs : parents-enfants, après-midis et apéros créatifs, accompagnements individuels et interventions en institution."
            path="/moi-et-mon-atelier/catalogue"
        />

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
                    // PDFs that cannot be listed leave every workshop standing, with
                    // the "coming soon" wording: the list of workshops is still worth
                    // showing on its own.
                    let links = pdfs.await.ok().unwrap_or_default();

                    // A storage failure leaves the heading and nothing under it: a
                    // visitor came for the workshops, and the failure is already
                    // logged server-side.
                    services
                        .await
                        .ok()
                        .map(|list| view! { <Sections services=list pdfs=links/> })
                })}
            </Transition>
        </div>
    }
}

/// The two sections the menu also uses, bookable workshops first.
#[component]
fn Sections(services: Vec<ServiceView>, pdfs: Vec<ServicePdfLink>) -> impl IntoView {
    // Keyed by slug rather than scanned per workshop.
    let pdfs: HashMap<String, String> = pdfs
        .into_iter()
        .map(|link| (link.service_slug, link.url))
        .collect();

    let sections = [false, true]
        .into_iter()
        .filter_map(|pro| {
            let group: Vec<_> = services.iter().filter(|service| service.pro == pro).collect();

            // A heading over nothing is worse than no heading: an admin who
            // deletes every workshop for structures should not be left with an
            // empty "Autres Ateliers".
            (!group.is_empty()).then(|| {
                let cards = group
                    .into_iter()
                    .map(|service| {
                        let pdf = pdfs.get(&service.slug).cloned();
                        view! { <li><Workshop service=service.clone() pdf=pdf/></li> }
                    })
                    .collect::<Vec<_>>();

                view! {
                    <section class="space-y-6">
                        <h2 class="text-2xl font-semibold tracking-tight text-heading">
                            {section_title(pro)}
                        </h2>
                        <ul class="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">{cards}</ul>
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

/// One workshop: its name, and the way to its PDF.
#[component]
fn Workshop(
    service: ServiceView,
    /// Where its PDF is served, `None` when it has none yet.
    pdf: Option<String>,
) -> impl IntoView {
    let link = match pdf {
        // A new tab, so the visitor keeps the catalogue: the browser's own reader
        // opens the file, and closing it leads back here.
        Some(url) => Either::Left(view! {
            <a
                href=url
                target="_blank"
                rel="noopener"
                class="inline-flex gap-2 items-center text-sm font-medium underline-offset-4 text-primary hover:underline"
            >
                <FileText class="w-4 h-4"/>
                "Ouvrir le PDF"
            </a>
        }),
        None => Either::Right(view! { <p class="text-sm text-muted-foreground">{NO_PDF}</p> }),
    };

    view! {
        <article class="flex flex-col gap-4 p-6 h-full rounded-[2rem] border shadow-sm border-border bg-card">
            <div class="flex gap-2 items-center">
                <ServiceIcon name=service.icon class="w-5 h-5 shrink-0 text-heading-soft"/>
                <h3 class="text-lg font-semibold text-heading">{service.label}</h3>
            </div>

            <div class="mt-auto">{link}</div>
        </article>
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
            min_persons: 1,
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

    fn pdf(slug: &str) -> ServicePdfLink {
        ServicePdfLink {
            service_slug: slug.to_owned(),
            url: format!("/media/service-pdf/{slug}"),
        }
    }

    fn sections_html(services: Vec<ServiceView>, pdfs: Vec<ServicePdfLink>) -> String {
        Owner::new().with(|| view! { <Sections services=services pdfs=pdfs/> }.to_html())
    }

    /// The page exists to be exhaustive: every workshop on offer is named.
    #[test]
    fn every_workshop_is_named() {
        let html = sections_html(services(), Vec::new());

        for service in services() {
            assert!(html.contains(&service.label), "{} is not named: {html}", service.slug);
        }
    }

    /// Nothing but the name and the PDF: the description and the pictures live on
    /// the workshop's own page now.
    #[test]
    fn the_catalogue_shows_neither_description_nor_pictures() {
        let html = sections_html(services(), vec![pdf("aperos-creatifs")]);

        assert!(!html.contains("Description de"), "a description slipped in: {html}");
        assert!(!html.contains("<img"), "a picture slipped in: {html}");
    }

    /// Each link leads to its own workshop's file, in a new tab.
    #[test]
    fn a_workshop_with_a_pdf_links_to_it_in_a_new_tab() {
        let html = sections_html(services(), vec![pdf("aperos-creatifs"), pdf("en-institution")]);

        assert_eq!(html.matches("Ouvrir le PDF").count(), 2, "not one link per PDF: {html}");
        assert!(html.contains(r#"href="/media/service-pdf/aperos-creatifs""#), "{html}");
        assert!(html.contains(r#"href="/media/service-pdf/en-institution""#), "{html}");
        assert_eq!(html.matches(r#"target="_blank""#).count(), 2, "{html}");
    }

    /// The link sits in the card of the workshop it belongs to, not the next one.
    #[test]
    fn the_link_sits_under_its_own_workshop() {
        let html = sections_html(services(), vec![pdf("apres-midis-creatifs")]);

        let owner = html.find("Après-midis créatifs").expect("the workshop should render");
        let link = html.find("/media/service-pdf/apres-midis-creatifs").expect("its link too");
        let next = html.find("Ateliers en institution").expect("the next workshop too");

        assert!(owner < link && link < next, "the link is out of place: {owner}, {link}, {next}");
    }

    /// A workshop with no PDF yet is still listed, and says why there is no link.
    #[test]
    fn a_workshop_without_a_pdf_says_it_is_coming() {
        let html = sections_html(services(), vec![pdf("aperos-creatifs")]);

        assert_eq!(html.matches(NO_PDF).count(), 2, "the other two should say so: {html}");
        assert_eq!(html.matches("Ouvrir le PDF").count(), 1, "{html}");
    }

    /// Two sections, in the menu's order, so a visitor meets one taxonomy.
    #[test]
    fn the_sections_follow_the_menu() {
        let html = sections_html(services(), Vec::new());

        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle:?} is missing: {html}"))
        };

        assert!(
            at(section_title(false)) < at(section_title(true)),
            "the sections are out of order: {html}"
        );
        assert_eq!(html.matches("<section").count(), 2, "not two sections: {html}");
    }

    /// An admin who deletes every workshop of one kind should not be left with a
    /// heading standing over nothing.
    #[test]
    fn a_section_with_no_workshop_is_left_out() {
        let html = sections_html(
            vec![service("aperos-creatifs", "Apéros créatifs", false, 0)],
            Vec::new(),
        );

        assert!(html.contains(section_title(false)), "the filled section shows: {html}");
        assert!(!html.contains(section_title(true)), "the empty one does not: {html}");
    }

    /// Nothing at all must read as an answer rather than as a page that failed.
    #[test]
    fn says_so_when_nothing_is_on_offer() {
        let html = sections_html(vec![], Vec::new());

        assert!(html.contains("Aucun atelier"), "{html}");
        assert_eq!(html.matches("<section").count(), 0, "{html}");
    }
}
