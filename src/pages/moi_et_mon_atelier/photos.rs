use std::collections::HashMap;

use leptos::either::Either;
use leptos::prelude::*;

use crate::api::catalogue::catalogue_photos;
use crate::api::services::all_services;
use crate::components::seo::PageMeta;
use crate::models::{PhotoView, ServicePictures, ServiceView};

/// Every photo on the site, gathered in one album at `/moi-et-mon-atelier/photos`.
///
/// Nothing new is stored for this page: [`catalogue_photos`] already returns every
/// picture the site holds, grouped by workshop -- the themes of a bookable one, the
/// photos of one run for a structure. The catalogue shows them a few at a time in a
/// carousel; here they are all laid out at once, which is what an album is for.
///
/// A workshop with no picture is left out entirely rather than given an empty
/// heading: the album is what there is to look at, and a name with nothing under it
/// is an answer to a question nobody asked.
#[component]
pub fn PhotosPage() -> impl IntoView {
    let services = Resource::new(|| (), |()| async move { all_services().await });

    // Created here, beside the one above, rather than inside the `Suspend` below: a
    // resource born while its parent suspense is already resolving misses the
    // server's render altogether, the trap documented on
    // [`crate::pages::service::ServicePage`].
    let photos = Resource::new(|| (), |()| async move { catalogue_photos().await });

    view! {
        <PageMeta
            title="Album des ateliers — Bulle Créaline (E.I)"
            description="Les photos des ateliers créatifs de Bulle Créaline à Feurs : les thèmes proposés et les séances menées en institution."
            path="/moi-et-mon-atelier/photos"
        />

        <div class="py-6 mx-auto space-y-10 max-w-6xl">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Album de mes ateliers"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
                <p class="mx-auto max-w-2xl text-lg leading-8 text-muted-foreground">
                    "Les thèmes proposés et les séances déjà menées, atelier par atelier."
                </p>
            </div>

            <Transition fallback=|| {
                view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
            }>
                {move || Suspend::new(async move {
                    let pictures = photos.await.ok().unwrap_or_default();

                    // An album whose pictures cannot be read has nothing left to
                    // show, so it says so rather than drawing a page of headings.
                    services
                        .await
                        .ok()
                        .map(|list| view! { <Albums services=list photos=pictures/> })
                })}
            </Transition>
        </div>
    }
}

/// One block per workshop that has something to show.
#[component]
fn Albums(services: Vec<ServiceView>, photos: Vec<ServicePictures>) -> impl IntoView {
    // Keyed by slug rather than scanned per workshop: the page draws every one of
    // them, and a scan each would be quadratic for no reason.
    let mut by_slug: HashMap<String, Vec<PhotoView>> = photos
        .into_iter()
        .map(|pictures| (pictures.service_slug, pictures.photos))
        .collect();

    let albums = services
        .into_iter()
        .filter_map(|workshop| {
            let pictures = by_slug.remove(&workshop.slug).unwrap_or_default();

            (!pictures.is_empty()).then(|| view! { <Album workshop=workshop photos=pictures/> })
        })
        .collect::<Vec<_>>();

    if albums.is_empty() {
        // The ordinary starting state, not a failure: the admin has not uploaded a
        // photo yet, and saying so beats an empty page that looks broken.
        return Either::Left(view! {
            <p class="text-center text-muted-foreground">
                "Les photos des ateliers arrivent bientôt."
            </p>
        });
    }

    Either::Right(view! { <div class="space-y-12">{albums}</div> })
}

/// One workshop's pictures, under its name and linked to its page.
#[component]
fn Album(workshop: ServiceView, photos: Vec<PhotoView>) -> impl IntoView {
    let tiles = photos
        .into_iter()
        .map(|photo| {
            view! {
                <li class="overflow-hidden rounded-[2rem] border border-border aspect-16/9">
                    <img
                        src=photo.url
                        alt=photo.alt
                        loading="lazy"
                        class="object-cover w-full h-full"
                    />
                </li>
            }
        })
        .collect::<Vec<_>>();

    let label = workshop.label.clone();

    view! {
        <section>
            <h2 class="text-2xl font-bold lg:text-3xl text-heading">
                // The name leads to the workshop, so an album is a way in and not a
                // dead end: a visitor taken by a picture wants the dates, not a
                // trip back through the menu.
                <a href=workshop.page_path() class="hover:underline">{label}</a>
            </h2>
            <span class="block mt-3 w-10 h-1 rounded-full bg-heading-soft"></span>

            <ul class="grid gap-5 mt-6 sm:grid-cols-2 lg:grid-cols-3">{tiles}</ul>
        </section>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn workshop(slug: &str, label: &str) -> ServiceView {
        ServiceView {
            id: String::new(),
            slug: slug.to_owned(),
            label: label.to_owned(),
            description: String::new(),
            age: String::new(),
            steps: Vec::new(),
            icon: String::new(),
            pro: false,
            position: 0,
            min_persons: 1,
        }
    }

    fn pictures(slug: &str, count: usize) -> ServicePictures {
        ServicePictures {
            service_slug: slug.to_owned(),
            photos: (0..count)
                .map(|rank| PhotoView {
                    url: format!("/media/theme/{slug}-{rank}"),
                    alt: format!("Thème : {slug} {rank}"),
                })
                .collect(),
        }
    }

    fn albums_html(services: Vec<ServiceView>, photos: Vec<ServicePictures>) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(move || {
            // The workshop names are links, and a link resolves `aria-current`
            // against the location being rendered.
            provide_context(RequestUrl::new("/moi-et-mon-atelier/photos"));

            view! { <Router><Albums services=services photos=photos/></Router> }.to_html()
        })
    }

    /// Every picture, not a sample of them: the catalogue is where they are shown a
    /// few at a time, and this page exists because that is not always what is
    /// wanted.
    #[test]
    fn every_picture_of_every_workshop_is_laid_out() {
        let html = albums_html(
            vec![workshop("aperos-creatifs", "Apéros créatifs")],
            vec![pictures("aperos-creatifs", 4)],
        );

        assert_eq!(html.matches("<img").count(), 4, "not every picture: {html}");
        assert!(html.contains("Apéros créatifs"), "no workshop name: {html}");
    }

    /// A name with nothing under it answers a question nobody asked.
    #[test]
    fn a_workshop_with_no_picture_is_left_out() {
        let html = albums_html(
            vec![workshop("aperos-creatifs", "Apéros créatifs"), workshop("sans-photo", "Sans photo")],
            vec![pictures("aperos-creatifs", 2)],
        );

        assert!(html.contains("Apéros créatifs"), "{html}");
        assert!(!html.contains("Sans photo"), "an empty workshop was drawn: {html}");
    }

    /// An album is a way in, not a dead end: a visitor taken by a picture wants the
    /// dates, and the name above it is the way there.
    #[test]
    fn each_album_leads_to_the_workshop_it_shows() {
        let html = albums_html(
            vec![workshop("aperos-creatifs", "Apéros créatifs")],
            vec![pictures("aperos-creatifs", 1)],
        );

        assert!(
            html.contains(r#"href="/services/aperos-creatifs""#),
            "the album leads nowhere: {html}"
        );
    }

    /// Nothing stored yet is the ordinary starting state, and an empty page reads
    /// as broken where a sentence reads as early.
    #[test]
    fn an_album_with_nothing_in_it_says_so() {
        let html = albums_html(vec![workshop("aperos-creatifs", "Apéros créatifs")], Vec::new());

        assert!(html.contains("bientôt"), "nothing said about the emptiness: {html}");
        assert!(!html.contains("<img"), "a picture out of nowhere: {html}");
    }
}
