//! Everything a crawler and a link preview read about a page.

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

use crate::models::site_url;

/// The title, the description, the canonical and the link preview, in one place.
///
/// One component rather than five tags per page, because four of the five have to
/// agree: the canonical URL, `og:url` and the path the sitemap lists are the same
/// string, and a page setting one while forgetting another is worse off than a page
/// setting none.
///
/// # It has to be rendered synchronously
///
/// `leptos_meta` only injects into the `<head>` what was rendered in the **first
/// chunk** of the stream, and the default SSR mode sends suspended fragments after
/// it. A `<Title>` inside a `Suspend` therefore never reaches the head -- which is
/// exactly what every workshop page used to do, shipping the site's generic title.
/// Where the wording depends on stored data, the resource behind it has to be a
/// [`Resource::new_blocking`](leptos::prelude::Resource::new_blocking), which holds
/// the first chunk back until it resolves.
///
/// # Nothing here may also be set globally
///
/// `<Title>` is a single slot and the last writer wins, so [`crate::app::App`] may
/// keep one as a fallback. `<Meta>` and `<Link>` are an append-only buffer with no
/// deduplication: a tag set here *and* in `App` would appear twice in the HTML.
/// That is why `App` carries only the tags no page overrides.
#[component]
pub fn PageMeta(
    /// The `<title>`, site name included: "Apéros créatifs — Bulle Créaline (E.I)".
    /// Aim for 50 to 60 characters, past which a search result cuts it.
    #[prop(into)]
    title: String,
    /// One or two sentences, 120 to 160 characters, written for the search result
    /// rather than for the page.
    #[prop(into)]
    description: String,
    /// The path this page answers on, rooted and without a query: "/contact".
    ///
    /// A prop rather than something read off the router, because the two are allowed
    /// to differ: a workshop answers on both `/services/<slug>` and `/pro/<slug>`,
    /// and only the one its section calls its own belongs in a canonical.
    #[prop(into)]
    path: String,
    /// Absolute URL of the image a link preview shows, for a page having one of its
    /// own. Falls back to the site's.
    #[prop(optional, into)]
    image: Option<String>,
    /// Keeps the page out of the index while leaving its links crawlable, for a page
    /// that exists but has nothing to rank: a booking form, a stub.
    ///
    /// Deliberately not a `Disallow` in `robots.txt`: a crawler kept out never reads
    /// the `noindex`, and can still index the URL from a link pointing at it.
    #[prop(optional)]
    noindex: bool,
) -> impl IntoView {
    let url = site_url(&path);
    let image = image.unwrap_or_else(|| site_url("/assets/meta.png"));

    view! {
        <Title text=title.clone()/>
        <Meta name="description" content=description.clone()/>
        <Link rel="canonical" href=url.clone()/>

        <Meta property="og:title" content=title/>
        <Meta property="og:description" content=description/>
        <Meta property="og:url" content=url/>
        <Meta property="og:image" content=image/>
        <Meta name="twitter:card" content="summary_large_image"/>

        {noindex.then(|| view! { <Meta name="robots" content="noindex, follow"/> })}
    }
}

/// A description cut down to what a search result will show, ended on a word.
///
/// Workshop descriptions are typed in the admin area and are written for the page,
/// where length costs nothing. Past roughly 160 characters a search result cuts them
/// mid-word anyway; cutting here at least chooses where.
///
/// Counts and cuts `char`s, not bytes: every description here is French, and slicing
/// "créatifs" through its accent panics rather than merely reading badly.
pub fn clamp_description(text: &str, limit: usize) -> String {
    let text = text.trim();

    if text.chars().count() <= limit {
        return text.to_owned();
    }

    // One character short of the limit, so the ellipsis fits inside it.
    let mut cut: String = text.chars().take(limit.saturating_sub(1)).collect();

    // Back up to the last word boundary. A cut with no space in it at all is a
    // single very long word, and there is nothing better to do than cut it.
    if let Some(space) = cut.rfind(' ') {
        cut.truncate(space);
    }

    format!("{}…", cut.trim_end_matches([' ', ',', ';', ':', '.']))
}

#[cfg(all(test, feature = "ssr"))]
mod head_tests {
    use futures_util::StreamExt;
    use leptos::prelude::*;
    use leptos_meta::ServerMetaContext;

    use super::PageMeta;

    /// What a page actually puts in the `<head>`.
    ///
    /// Rendering a component to HTML is not enough to see its metadata: `<Title>`,
    /// `<Meta>` and `<Link>` render nothing where they stand and send themselves to
    /// a context instead, which the server splices into the first chunk of the
    /// stream. Asserting on the rendered body would pass whatever the tags said --
    /// so this does the splicing the server does, into a shell standing in for
    /// `main.rs`.
    fn head_of(page: impl FnOnce() + 'static) -> String {
        crate::pages::admin::init_test_executor();

        let (context, output) = ServerMetaContext::new();

        Owner::new().with(|| {
            provide_context(context);
            page();
        });

        let shell = "<html><head></head><body></body></html>".to_owned();

        futures_executor::block_on(async move {
            // `iter` rather than `once`: the stream has to be `Unpin`, and the future
            // inside a `once` is not.
            let stream = futures_util::stream::iter([shell]);
            let mut injected = Box::pin(output.inject_meta_context(stream).await);

            injected.next().await.unwrap_or_default()
        })
    }

