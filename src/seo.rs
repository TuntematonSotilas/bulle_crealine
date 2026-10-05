//! What crawlers read: `/robots.txt` and `/sitemap.xml`.
//!
//! Both are rooted, which is why they are actix routes rather than files dropped in
//! `assets/`: that directory is mounted under `/assets`, so a `robots.txt` placed
//! there would answer on `/assets/robots.txt`, which no crawler ever asks for. The
//! favicon is served by hand for the same reason.

use actix_web::HttpResponse;

use crate::auth::ADMIN_PATH;
use crate::models::{ServiceView, is_valid_slug, site_url};

/// The pages that exist whatever the database says.
///
/// Written out rather than derived from the router's route list: `generate_route_list`
/// knows the shapes, not which of them is worth indexing -- it would offer `/admin`,
/// `/booking/:service` and the wildcard just the same. The workshops are the part
/// that changes, and those are read from storage.
const STATIC_PATHS: [&str; 8] = [
    "/",
    "/moi-et-mon-atelier/qui-suis-je",
    "/moi-et-mon-atelier/mon-atelier",
    "/moi-et-mon-atelier/photos",
    "/moi-et-mon-atelier/diplomes-et-formations",
    "/moi-et-mon-atelier/catalogue",
    "/contact",
    "/mentions-legales",
];

/// `GET /robots.txt` -- where crawlers may go, and where the map is.
pub async fn robots() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(robots_txt())
}

/// `GET /sitemap.xml` -- every public page, workshops included.
pub async fn sitemap() -> HttpResponse {
    use crate::db::service;

    // A sitemap without the workshops is still a sitemap; one answering 500 is not.
    // Storage is optional here as it is everywhere else, and the failure is the
    // server's business rather than the crawler's.
    let workshops: Vec<ServiceView> = match service::list_all().await {
        Ok(found) => found.iter().map(|document| document.to_view()).collect(),
        Err(error) => {
            eprintln!("listing the workshops for the sitemap failed: {error}");
            Vec::new()
        }
    };

    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(sitemap_xml(&sitemap_paths(&workshops)))
}

/// The body of `/robots.txt`.
///
/// `/booking/` is deliberately **not** disallowed. Those pages carry a `noindex`
/// instead, and the two do not combine: a crawler kept out by `Disallow` never reads
/// the `noindex`, and can still list the URL from a link pointing at it. Blocking
/// crawling and refusing indexing are different asks, and only the second is wanted.
fn robots_txt() -> String {
    format!(
        "User-agent: *\n\
         Allow: /\n\
         Disallow: {ADMIN_PATH}\n\
         Disallow: /api/\n\
         \n\
         Sitemap: {}\n",
        site_url("/sitemap.xml")
    )
}

/// Every path worth offering, the static pages first and then one per workshop.
fn sitemap_paths(workshops: &[ServiceView]) -> Vec<String> {
    STATIC_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .chain(
            workshops
                .iter()
                // A slug reaches a URL unescaped, and `is_valid_slug` is the whole
                // guarantee that it never needs escaping. A document predating that
                // check is left out rather than allowed to break the XML.
                .filter(|workshop| is_valid_slug(&workshop.slug))
                // `page_path()`, so a workshop is offered under the one prefix its
                // section calls its own: listing both would hand the crawler two
                // addresses for one page, which is the thing a sitemap settles.
                .map(ServiceView::page_path),
        )
        .collect()
}

