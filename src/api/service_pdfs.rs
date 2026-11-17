//! Server functions over the PDF each workshop offers from the catalogue.
//!
//! [`service_pdf`] is open to everyone: the file is what the catalogue links to.
//! [`upload_service_pdf`] reads a multipart stream, a file being unable to travel
//! through the urlencoded body an `<ActionForm>` builds; it follows
//! [`crate::api::service_photos::add_service_photo`] closely.

use leptos::prelude::*;
// At file scope rather than inside the function: both sides of `upload_service_pdf`
// name these, the browser to build the body and the server to read it.
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

/// Where one workshop's PDF is served, or `None` when it has none.
#[server]
pub async fn service_pdf(service: String) -> Result<Option<String>, ServerFnError> {
    use crate::api::log_failure;
    use crate::db::service_pdf;

    let found = service_pdf::find_for_service(service.trim())
        .await
        .map_err(|error| log_failure("loading the PDF of a workshop", error))?;

    Ok(found.map(|found| found.pdf_url()))
}

/// Gives a workshop its PDF, replacing the one it had.
///
/// Reads two multipart fields: `service`, the workshop's slug, and `pdf`.
#[server(input = MultipartFormData)]
pub async fn upload_service_pdf(data: MultipartData) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::{service as workshop, service_pdf};
    use crate::models::{MAX_PDF_BYTES, MAX_PDF_LABEL, is_pdf};

    require_admin()?;

    let mut fields = data
        .into_inner()
        .ok_or_else(|| ServerFnError::new("Ce formulaire n'a pas pu être lu."))?;

    let mut service = String::new();
    let mut pdf: Vec<u8> = Vec::new();

    while let Some(mut field) = fields
        .next_field()
        .await
        .map_err(|error| read_failure("reading the PDF form", error))?
    {
        let field_name = field.name().unwrap_or_default().to_owned();

        match field_name.as_str() {
            "service" => {
                service = field
                    .text()
                    .await
                    .map_err(|error| read_failure("reading the workshop field", error))?;
            }
            "pdf" => {
                // Read chunk by chunk so an oversized upload is turned down as soon
                // as it crosses the line, rather than after it has all been buffered.
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|error| read_failure("reading a workshop PDF", error))?
                {
                    if pdf.len() + chunk.len() > MAX_PDF_BYTES {
                        return Err(ServerFnError::new(format!(
                            "Ce PDF est trop lourd : {MAX_PDF_LABEL} au maximum."
                        )));
                    }
                    pdf.extend_from_slice(&chunk);
                }
            }
            // Ignored rather than refused: a browser may add fields of its own.
            _ => {}
        }
    }

    // The slug comes from a hidden field, but the form is an ordinary POST anyone
    // can forge, and a file filed under a slug naming nothing would be unreachable.
    let Some(found) = workshop::find_by_slug(service.trim())
        .await
        .map_err(|error| log_failure("checking the workshop of a PDF", error))?
    else {
        return Err(ServerFnError::new("Service inconnu."));
    };

    if pdf.is_empty() {
        return Err(ServerFnError::new("Choisissez un fichier PDF."));
    }
    if !is_pdf(&pdf) {
        return Err(ServerFnError::new("Ce fichier n'est pas un PDF."));
    }

    service_pdf::replace_for_service(&found.slug, pdf)
        .await
        .map_err(|error| log_failure("storing the PDF of a workshop", error))
}

/// Drops a workshop's PDF.
#[server]
pub async fn delete_service_pdf(service: String) -> Result<(), ServerFnError> {
    use crate::api::log_failure;
    use crate::auth::require_admin;
    use crate::db::service_pdf;

    require_admin()?;

    service_pdf::delete_for_service(service.trim())
        .await
        .map_err(|error| log_failure("deleting the PDF of a workshop", error))
}

/// Logs why a multipart body could not be read, and words it for the admin.
#[cfg(feature = "ssr")]
fn read_failure(context: &str, error: impl std::fmt::Display) -> ServerFnError {
    eprintln!("{context} failed: {error}");

    ServerFnError::new("Ce formulaire n'a pas pu être lu. Réessayez.")
}
