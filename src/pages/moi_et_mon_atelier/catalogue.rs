use std::collections::HashMap;

use icons::ArrowRight;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::catalogue::catalogue_photos;
use crate::api::services::all_services;
use crate::components::ui::carousel::{
    Carousel, CarouselContent, CarouselIndicator, CarouselItem, CarouselNext, CarouselPrevious,
};
use crate::components::ui::service_icon::ServiceIcon;
use crate::models::{PhotoView, ServicePictures, ServiceView, section_title};

/// Renders the "Catalogue des ateliers" page.
///
/// Built from the collection rather than from a list written here, so a workshop
/// added in the admin area shows up without a deploy -- and so the catalogue can
/// no longer drift from the menu, both reading the same rows in the same order.
#[component]
pub fn CataloguePage() -> impl IntoView {
    let services = Resource::new(|| (), |()| async move { all_services().await });

    // Created here, beside the one above, rather than inside the `Suspend` below: a
    // resource born while its parent suspense is already resolving misses the
    // server's render altogether. Both are awaited in the same `Suspend` because the
    // page interleaves them -- each workshop's pictures sit inside its own rubric.
    let photos = Resource::new(|| (), |()| async move { catalogue_photos().await });

    view! {
        <Title text="Catalogue des ateliers — Bulle Créaline (E.I)"/>

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
                    // Pictures that cannot be read leave every rubric standing with
                    // its wording: a catalogue without images is still a catalogue,
                    // where a catalogue without workshops is nothing at all.
                    let pictures = photos.await.ok().unwrap_or_default();

                    // A storage failure leaves the heading and nothing under it: a
                    // visitor came for the workshops, and the failure is already
                    // logged server-side.
                    services
                        .await
                        .ok()
                        .map(|list| view! { <Sections services=list photos=pictures/> })
                })}
            </Transition>
        </div>
    }
}