/// The paths as a sitemap.
///
/// Bare `<loc>` entries: no `<lastmod>`, because no document carries a modification
/// date and one derived from the `ObjectId` would say "created" while claiming to
/// say "changed" -- a lie crawlers have learned to discount. No `<changefreq>` and
/// no `<priority>` either; Google ignores both and says so.
fn sitemap_xml(paths: &[String]) -> String {
    let entries: String = paths
        .iter()
        .map(|path| format!("  <url><loc>{}</loc></url>\n", site_url(path)))
        .collect();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         {entries}</urlset>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workshop(slug: &str, pro: bool) -> ServiceView {
        ServiceView {
            id: String::new(),
            slug: slug.to_owned(),
            label: "Un atelier".to_owned(),
            description: String::new(),
            age: String::new(),
            steps: Vec::new(),
            icon: String::new(),
            pro,
            position: 0,
            min_persons: 1,
        }
    }

    /// One workshop, one address. Offering both prefixes would be handing the
    /// crawler the duplicate the canonical tags exist to resolve.
    #[test]
    fn every_workshop_is_listed_under_the_section_it_belongs_to() {
        let paths =
            sitemap_paths(&[workshop("aperos-creatifs", false), workshop("en-institution", true)]);

        assert!(paths.contains(&"/services/aperos-creatifs".to_owned()), "{paths:?}");
        assert!(paths.contains(&"/pro/en-institution".to_owned()), "{paths:?}");
        assert!(!paths.contains(&"/pro/aperos-creatifs".to_owned()), "{paths:?}");
        assert!(!paths.contains(&"/services/en-institution".to_owned()), "{paths:?}");
    }

    /// A sitemap is read from somewhere else entirely, so a relative path in it
    /// means nothing.
    #[test]
    fn every_address_in_the_sitemap_is_absolute() {
        let xml = sitemap_xml(&sitemap_paths(&[workshop("aperos-creatifs", false)]));

        for line in xml.lines().filter(|line| line.contains("<loc>")) {
            assert!(
                line.contains(&format!("<loc>{}", crate::models::SITE_ORIGIN)),
                "not an absolute address: {line}"
            );
        }
    }

    /// Storage is optional, and a short sitemap is worth more than a 500.
    #[test]
    fn a_sitemap_without_any_workshop_still_lists_the_pages() {
        let paths = sitemap_paths(&[]);

        assert!(paths.contains(&"/".to_owned()), "{paths:?}");
        assert!(paths.contains(&"/contact".to_owned()), "{paths:?}");
    }

    /// Offering a crawler an address with nothing to rank spends its budget on
    /// nothing, and an address that answers 404 teaches it the map is unreliable.
    #[test]
    fn the_sitemap_never_offers_a_page_that_has_nothing_to_rank() {
        let paths = sitemap_paths(&[workshop("aperos-creatifs", false)]);

        for unwanted in ["/admin", "/admin/login", "/newsletter", "/booking/aperos-creatifs"] {
            assert!(
                !paths.contains(&unwanted.to_owned()),
                "{unwanted} should be left out: {paths:?}"
            );
        }
    }

    /// Nothing escapes a slug on its way into the XML, so a slug needing escaping
    /// is a broken sitemap. `is_valid_slug` is what rules that out, and a document
    /// predating it is dropped rather than trusted.
    #[test]
    fn a_slug_that_would_need_escaping_is_left_out() {
        let paths = sitemap_paths(&[workshop("un atelier & deux", false)]);

        assert_eq!(paths.len(), STATIC_PATHS.len(), "a bad slug reached the sitemap: {paths:?}");
    }

    /// The map is worth nothing if nothing points at it.
    #[test]
    fn robots_points_at_the_sitemap() {
        let robots = robots_txt();

        assert!(robots.contains(&site_url("/sitemap.xml")), "{robots}");
    }

    /// The admin area is already behind a password; keeping crawlers out of it
    /// spares it the traffic and keeps its addresses out of search results.
    #[test]
    fn robots_keeps_crawlers_out_of_the_admin_area() {
        let robots = robots_txt();

        assert!(robots.contains(&format!("Disallow: {ADMIN_PATH}")), "{robots}");
    }

    /// A booking page is `noindex`, not disallowed: a crawler kept out never reads
    /// the `noindex`. The two look interchangeable and are not.
    #[test]
    fn booking_pages_are_left_crawlable_so_their_noindex_is_read() {
        let robots = robots_txt();

        assert!(!robots.contains("Disallow: /booking"), "{robots}");
    }
}