    fn head_of_a_page(noindex: bool) -> String {
        head_of(move || {
            let _ = view! {
                <PageMeta
                    title="Apéros créatifs — Bulle Créaline (E.I)"
                    description="Une soirée entre adultes autour d'un verre, à Feurs."
                    path="/services/aperos-creatifs"
                    noindex=noindex
                />
            }
            .to_html();
        })
    }

    /// The `<link>` element declaring the canonical, attributes in whatever order
    /// the renderer chose to write them.
    fn canonical_element(head: &str) -> &str {
        head.split("<link")
            .find(|element| element.contains(r#"rel="canonical""#))
            .unwrap_or_else(|| panic!("no canonical: {head}"))
    }

    /// The three a search result is built from. None of them existed anywhere on
    /// the site before: there was one title for every page and no description at
    /// all.
    #[test]
    fn a_page_carries_its_own_title_description_and_canonical() {
        let head = head_of_a_page(false);

        assert!(
            head.contains("<title>Apéros créatifs — Bulle Créaline (E.I)</title>"),
            "no title of its own: {head}"
        );
        assert!(
            head.contains(r#"<meta name="description" content="Une soirée entre adultes"#),
            "no description: {head}"
        );
        assert!(
            canonical_element(&head).contains(
                r#"href="https://bulle-crealine.onrender.com/services/aperos-creatifs""#
            ),
            "the canonical points elsewhere: {head}"
        );
    }

    /// A title is a single slot and the last writer wins, but `<Meta>` and `<Link>`
    /// are an append-only buffer with no deduplication. A tag set both here and in
    /// `App` would reach the HTML twice, and two descriptions are worse than none.
    #[test]
    fn nothing_a_page_sets_is_ever_set_twice() {
        let head = head_of_a_page(false);

        for tag in [r#"name="description""#, r#"rel="canonical""#, r#"property="og:url""#] {
            assert_eq!(head.matches(tag).count(), 1, "{tag} appears twice: {head}");
        }
    }

    /// The canonical is what tells a crawler that `/services/<slug>` and
    /// `/pro/<slug>` are one page and not two. It has to be absolute: a sitemap and
    /// a search index read it from somewhere else entirely.
    #[test]
    fn the_canonical_and_the_shared_address_are_the_same_absolute_one() {
        let head = head_of_a_page(false);

        let canonical = "https://bulle-crealine.onrender.com/services/aperos-creatifs";

        assert!(
            canonical_element(&head).contains(&format!(r#"href="{canonical}""#)),
            "the canonical is not the absolute address: {head}"
        );
        assert!(
            head.contains(&format!(r#"property="og:url" content="{canonical}""#)),
            "a link preview would name a different address: {head}"
        );
    }

    /// Asked for, and only when asked for: a `noindex` left on by accident takes a
    /// page out of search results and says nothing about it anywhere.
    #[test]
    fn a_page_is_kept_out_of_the_index_only_when_it_asks_to_be() {
        assert!(
            head_of_a_page(true).contains(r#"<meta name="robots" content="noindex, follow">"#),
            "{}",
            head_of_a_page(true)
        );
        assert!(
            !head_of_a_page(false).contains(r#"name="robots""#),
            "{}",
            head_of_a_page(false)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Slicing by byte would panic here: the cut falls inside the two bytes of "é".
    #[test]
    fn a_description_is_cut_between_letters_and_not_inside_one() {
        let text = "Un atelier créatif à Feurs, où l'on crée, où l'on souffle, et où l'on repart fier";

        for limit in 1..text.chars().count() {
            let cut = clamp_description(text, limit);

            assert!(
                cut.chars().count() <= limit,
                "{limit} characters asked for, {} given: {cut}",
                cut.chars().count()
            );
        }
    }

    /// Short enough to show whole is shown whole, ellipsis included.
    #[test]
    fn a_description_that_already_fits_is_left_alone() {
        let text = "Ateliers créatifs à Feurs.";

        assert_eq!(clamp_description(text, 160), text);
        assert_eq!(clamp_description(text, text.chars().count()), text);
    }

    /// Cutting mid-word reads as a typo rather than as a continuation.
    #[test]
    fn a_description_is_cut_between_words() {
        let cut = clamp_description("Ateliers créatifs bien-être à Feurs, dans la Loire", 30);

        assert!(cut.ends_with('…'), "no ellipsis: {cut}");
        assert!(!cut.contains("…e"), "cut inside a word: {cut}");
        assert!(
            "Ateliers créatifs bien-être à Feurs, dans la Loire".starts_with(
                cut.trim_end_matches('…')
            ),
            "the cut should be a prefix of the original: {cut}"
        );
    }

    /// Punctuation left hanging before an ellipsis reads as a mistake.
    #[test]
    fn a_cut_never_leaves_punctuation_dangling_before_the_ellipsis() {
        let cut = clamp_description("Ateliers créatifs, à Feurs, dans la Loire", 20);

        assert!(!cut.contains(",…"), "a comma left before the ellipsis: {cut}");
    }
}
