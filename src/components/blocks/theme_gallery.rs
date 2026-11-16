//! The themes of a workshop, as the catalogue and the workshop's own page show them.

use leptos::prelude::*;

use crate::models::PhotoView;

/// Splits a workshop's pictures into one group per theme, in the order they come.
///
/// Consecutive pictures of the same theme share a group; the server hands each
/// theme's pictures together. A workshop's own photos belong to no theme and form a
/// single group whose name is empty.
fn by_theme(photos: Vec<PhotoView>) -> Vec<(String, Vec<PhotoView>)> {
    let mut groups: Vec<(String, Vec<PhotoView>)> = Vec::new();

    for photo in photos {
        match groups.last_mut() {
            Some((theme, pictures)) if *theme == photo.theme => pictures.push(photo),
            _ => groups.push((photo.theme.clone(), vec![photo])),
        }
    }

    groups
}

/// Pictures grouped by theme, laid out in columns as the album lays out its own.
///
/// Each theme is a cell of the grid, its name over its pictures. Pictures that
/// belong to no theme -- a workshop run for structures has none -- take a cell
/// each, with no heading.
#[component]
pub fn ThemeGallery(photos: Vec<PhotoView>) -> impl IntoView {
    let cells = by_theme(photos)
        .into_iter()
        .flat_map(|(theme, pictures)| {
            if theme.is_empty() {
                pictures
                    .into_iter()
                    .map(|photo| view! { <li><Picture photo=photo/></li> }.into_any())
                    .collect::<Vec<_>>()
            } else {
                let stacked = pictures
                    .into_iter()
                    .map(|photo| view! { <Picture photo=photo/> })
                    .collect::<Vec<_>>();

                vec![
                    view! {
                        <li class="flex flex-col gap-3">
                            <h4 class="text-base font-semibold text-heading">{theme}</h4>
                            {stacked}
                        </li>
                    }
                    .into_any(),
                ]
            }
        })
        .collect::<Vec<_>>();

    view! { <ul class="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">{cells}</ul> }
}

/// One picture, framed as on the album page.
#[component]
fn Picture(photo: PhotoView) -> impl IntoView {
    view! {
        <div class="overflow-hidden rounded-[1.5rem] border border-border aspect-16/9">
            <img src=photo.url alt=photo.alt loading="lazy" class="object-cover w-full h-full"/>
        </div>
    }
}

/// The "Les thèmes" section of a bookable workshop's page: every theme it offers,
/// whether or not a date is on the calendar right now.
///
/// Draws nothing when the workshop has no theme yet, which is an ordinary starting
/// state rather than a gap worth a heading over it.
#[component]
pub fn ServiceThemes(photos: Vec<PhotoView>) -> impl IntoView {
    (!photos.is_empty()).then(|| {
        // The same heading and rule as the page's other sections.
        view! {
            <section class="mx-auto mt-12 max-w-6xl">
                <h2 class="text-2xl font-bold lg:text-3xl text-heading">"Les thèmes"</h2>
                <span class="block mt-3 w-10 h-1 rounded-full bg-heading-soft"></span>

                <div class="mt-6">
                    <ThemeGallery photos=photos/>
                </div>
            </section>
        }
    })
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn photo(theme: &str, rank: usize) -> PhotoView {
        PhotoView {
            url: format!("/media/theme/{rank}"),
            alt: format!("Thème : {theme}"),
            theme: theme.to_owned(),
        }
    }

    fn themes_html(photos: Vec<PhotoView>) -> String {
        Owner::new().with(|| view! { <ServiceThemes photos=photos/> }.to_html())
    }

    #[test]
    fn consecutive_pictures_of_one_theme_form_one_group() {
        let groups = by_theme(vec![photo("Aquarelle", 1), photo("Aquarelle", 2), photo("Mosaïque", 3)]);

        let names: Vec<_> = groups.iter().map(|(name, photos)| (name.as_str(), photos.len())).collect();
        assert_eq!(names, vec![("Aquarelle", 2), ("Mosaïque", 1)]);
    }

    #[test]
    fn the_section_names_every_theme_over_its_picture() {
        let html = themes_html(vec![photo("Aquarelle", 1), photo("Mosaïque", 2)]);

        assert!(html.contains("Les thèmes"), "no heading: {html}");
        assert_eq!(html.matches("<h4").count(), 2, "not one name per theme: {html}");
        assert_eq!(html.matches("<img").count(), 2, "not one picture per theme: {html}");
        assert!(
            html.find("Aquarelle").unwrap() < html.find("/media/theme/1").unwrap(),
            "the name should stand over its picture: {html}"
        );
    }

    /// A workshop with no theme yet gets no heading over nothing.
    #[test]
    fn a_workshop_without_themes_draws_no_section() {
        let html = themes_html(Vec::new());

        assert!(!html.contains("Les thèmes"), "an empty section was drawn: {html}");
    }
}
