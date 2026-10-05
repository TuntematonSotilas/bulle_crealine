use icons::{Facebook, Instagram};
use leptos::prelude::*;

use crate::components::ui::footer::*;

#[component]
pub fn FooterBlock() -> impl IntoView {
    view! {
        <Footer class="py-4 md:py-8 bg-accent">
            <FooterContainer>
                <FooterBrandLink class="mx-auto" attr:aria-label="go home" attr:href="/">
                    <div class="flex items-center gap-2">
                        <img src="/assets/icon.svg" alt="Logo" class="w-16 h-16"/>
                        <div class="flex flex-col">
                            <div>Bulle Créaline (E.I)</div>
                            <div class="text-sm text-muted-foreground">Ma source de créativité</div>
                        </div>
                    </div>
                </FooterBrandLink>
                // Ordered by who needs them: a visitor wanting to get in touch, then
                // the notice the law requires to be reachable from everywhere, then
                // the way in for the one person who administers the site.
                <FooterNavContainer>
                    <FooterLink attr:href="/contact">Contact</FooterLink>
                    <FooterLink attr:href="/mentions-legales">Mentions légales</FooterLink>
                    <FooterLink attr:href="/admin">Espace admin</FooterLink>
                </FooterNavContainer>
                <FooterNavContainer>
                    <FooterExternalLink href="https://www.facebook.com/BulleCrealine" attr:aria-label="Facebook">
                        <Facebook class="no-tooltips"/>
                    </FooterExternalLink>
                    <FooterExternalLink href="https://www.instagram.com/bullecrealine" attr:aria-label="Instagram">
                        <Instagram class="no-tooltips" />
                    </FooterExternalLink>
                </FooterNavContainer>
            </FooterContainer>
        </Footer>
    }
}
#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn footer_html() -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The links resolve `aria-current` against the location being rendered.
            provide_context(RequestUrl::new("/"));

            view! { <Router><FooterBlock/></Router> }.to_html()
        })
    }

    /// The footer is on every page, which is why the legal notice lives here: it has
    /// to be reachable from anywhere, and this is the only place that is.
    #[test]
    fn the_footer_leads_everywhere_it_has_to() {
        let html = footer_html();

        for path in ["/contact", "/mentions-legales", "/admin"] {
            assert!(
                html.contains(&format!(r#"href="{path}""#)),
                "{path} is unreachable from the footer: {html}"
            );
        }
    }

    /// Getting in touch comes before the notice, and the notice before the way in
    /// for the one person who administers the site.
    #[test]
    fn the_links_are_ordered_by_who_needs_them() {
        let html = footer_html();

        let at = |path: &str| {
            html.find(&format!(r#"href="{path}""#))
                .unwrap_or_else(|| panic!("{path} is missing: {html}"))
        };

        assert!(at("/contact") < at("/mentions-legales"), "contact comes first: {html}");
        assert!(at("/mentions-legales") < at("/admin"), "admin comes last: {html}");
    }
}
