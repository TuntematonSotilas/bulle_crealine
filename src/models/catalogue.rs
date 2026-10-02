use serde::{Deserialize, Serialize};

/// A picture as a page shows it: where to fetch it, and what to say in its place.
///
/// Deliberately thinner than [`super::ServicePhotoView`], which also carries the id
/// the admin's delete button posts back. The catalogue only looks, so it is handed
/// only what it draws -- and that lets one type stand for two sources, a theme's
/// photo and a workshop's own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhotoView {
    /// Where the image is served from.
    pub url: String,
    /// Alternative text, resolved server-side.
    ///
    /// The catalogue never sees what either source is called: a theme's name and a
    /// workshop's label both live where the photos were gathered, not where they
    /// are drawn. A gallery of five `alt=""` images is the whole of what the page
    /// says to anyone reading it with their ears.
    pub alt: String,
}

/// One workshop's pictures, as the catalogue shows them.
///
/// Where they come from depends on the workshop and is settled server-side, so the
/// page draws one carousel either way:
///
/// - a bookable workshop shows the themes attached to it -- what it is about, run
///   after run, whether or not a date is on the calendar right now;
/// - a workshop run for structures shows its own photos, the ones the admin uploads
///   on its fiche, because it has no themes to speak of.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServicePictures {
    /// Which workshop these belong to. The slug, as every other document refers to
    /// a workshop, so this cannot go stale under a reworded label.
    pub service_slug: String,
    /// May be empty: a workshop with no theme and no photo yet is an ordinary
    /// starting state, and the page simply draws no carousel.
    pub photos: Vec<PhotoView>,
}
