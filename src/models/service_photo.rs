use serde::{Deserialize, Serialize};

/// How many photos one workshop's gallery may hold.
///
/// Lives here rather than beside the document it bounds so that the admin form and
/// the server quote the same figure: the db layer is server-only, and the page
/// could not read a constant from it. Same reasoning as [`super::MAX_PHOTO_BYTES`].
pub const MAX_SERVICE_PHOTOS: usize = 5;

/// One photo of a workshop, as the browser sees it.
///
/// Carries a URL, never bytes: the image itself is served by
/// `GET /media/service-photo/{id}`, so listing a gallery stays cheap and the WASM
/// bundle never handles binary data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServicePhotoView {
    /// Hex form of the Mongo `ObjectId`, which is what the delete button posts back.
    pub id: String,
    /// Where to fetch it. No cache-busting stamp: a photo document is never
    /// rewritten, so the id alone addresses bytes that cannot change.
    pub url: String,
    /// Alternative text, resolved server-side.
    ///
    /// The gallery is a sibling of the workshop's presentation rather than its
    /// child, so it never sees the label this is built from -- and a gallery of
    /// five `alt=""` images is the whole content of a page for anyone reading it
    /// with their ears.
    pub alt: String,
}
