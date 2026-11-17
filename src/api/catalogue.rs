//! Server functions backing the catalogue page.

use leptos::prelude::*;

use crate::models::{PhotoView, ServicePdfLink, ServicePictures};

/// A bookable workshop's themes as pictures, in the order storage lists them.
#[cfg(feature = "ssr")]
fn theme_pictures(themes: &[crate::db::theme::ThemeSummaryDoc], slug: &str) -> Vec<PhotoView> {
    themes
        .iter()
        .filter(|found| found.service_slug == slug)
        .map(|found| PhotoView {
            url: found.photo_url(),
            alt: format!("Thème : {}", found.name),
            theme: found.name.clone(),
        })
        .collect()
}

/// The pictures every workshop shows in the catalogue, one entry per workshop.
///
/// Three reads for the whole page rather than one per workshop. The catalogue draws
/// every workshop at once, so a call per rubric would be one request per tile -- and
/// a `Resource` born inside the page's already-resolving suspense would miss the
/// server's render altogether, the trap documented on
/// [`crate::pages::service::ServicePage`].
///
/// Where the pictures come from depends on the workshop, and that is settled here so
/// the page has one shape to draw:
///
/// - a bookable workshop shows the themes attached to it, whether or not any of them
///   has a date on the calendar right now: the catalogue is the shop window, and a
///   theme between two runs is still something on offer;
/// - a workshop run for structures shows its own photos, having no themes.
///
/// Open to everyone: it exposes nothing a visitor cannot already see on the matching
/// workshop page.
#[server]
pub async fn catalogue_photos() -> Result<Vec<ServicePictures>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::{service, service_photo, theme};
    use crate::models::MAX_SERVICE_PHOTOS;

    let services = service::list_all()
        .await
        .map_err(|error| log_failure("listing the workshops of the catalogue", error))?;

    // Summaries, so neither read drags a single image through memory: both
    // collections project their bytes away, and what comes back is a URL each.
    let themes = theme::list_all()
        .await
        .map_err(|error| log_failure("listing the themes of the catalogue", error))?;
    let photos = service_photo::list_all()
        .await
        .map_err(|error| log_failure("listing the photos of the catalogue", error))?;

    Ok(services
        .into_iter()
        .map(|workshop| {
            let pictures = if workshop.pro {
                photos
                    .iter()
                    .filter(|photo| photo.service_slug == workshop.slug)
                    // The same cap the workshop's own page applies: the admin form
                    // refuses a sixth, and anything already stored past it would
                    // otherwise show here and nowhere else.
                    .take(MAX_SERVICE_PHOTOS)
                    .enumerate()
                    .map(|(rank, photo)| PhotoView {
                        url: photo.photo_url(),
                        // Numbered because the photos carry no wording of their own.
                        // It says which workshop and which of its pictures, which is
                        // all there is to say truthfully.
                        alt: format!("{} — photo {}", workshop.label, rank + 1),
                        theme: String::new(),
                    })
                    .collect()
            } else {
                theme_pictures(&themes, &workshop.slug)
            };

            ServicePictures { service_slug: workshop.slug, photos: pictures }
        })
        .collect())
}

/// Every theme of one workshop, for the "Les thèmes" section of its page.
///
/// Whether or not a date is on the calendar: the page's schedule already lists the
/// themes that have one, and this is the rest of what is on offer. Empty for an
/// unknown slug, and for a workshop run for structures, which has no themes.
///
/// Open to everyone, like [`catalogue_photos`], which already shows the same
/// pictures.
#[server]
pub async fn service_themes(service: String) -> Result<Vec<PhotoView>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::theme;

    let themes = theme::list_all()
        .await
        .map_err(|error| log_failure("listing the themes of a workshop", error))?;

    Ok(theme_pictures(&themes, service.trim()))
}

/// Every workshop's PDF, one entry per workshop that has one.
///
/// One read for the whole page, the catalogue listing every workshop at once. Open
/// to everyone: the files are what the catalogue links to.
#[server]
pub async fn catalogue_pdfs() -> Result<Vec<ServicePdfLink>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::service_pdf;

    let pdfs = service_pdf::list_all()
        .await
        .map_err(|error| log_failure("listing the PDFs of the catalogue", error))?;

    Ok(pdfs
        .into_iter()
        .map(|pdf| ServicePdfLink { url: pdf.pdf_url(), service_slug: pdf.service_slug })
        .collect())
}
