use leptos::prelude::*;
use leptos_meta::{Meta, Title, provide_meta_context};
use leptos_router::{
    StaticSegment, WildcardSegment, components::{Route, Router, Routes}, path
};

use crate::components::hooks::use_theme_mode::ThemeMode;
use crate::components::blocks::{NavMenu, FooterBlock};
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    // Provides context for theme mode management
    provide_context(ThemeMode::init());

    view! {
        // The fallback title, for a page that sets none of its own. A title is a
        // single slot where the last writer wins, so a page's own `PageMeta`
        // replaces this one rather than adding to it.
        <Title text="Bulle Créaline (E.I)"/>

        // Only what no page overrides. `leptos_meta` keeps `<Meta>` in an
        // append-only buffer with no deduplication, so anything set here *and* by
        // `PageMeta` would reach the HTML twice -- which is why `og:title`,
        // `og:url`, `og:description` and `og:image` are the page's business alone.
        <Meta property="og:site_name" content="Bulle Créaline (E.I)" />
        <Meta property="og:type" content="website" />
        <Meta property="og:locale" content="fr_FR" />

        // content for this welcome page
        <Router>
            <NavMenu/>
            <main class="container mx-auto px-4 py-4 min-h-[72vh] bubbles">
                <Routes fallback=move || "Not found.">
                    <Route path=StaticSegment("") view=HomePage/>

                    // One route per section, one component behind both: workshops
                    // are rows the admin edits, so which ones exist is not known at
                    // compile time. The prefix says whether a workshop can be booked.
                    <Route path=path!("/services/:slug") view=ServicePage/>
                    <Route path=path!("/pro/:slug") view=ServicePage/>

                    <Route path=path!("/moi-et-mon-atelier/qui-suis-je") view=QuiSuisJePage/>
                    <Route path=path!("/moi-et-mon-atelier/mon-atelier") view=MonAtelierPage/>
                    <Route path=path!("/moi-et-mon-atelier/photos") view=PhotosPage/>
                    <Route path=path!("/moi-et-mon-atelier/diplomes-et-formations") view=DiplomesEtFormationsPage/>
                    <Route path=path!("/moi-et-mon-atelier/catalogue") view=CataloguePage/>
                    
                    <Route path=path!("/newsletter") view=NewsletterPage/>
                    <Route path=path!("/contact") view=ContactPage/>
                    <Route path=path!("/mentions-legales") view=MentionsLegales/>
                    <Route path=path!("/politique-de-confidentialite") view=PrivacyPolicyPage/>
                    
                    <Route path=path!("/booking/:service") view=BookingPage/>
                    <Route path=path!("/admin") view=AdminPage/>
                    <Route path=path!("/admin/login") view=AdminLoginPage/>
                    <Route path=path!("/admin/services") view=AdminServicesPage/>
                    <Route path=path!("/admin/sessions") view=AdminSessionsPage/>
                    <Route path=path!("/admin/themes") view=AdminThemesPage/>
                    <Route path=path!("/admin/bookings") view=AdminBookingsPage/>
                    <Route path=WildcardSegment("any") view=NotFound/>
                </Routes>
            </main>
            <FooterBlock/>
        </Router>
    }
}
