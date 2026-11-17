use serde::{Deserialize, Serialize};

/// Largest PDF accepted, in bytes.
///
/// Under Mongo's 16 MB document ceiling, the file being stored whole in one
/// document. Lives here rather than beside that document so that the admin form and
/// the server quote the same figure: the db layer is server-only.
pub const MAX_PDF_BYTES: usize = 10 * 1024 * 1024;

/// [`MAX_PDF_BYTES`] as it is worded to the admin.
pub const MAX_PDF_LABEL: &str = "10 Mo";

/// Whether these bytes are a PDF, judged by their signature.
///
/// Sniffed rather than trusting the type the browser declares: the media route
/// serves the file back as `application/pdf`, and anything else uploaded under that
/// banner would be handed to the visitor's PDF reader.
pub fn is_pdf(bytes: &[u8]) -> bool {
    bytes.starts_with(b"%PDF-")
}

/// Where one workshop's PDF can be read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServicePdfLink {
    /// Which workshop it belongs to, by slug, as every other document refers to one.
    pub service_slug: String,
    /// Where `GET /media/service-pdf/{id}` serves it.
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pdf_is_recognised_by_its_signature() {
        assert!(is_pdf(b"%PDF-1.7\n%..."));
        assert!(is_pdf(b"%PDF-2.0"));
    }

    #[test]
    fn anything_else_is_refused() {
        assert!(!is_pdf(b""), "an empty file");
        assert!(!is_pdf(b"\x89PNG\r\n\x1a\n"), "an image");
        assert!(!is_pdf(b"<html>%PDF-1.7"), "a signature not at the start");
        assert!(!is_pdf(b"%PDF"), "a truncated signature");
    }
}
