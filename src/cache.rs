//! How long browsers may hold on to what actix serves.
//!
//! `actix-files` sets `ETag` and `Last-Modified` but no `Cache-Control`, which
//! leaves browsers free to invent a freshness lifetime of their own -- commonly a
//! tenth of the file's age. A visitor could then go on running the bundle of the
//! previous deploy for hours without ever asking the server whether it still is
//! the right one. Saying it out loud takes the guesswork away.

use std::sync::OnceLock;

use actix_web::Error;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::middleware::Next;

use crate::auth::ADMIN_PATH;

/// The bundle, once the image has stamped its name with a digest of its contents:
/// a browser that holds a copy may keep it for good, because the next deploy asks
/// for a different file altogether.
const BUNDLE: &str = "public, max-age=31536000, immutable";

/// The same bundle before that stamping -- `cargo leptos serve` locally, where the
/// name is fixed and a year would mean a year of stale WASM. The browser keeps its
/// copy but has to ask; answering costs a 304 with no body.
const UNSTAMPED_BUNDLE: &str = "no-cache";

/// Images and icons, whose names never change. They are replaced far less often
/// than the bundle and are harmless an hour out of date.
const ASSETS: &str = "public, max-age=3600";

/// What crawlers read. An hour is far shorter than any crawler's own interval, and
/// both files are cheap enough to rebuild that holding them longer buys nothing.
const CRAWLER: &str = "public, max-age=3600";

/// Server-rendered pages. `no-cache` rather than `no-store`, so that the back
/// button's instant restore goes on working.
const DOCUMENT: &str = "no-cache";

/// An admin page is personalised and belongs in no shared cache, nor on a disk.
const PRIVATE: &str = "no-store";

/// Whether the bundle ships under a name that changes with its contents.
///
/// Decided once, at startup, by comparing the name found on disk with the one the
/// build wrote in. A `OnceLock` rather than a parameter because the middleware is
/// a bare function: actix builds it afresh for every worker thread, and this is
/// the same answer for all of them.
static STAMPED: OnceLock<bool> = OnceLock::new();

/// Records what [`crate::main`] found on disk. Later calls are ignored, which is
/// what makes it safe to call from a worker factory.
pub fn set_bundle_is_stamped(stamped: bool) {
    let _ = STAMPED.set(stamped);
}

/// Attaches the cache policy that matches the path, leaving alone any response
/// that already carries one of its own -- the photos in [`crate::media`] do.
pub async fn cache_headers<B>(
    request: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody,
{
    let policy = policy_for(request.path(), *STAMPED.get().unwrap_or(&false));
    let mut response = next.call(request).await?;

    if let Some(policy) = policy {
        if !response.headers().contains_key(CACHE_CONTROL) {
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static(policy));
        }
    }

    Ok(response)
}

/// How long the response for `path` may be reused, or `None` to say nothing and
/// let whatever served it answer for itself.
fn policy_for(path: &str, stamped: bool) -> Option<&'static str> {
    if path.starts_with("/pkg/") {
        // wasm-bindgen copies snippets under `pkg` under names of its own, which
        // the stamping does not touch -- so they keep revalidating either way.
        if stamped && !path.starts_with("/pkg/snippets/") {
            Some(BUNDLE)
        } else {
            Some(UNSTAMPED_BUNDLE)
        }
    } else if path.starts_with("/assets/") || path == "/favicon.ico" {
        Some(ASSETS)
    } else if path == "/robots.txt" || path == "/sitemap.xml" {
        Some(CRAWLER)
    } else if path.starts_with("/media/") || path.starts_with("/api/") {
        // A photo states its own `immutable`, and a server function's answer is
        // never cached to begin with.
        None
    } else if path == ADMIN_PATH || path.starts_with(&format!("{ADMIN_PATH}/")) {
        Some(PRIVATE)
    } else {
        Some(DOCUMENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamped_bundle_may_be_kept_for_good() {
        for file in ["bulle_crealine.a3f8c21d9e04.js", "bulle_crealine.a3f8c21d9e04.wasm"] {
            let path = format!("/pkg/{file}");

            assert_eq!(policy_for(&path, true), Some(BUNDLE), "{path}");
        }
    }

    /// The same paths, the other way round: locally the name is fixed, so a year
    /// would be a year of running yesterday's code.
    #[test]
    fn an_unstamped_bundle_is_revalidated_every_time() {
        assert_eq!(policy_for("/pkg/bulle_crealine.js", false), Some(UNSTAMPED_BUNDLE));
        assert_eq!(policy_for("/pkg/bulle_crealine.wasm", false), Some(UNSTAMPED_BUNDLE));
    }

    /// Stamping renames three files; anything else under `pkg` keeps the name
    /// wasm-bindgen gave it, so it cannot claim to be immutable.
    #[test]
    fn snippets_are_revalidated_even_once_the_bundle_is_stamped() {
        assert_eq!(
            policy_for("/pkg/snippets/some-crate/inline0.js", true),
            Some(UNSTAMPED_BUNDLE)
        );
    }

    #[test]
    fn assets_may_go_stale_for_an_hour() {
        assert_eq!(policy_for("/assets/icon.svg", true), Some(ASSETS));
        assert_eq!(policy_for("/favicon.ico", true), Some(ASSETS));
    }

    /// Neither is a page, and letting them fall through to `DOCUMENT` would have
    /// every crawler revalidate a file that changes when a workshop is added.
    #[test]
    fn what_crawlers_read_may_be_kept_for_an_hour() {
        assert_eq!(policy_for("/robots.txt", true), Some(CRAWLER));
        assert_eq!(policy_for("/sitemap.xml", true), Some(CRAWLER));
    }

    #[test]
    fn pages_are_revalidated_and_admin_pages_are_not_stored() {
        assert_eq!(policy_for("/", true), Some(DOCUMENT));
        assert_eq!(policy_for("/services/aperos-creatifs", true), Some(DOCUMENT));
        assert_eq!(policy_for("/admin", true), Some(PRIVATE));
        assert_eq!(policy_for("/admin/services", true), Some(PRIVATE));
    }

    /// A public page whose path merely starts with the same letters is not an
    /// admin page -- the same trap [`crate::auth::middleware`] guards against.
    #[test]
    fn the_admin_policy_does_not_spill_onto_the_public_site() {
        assert_eq!(policy_for("/administration", true), Some(DOCUMENT));
    }

    /// Both of these already say what they need to; overwriting a photo's
    /// `immutable` with an hour would be a plain regression.
    #[test]
    fn what_answers_for_itself_is_left_alone() {
        assert_eq!(policy_for("/media/theme/651d1f0a0000000000000001", true), None);
        assert_eq!(policy_for("/media/service-photo/651d1f0a0000000000000002", true), None);
        assert_eq!(policy_for("/api/create_booking", true), None);
    }
}
