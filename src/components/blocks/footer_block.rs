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
                            <div>Bulle Créaline</div>
                            <div class="text-sm text-muted-foreground">Ma source de créativité</div>
                        </div>
                    </div>
                </FooterBrandLink>
                <FooterNavContainer>
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