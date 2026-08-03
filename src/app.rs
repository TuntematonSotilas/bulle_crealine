use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment, WildcardSegment, components::{Route, Router, Routes}, path
};

use crate::components::hooks::use_theme_mode::ThemeMode;
use crate::components::ui::carousel::CarouselContext;
use crate::components::blocks::{NavMenu, FooterBlock};
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    // Provides context for theme mode management
    provide_context(ThemeMode::init());
    // Provides context for carousel functionality
    provide_context(CarouselContext::init());

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/bulle_crealine.css"/>

        // sets the document title
        <Title text="Bulle Créaline"/>

        <Meta property="og:title" content="Bulle Créaline" />
        <Meta property="og:url" content="https://bulle-crealine.onrender.com" />
        <Meta property="og:image" content="https://bulle-crealine.onrender.com/assets/meta.png" />
        <Meta property="og:description" content="Médiation artistique en relation d'aide." />
        <Meta property="og:site_name" content="Bulle Créaline" />
        <Meta property="og:type" content="website" />

        // content for this welcome page
        <Router>
            <NavMenu/>
            <main class="container mx-auto px-4 py-4 min-h-[72vh]">
                <Routes fallback=move || "Not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=path!("/services/hors-les-murs") view=AteliersHorsLesMurs/>
                    <Route path=path!("/services/en-institution") view=AteliersEnInstitution/>
                    <Route path=path!("/services/parents-enfants") view=AteliersParentsEnfants/>
                    <Route path=path!("/services/creatifs-pour-tous") view=AteliersCreatifsPourTous/>
                    <Route path=path!("/services/aperos-creatifs") view=AperosCreatifs/>
                    <Route path=path!("/services/individuels") view=AteliersIndividuels/>
                    <Route path=path!("/qui-suis-je") view=QuiSuisJePage/>
                    <Route path=path!("/mentions-legales") view=MentionsLegales/>
                    <Route path=path!("/newsletter") view=NewsletterPage/>
                    <Route path=path!("/admin") view=AdminPage/>
                    <Route path=path!("/admin/login") view=AdminLoginPage/>
                    <Route path=WildcardSegment("any") view=NotFound/>
                </Routes>
            </main>
            <FooterBlock/>
        </Router>
    }
}
