use leptos::prelude::*;

use crate::components::seo::PageMeta;
use crate::components::ui::lines::Lines;
use crate::models::{APP_HOST, DATA_HOST, OWNER};

/// Renders the "Mentions légales" page.
///
/// The legal notice a professional site owes under the LCEN, and nothing else. What a
/// visitor's data becomes is a different duty under a different law, and now lives on
/// its own page -- see [`crate::pages::privacy_policy::PrivacyPolicyPage`]. The two
/// were one page, which meant a visitor looking for the second read past a company
/// registration number to find it.
///
/// Every particular is read off [`OWNER`] rather than written here, so the legal
/// notice and the contact page cannot come to disagree.
#[component]
pub fn MentionsLegales() -> impl IntoView {
    view! {
        <PageMeta
            title="Mentions légales — Bulle Créaline (E.I)"
            description="Mentions légales de Bulle Créaline (E.I) : éditrice du site, statut, coordonnées, SIRET et hébergeurs."
            path="/mentions-legales"
        />

        <div class="py-6 mx-auto space-y-10 max-w-3xl">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Mentions légales"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
            </div>

            <section class="space-y-4">
                <h2 class="text-2xl font-semibold tracking-tight text-heading">"Éditeur"</h2>
                <Lines rows=vec![
                    ("Éditrice du site", OWNER.owner.to_owned()),
                    ("Statut", "Entrepreneuse individuelle".to_owned()),
                    ("Adresse", OWNER.address.to_owned()),
                    ("Téléphone", OWNER.phone.to_owned()),
                    ("Email", OWNER.email.to_owned()),
                    ("SIRET", OWNER.siret.to_owned()),
                    ("TVA", OWNER.vat.to_owned()),
                    ("Directrice de la publication", OWNER.owner.to_owned()),
                ]/>
            </section>

            <section class="space-y-4">
                <h2 class="text-2xl font-semibold tracking-tight text-heading">"Hébergement"</h2>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Le site est hébergé par "{APP_HOST.name}", 525 Brannan Street,
                     San Francisco, CA 94107, États-Unis, sur des serveurs situés à "
                    {APP_HOST.location}". Les données sont enregistrées chez "
                    {DATA_HOST.name}" sur des serveurs situés à "{DATA_HOST.location}"."
                </p>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Les deux prestataires sont des sociétés américaines, mais les
                     serveurs qu'ils mettent à disposition se trouvent dans l'Union
                     européenne : aucune donnée personnelle n'en sort."
                </p>
            </section>

            <section class="space-y-4">
                <h2 class="text-2xl font-semibold tracking-tight text-heading">
                    "Données personnelles"
                </h2>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Ce qui est collecté lors d'une réservation, pourquoi, combien de
                     temps c'est conservé et comment exercer vos droits est détaillé
                     dans la "
                    <a
                        href="/politique-de-confidentialite"
                        class="font-medium underline underline-offset-4 text-primary"
                    >
                        "politique de confidentialité"
                    </a>
                    "."
                </p>
            </section>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn page_html() -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            provide_context(RequestUrl::new("/mentions-legales"));

            view! { <Router><MentionsLegales/></Router> }.to_html()
        })
    }

    /// The LCEN asks for who publishes the site and how to reach them. Asserted
    /// against `OWNER` rather than against copies: a test spelling the address out
    /// would go on passing after the constant changed.
    #[test]
    fn the_notice_names_who_publishes_the_site() {
        let html = page_html();

        for particular in [OWNER.owner, OWNER.address, OWNER.phone, OWNER.email] {
            assert!(html.contains(particular), "{particular} is missing: {html}");
        }
    }

    /// Both are pending, and the page says so rather than leaving the line out: a
    /// mention marked as such is a reminder, a missing one is merely missing. The
    /// day either constant is filled in, the page follows and this test does not
    /// change.
    #[test]
    fn the_notice_prints_whatever_the_registration_lines_hold() {
        let html = page_html();

        assert!(html.contains("SIRET"), "no SIRET line at all: {html}");
        assert!(html.contains(OWNER.siret), "the SIRET line is not shown: {html}");
        assert!(html.contains(OWNER.vat), "the VAT line is not shown: {html}");
    }

    /// Naming the host is an obligation of its own, and where the servers stand is
    /// what decides whether anything has to be said about transfers. Both are read
    /// off the constants the privacy policy also reads, so the two pages cannot end
    /// up naming different hosts.
    #[test]
    fn the_notice_names_the_host_and_where_the_servers_stand() {
        let html = page_html();

        assert!(html.contains(APP_HOST.name), "no host: {html}");
        assert!(html.contains(APP_HOST.location), "no application server: {html}");
        assert!(html.contains(DATA_HOST.name), "no data host: {html}");
        assert!(html.contains(DATA_HOST.location), "no database server: {html}");
    }

    /// What became of the privacy policy is the one thing a visitor arriving here
    /// for it has to be told, since this is where it used to be.
    #[test]
    fn the_notice_leads_on_to_the_privacy_policy() {
        let html = page_html();

        assert!(
            html.contains(r#"href="/politique-de-confidentialite""#),
            "no way across to the policy: {html}"
        );
    }
}
