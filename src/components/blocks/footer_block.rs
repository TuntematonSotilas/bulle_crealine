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
                        <img src="/assets/icon.svg" alt="Logo" class="w-8 h-8"/>
                        Bulle Créaline
                    </div>
                </FooterBrandLink>
                <FooterNavContainer>
                    <FooterLink attr:href="/mentions-legales">Mentions légales</FooterLink>
                </FooterNavContainer>
                <FooterNavContainer>
                    <FooterExternalLink href="https://www.facebook.com/coraline.colibri" attr:aria-label="Facebook">
                        <Facebook />
                    </FooterExternalLink>
                    <FooterExternalLink href="https://www.instagram.com/bullecrealine" attr:aria-label="Instagram">
                        <Instagram />
                    </FooterExternalLink>
                </FooterNavContainer>
            </FooterContainer>
        </Footer>
    }
}