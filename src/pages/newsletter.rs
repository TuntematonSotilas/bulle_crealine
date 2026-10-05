use leptos::prelude::*;

use crate::components::seo::PageMeta;
use crate::components::ui::{button::{Button, ButtonSize}, input::Input};

/// Renders the newsletter subscription page.
#[component]
pub fn NewsletterPage() -> impl IntoView {
    
    view! {
        // `noindex`: the form posts nowhere yet. A page that cannot do what it
        // announces is a page a search result should not send anyone to.
        <PageMeta
            title="Newsletter — Bulle Créaline (E.I)"
            description="S'inscrire à la newsletter de Bulle Créaline pour suivre les prochaines séances d'ateliers créatifs à Feurs."
            path="/newsletter"
            noindex=true
        />

        <div>
            <h1>Notre Newsletter</h1>

            <section>
                <h2>"Restez Informé"</h2>
                <p>"Inscrivez-vous à notre newsletter pour recevoir :"</p>
                <ul>
                    <li>"Les dates et horaires de nos nouveaux ateliers"</li>
                    <li>"Des conseils créatifs et des astuces DIY"</li>
                    <li>"Les actualités et les projets spéciaux"</li>
                    <li>"Des offres et promotions exclusives"</li>
                </ul>
            </section>
            
            <form class="row-start-1 pb-8 text-sm border-b md:col-span-2 md:border-none lg:col-span-1">
                <div class="space-y-4">
                    <label
                        data-slot="label"
                        class="block text-sm font-medium leading-none select-none peer-disabled:cursor-not-allowed peer-disabled:opacity-50 group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50"
                        for="mail"
                    >
                        Newsletter
                    </label>
                    <div class="flex gap-2">
                        <Input
                            attr:r#type="email"
                            attr:id="mail"
                            attr:placeholder="Your email"
                            attr:name="mail"
                            class="h-8 text-sm"
                        />
                        <Button size=ButtonSize::Sm>Submit</Button>
                    </div>
                    <span class="block text-sm text-muted-foreground">"Don't miss any update!"</span>
                </div>
            </form>
        

            <section>
                <h2>"Votre Confidentialité"</h2>
                <p>"Nous respectons votre vie privée. Vos données ne seront jamais partagées à des tiers. Vous pouvez vous désabonner à tout moment."</p>
            </section>

        </div>
    }
}
