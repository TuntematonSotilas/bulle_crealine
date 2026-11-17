//! QR code generator, at `/admin/qrcode`.
//!
//! Nothing is stored and no server function is involved: the code is computed in
//! the page itself, from whatever the admin types, and handed back as an SVG file
//! to download — ready for a flyer, a business card or a poster in the studio.

use leptos::prelude::*;
use leptos_meta::Title;
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};

use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::label::Label;
use crate::components::ui::textarea::Textarea;
use crate::models::site_url;
use crate::pages::admin::AdminShell;

/// Smallest side of the generated image, in pixels. The SVG scales freely, this
/// only sets how large it opens by default.
const MIN_SIZE: u32 = 256;

/// Name offered for the downloaded file.
const FILE_NAME: &str = "qrcode-bulle-crealine.svg";

/// Why no code is shown, worded for the admin.
#[derive(Clone, Debug, PartialEq)]
enum QrProblem {
    Empty,
    TooLong,
}

impl QrProblem {
    fn message(&self) -> &'static str {
        match self {
            Self::Empty => "Saisissez un texte ou une adresse pour générer le QR code.",
            Self::TooLong => "Ce texte est trop long pour tenir dans un QR code.",
        }
    }
}

/// Renders `content` as an SVG QR code, dark modules on a white background.
///
/// Medium error correction: it survives a smudge or a crease on paper while
/// keeping the code readable from a distance. White is spelled out rather than
/// left transparent, so the code stays scannable on a dark page or a coloured flyer.
fn qr_svg(content: &str) -> Result<String, QrProblem> {
    let content = content.trim();
    if content.is_empty() {
        return Err(QrProblem::Empty);
    }

    let code = QrCode::with_error_correction_level(content.as_bytes(), EcLevel::M)
        .map_err(|_| QrProblem::TooLong)?;

    Ok(code
        .render::<svg::Color>()
        .min_dimensions(MIN_SIZE, MIN_SIZE)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .quiet_zone(true)
        .build())
}

/// Wraps an SVG document into a `data:` URL a download link can point at.
///
/// Everything outside the URL-unreserved set is percent-encoded: `#` alone would
/// otherwise cut the URL short at the first colour.
fn svg_data_url(svg: &str) -> String {
    let mut url = String::from("data:image/svg+xml;charset=utf-8,");

    for byte in svg.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            url.push(byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }

    url
}

/// QR code generator page.
#[component]
pub fn AdminQrCodePage() -> impl IntoView {
    let content = RwSignal::new(site_url("/"));
    let rendered = Memo::new(move |_| content.with(|text| qr_svg(text)));

    view! {
        <Title text="QR code — Administration"/>

        <AdminShell title="QR code" current="/admin/qrcode">
            <Card>
                <CardHeader>
                    <CardTitle>"Générateur de QR code"</CardTitle>
                    <CardDescription>
                        "Saisissez une adresse ou un texte : le QR code se met à jour au fil de la frappe."
                    </CardDescription>
                </CardHeader>

                <CardContent class="grid gap-6 md:grid-cols-2">
                    <div class="flex flex-col gap-2">
                        <Label r#for="qr-content">"Contenu"</Label>
                        <Textarea id="qr-content" rows=4 bind_value=content/>
                    </div>

                    <div class="flex flex-col gap-4 items-center">
                        {move || match rendered.get() {
                            Ok(svg) => {
                                let href = svg_data_url(&svg);
                                view! {
                                    <div
                                        class="p-2 bg-white rounded-md border [&_svg]:w-64 [&_svg]:h-auto"
                                        role="img"
                                        aria-label="QR code"
                                        inner_html=svg
                                    ></div>
                                    <a
                                        href=href
                                        download=FILE_NAME
                                        class="inline-flex justify-center items-center px-4 h-9 text-sm font-medium rounded-md shadow-xs bg-primary text-primary-foreground hover:bg-primary/90"
                                    >
                                        "Télécharger (SVG)"
                                    </a>
                                }
                                    .into_any()
                            }
                            Err(problem) => {
                                view! {
                                    <p class="text-sm text-muted-foreground">{problem.message()}</p>
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </CardContent>
            </Card>
        </AdminShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_content_yields_no_code() {
        assert_eq!(qr_svg(""), Err(QrProblem::Empty));
        assert_eq!(qr_svg("  \n\t "), Err(QrProblem::Empty));
    }

    #[test]
    fn a_url_renders_as_an_svg_on_white() {
        let svg = qr_svg("https://example.com/").expect("a short URL fits in a QR code");

        assert!(svg.contains("<svg"), "not an SVG document: {svg}");
        assert!(svg.contains("#ffffff"), "background is not white");
        assert!(svg.contains("#000000"), "modules are not black");
    }

    #[test]
    fn surrounding_whitespace_does_not_change_the_code() {
        assert_eq!(qr_svg("  https://example.com/\n"), qr_svg("https://example.com/"));
    }

    #[test]
    fn oversized_content_is_reported() {
        let huge = "x".repeat(5_000);

        assert_eq!(qr_svg(&huge), Err(QrProblem::TooLong));
    }

    #[test]
    fn data_url_escapes_everything_but_unreserved_characters() {
        let url = svg_data_url(r##"<svg fill="#000">é</svg>"##);

        assert_eq!(
            url,
            "data:image/svg+xml;charset=utf-8,%3Csvg%20fill%3D%22%23000%22%3E%C3%A9%3C%2Fsvg%3E"
        );
    }

    #[test]
    fn every_problem_has_a_message() {
        for problem in [QrProblem::Empty, QrProblem::TooLong] {
            assert!(!problem.message().is_empty(), "{problem:?} has no message");
        }
    }
}