/// The two sections the menu also uses, bookable workshops first.
#[component]
fn Sections(services: Vec<ServiceView>, photos: Vec<ServicePictures>) -> impl IntoView {
    // Keyed by slug rather than walked per workshop: the catalogue draws every
    // workshop, and a scan each would be quadratic for no reason.
    let photos: HashMap<String, Vec<PhotoView>> = photos
        .into_iter()
        .map(|entry| (entry.service_slug, entry.photos))
        .collect();

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
                let rubrics = group
                    .into_iter()
                    .map(|service| {
                        let pictures = photos.get(&service.slug).cloned().unwrap_or_default();

                        view! { <Workshop service=service photos=pictures/> }
                    })
                    .collect::<Vec<_>>();

                view! {
                    <section class="space-y-8">
                        <h2 class="text-2xl font-semibold tracking-tight text-heading">
                            {section_title(pro)}
                        </h2>
                        // Stacked rather than laid out in a grid, as it was while a
                        // workshop was a name and two lines: each rubric now carries
                        // a carousel, which wants the width.
                        <div class="space-y-8">{rubrics}</div>
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

/// One workshop's rubric: what it is called, what it is, and what it looks like.
#[component]
fn Workshop(service: ServiceView, photos: Vec<PhotoView>) -> impl IntoView {
    view! {
        <article class="flex flex-col gap-4 p-6 rounded-[2rem] border shadow-sm border-border bg-card">
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

            <Gallery photos=photos/>

            <a
                href=service.page_path()
                class="inline-flex gap-1 items-center pt-1 mt-auto text-sm font-medium underline-offset-4 text-primary hover:underline"
            >
                "voir +"
                <ArrowRight class="w-4 h-4"/>
            </a>
        </article>
    }
}

/// What a workshop looks like, one image at a time.
///
/// Draws nothing at all when there is nothing to show. A bookable workshop with no
/// theme yet, and one run for structures with no photo yet, are both ordinary
/// starting states rather than gaps worth a frame around them.
#[component]
fn Gallery(photos: Vec<PhotoView>) -> impl IntoView {
    if photos.is_empty() {
        return None;
    }

    Some(view! {
        // The arrows are positioned at -3rem, outside the carousel's own box: this
        // padding is what keeps them on the page rather than off its left edge.
        <div class="px-12">
            <Carousel looping=true>
                <CarouselContent>
                    // Built here rather than collected into a `Vec` above and
                    // dropped in: every piece of a carousel reads the id off a
                    // context the `Carousel` provides, so a slide built before it
                    // exists finds nothing and brings the render down.
                    {photos
                        .into_iter()
                        .map(|photo| {
                            view! {
                                <CarouselItem>
                                    <div class="overflow-hidden rounded-[1.5rem] border border-border aspect-16/9">
                                        <img
                                            src=photo.url
                                            alt=photo.alt
                                            loading="lazy"
                                            class="object-cover w-full h-full"
                                        />
                                    </div>
                                </CarouselItem>
                            }
                        })
                        .collect::<Vec<_>>()}
                </CarouselContent>
                <CarouselPrevious/>
                <CarouselNext/>
                <CarouselIndicator/>
            </Carousel>
        </div>
    })
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

    /// Pictures for one workshop, worded so that an assertion about one cannot pass
    /// on another's.
    fn pictures(slug: &str, count: usize) -> ServicePictures {
        ServicePictures {
            service_slug: slug.to_owned(),
            photos: (1..=count)
                .map(|rank| PhotoView {
                    url: format!("/media/{slug}/{rank}"),
                    alt: format!("Image {rank} de {slug}"),
                })
                .collect(),
        }
    }

    fn sections_html(services: Vec<ServiceView>, photos: Vec<ServicePictures>) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The cards carry links, and a link resolves `aria-current` against the
            // current location, which server-side is the request being rendered.
            provide_context(RequestUrl::new("/moi-et-mon-atelier/catalogue"));

            view! {
                <Router>
                    <Sections services=services photos=photos/>
                </Router>
            }
            .to_html()
        })
    }

    /// The page exists to be exhaustive: every workshop on offer is named, with
    /// its own description and a way through to its page.
    #[test]
    fn every_workshop_is_named_described_and_linked() {
        let html = sections_html(services(), Vec::new());

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
        assert!(html.contains(r#"href="/pro/en-institution""#), "{html}");
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
        assert_eq!(html.matches("<section").count(), 1, "not one section: {html}");
    }

    /// Nothing at all is a state the collection can genuinely be in, and it must
    /// read as an answer rather than as a page that failed to load.
    #[test]
    fn says_so_when_nothing_is_on_offer() {
        let html = sections_html(vec![], Vec::new());

        assert!(html.contains("Aucun atelier"), "{html}");
        assert_eq!(html.matches("<section").count(), 0, "{html}");
    }

    /// The pictures land in the rubric they belong to, whichever source the server
    /// drew them from -- the page is handed one shape either way.
    #[test]
    fn each_workshop_shows_its_own_pictures() {
        let html = sections_html(
            services(),
            vec![pictures("aperos-creatifs", 2), pictures("en-institution", 3)],
        );

        assert_eq!(html.matches("<img").count(), 5, "not one image per picture: {html}");
        assert!(html.contains("/media/aperos-creatifs/1"), "the first is missing: {html}");
        assert!(html.contains("/media/en-institution/3"), "the last is missing: {html}");
    }

    /// Two carousels for two workshops with pictures, and none for the third:
    /// without this a single carousel holding everything would pass the count above.
    #[test]
    fn a_workshop_with_pictures_gets_a_carousel_of_its_own() {
        let html = sections_html(
            services(),
            vec![pictures("aperos-creatifs", 2), pictures("en-institution", 3)],
        );

        assert_eq!(
            html.matches(r#"data-name="Carousel""#).count(),
            2,
            "not one carousel per workshop with pictures: {html}"
        );
    }

    /// The pictures of one workshop must not drift into the rubric below it, which
    /// the count above could not catch.
    #[test]
    fn the_pictures_sit_inside_the_rubric_they_belong_to() {
        let html = sections_html(services(), vec![pictures("apres-midis-creatifs", 1)]);

        let owner = html.find("Après-midis créatifs").expect("the workshop should render");
        let picture = html.find("/media/apres-midis-creatifs/1").expect("its picture too");
        let next = html.find("Ateliers en institution").expect("the next workshop too");

        assert!(
            owner < picture && picture < next,
            "the picture is out of place: {owner}, {picture}, {next}"
        );
    }

    /// A workshop with no theme and no photo yet is the ordinary starting state, not
    /// a gap worth an empty frame.
    #[test]
    fn a_workshop_with_no_picture_draws_no_carousel() {
        let html = sections_html(services(), Vec::new());

        assert!(!html.contains(r#"data-name="Carousel""#), "no carousel at all: {html}");
        assert!(!html.contains("<img"), "and certainly no image: {html}");
        assert!(html.contains("Apéros créatifs"), "the rubrics still show: {html}");
    }

    /// Every picture says what it is, the carousel being the whole of what is new
    /// here -- and a page of `alt=""` says nothing at all to anyone reading it with
    /// their ears.
    #[test]
    fn every_picture_carries_its_own_wording() {
        let html = sections_html(services(), vec![pictures("aperos-creatifs", 2)]);

        assert!(html.contains("Image 1 de aperos-creatifs"), "the first is unnamed: {html}");
        assert!(html.contains("Image 2 de aperos-creatifs"), "the second is unnamed: {html}");
        assert!(!html.contains(r#"alt="""#), "an empty alt should not ship: {html}");
    }
}
