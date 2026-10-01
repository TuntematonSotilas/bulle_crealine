use leptos::prelude::*;

use crate::models::ServicePhotoView;

/// What a workshop run for a structure looks like, under its presentation.
///
/// A sibling of [`crate::components::blocks::service_block::ServiceBlock`] rather
/// than a part of it, because the photos are fetched and the block is not: a
/// `Resource` born inside an already-resolving suspense misses the server's render
/// altogether. The page owns the fetch; this draws what it got.
///
/// Draws nothing at all when there is no photo. A workshop with an empty gallery is
/// the ordinary starting state, not a gap worth a heading over it -- unlike a
/// bookable workshop with no date, which has something to say about that.
#[component]
pub fn ServiceGallery(photos: Vec<ServicePhotoView>) -> impl IntoView {
    if photos.is_empty() {
        return None;
    }

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

    // The same heading and rule as the schedule a bookable workshop carries, so the
    // two kinds of page do not end in two different styles of section.
    Some(view! {
        <section class="mx-auto mt-12 max-w-6xl">
            <h2 class="text-2xl font-bold lg:text-3xl text-heading">"En images"</h2>
            <span class="block mt-3 w-10 h-1 rounded-full bg-heading-soft"></span>

            <ul class="grid gap-5 mt-6 sm:grid-cols-2 lg:grid-cols-3">{tiles}</ul>
        </section>
    })
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// Two photos whose URLs and wording share nothing, so an assertion about one
    /// cannot pass on the other.
    fn photos() -> Vec<ServicePhotoView> {
        ["651d1f0a0000000000000001", "651d1f0a0000000000000002"]
            .iter()
            .enumerate()
            .map(|(rank, id)| ServicePhotoView {
                id: (*id).to_owned(),
                url: format!("/media/service-photo/{id}"),
                alt: format!("Ateliers en institution — photo {}", rank + 1),
            })
            .collect()
    }

    fn gallery_html(photos: Vec<ServicePhotoView>) -> String {
        Owner::new().with(|| view! { <ServiceGallery photos=photos/> }.to_html())
    }

    #[test]
    fn draws_every_photo_it_was_given() {
        let html = gallery_html(photos());

        assert_eq!(html.matches("<img").count(), 2, "not one image per photo: {html}");
        for photo in photos() {
            assert!(html.contains(&photo.url), "{} is missing: {html}", photo.url);
        }
    }

    /// The gallery is the whole of what is new on these pages, so an image with no
    /// alternative text is the page saying nothing at all to anyone reading it with
    /// their ears.
    #[test]
    fn every_photo_carries_its_own_wording() {
        let html = gallery_html(photos());

        assert!(html.contains("photo 1"), "the first is unnamed: {html}");
        assert!(html.contains("photo 2"), "the second is unnamed: {html}");
        assert!(!html.contains(r#"alt="""#), "an empty alt should not ship: {html}");
    }

    /// Below the fold, every one of them.
    #[test]
    fn the_photos_load_lazily() {
        let html = gallery_html(photos());

        assert_eq!(html.matches(r#"loading="lazy""#).count(), 2, "{html}");
    }

    /// A workshop with no photo yet is the ordinary starting state. A heading
    /// standing over nothing would read as an image that failed to load.
    ///
    /// Asserted element by element rather than on an empty string: a `None` still
    /// renders Leptos's `<!>` placeholder, which is a hydration marker and not
    /// something a visitor ever sees.
    #[test]
    fn an_empty_gallery_draws_nothing_at_all() {
        let html = gallery_html(Vec::new());

        assert!(!html.contains("<section"), "no section should open: {html}");
        assert!(!html.contains("En images"), "nor a heading over nothing: {html}");
        assert!(!html.contains("<img"), "and certainly no image: {html}");
    }
}
