use leptos::prelude::*;

use crate::components::seo::PageMeta;
use crate::components::ui::lines::Lines;
use crate::models::{APP_HOST, DATA_HOST, OWNER};

/// How long a booking is kept, counted from the last contact.
///
/// Stated here because this page is where the promise is made. Note that nothing in
/// the code enforces it yet: a booking is deleted logically, the document stays. The
/// day that changes, this is the figure to honour.
pub const RETENTION: &str = "3 ans à compter du dernier contact";

/// Renders the "Politique de confidentialité" page.
///
/// Split out of the legal notice, which it used to share. The two answer to different
/// laws -- the LCEN asks who publishes the site, article 13 of the GDPR asks what
/// becomes of a visitor's data -- and a visitor looking for the second was reading
/// past the first. Keeping them apart is also what lets each be linked to from where
/// it matters: the booking form points here, not at a company registration number.
///
/// Every particular is read off [`OWNER`], [`APP_HOST`] and [`DATA_HOST`] rather than
/// written here, so this page and the legal notice cannot come to disagree about who
/// holds the data or where it sits.
#[component]
pub fn PrivacyPolicyPage() -> impl IntoView {
    view! {
        <PageMeta
            title="Politique de confidentialité — Bulle Créaline (E.I)"
            description="Quelles données Bulle Créaline collecte lors d'une réservation, pourquoi, combien de temps elles sont conservées, et comment exercer vos droits."
            path="/politique-de-confidentialite"
        />

        <div class="py-6 mx-auto space-y-10 max-w-3xl">
            <div class="space-y-4 text-center">
                <h1 class="text-4xl font-semibold tracking-tight md:text-5xl text-heading">
                    "Politique de confidentialité"
                </h1>
                <span class="block mx-auto w-10 h-1 rounded-full bg-heading-soft"></span>
                <p class="mx-auto max-w-2xl text-sm leading-relaxed text-muted-foreground">
                    "Réserver une séance suppose de laisser de quoi vous recontacter. Voici ce qui est conservé, pourquoi, et ce que vous pouvez en exiger."
                </p>
            </div>

            <section class="space-y-4">
                <h2 class="text-2xl font-semibold tracking-tight text-heading">
                    "Données personnelles"
                </h2>
                <Lines rows=vec![
                    ("Responsable du traitement", format!("{}, {}", OWNER.owner, OWNER.email)),
                    (
                        "Données collectées",
                        "Nom, téléphone, email (facultatif), nombre de participants et \
                         commentaire éventuel, saisis au moment de la réservation."
                            .to_owned(),
                    ),
                    (
                        "Finalité",
                        "Gérer la réservation et pouvoir vous joindre au sujet de la \
                         séance concernée."
                            .to_owned(),
                    ),
                    (
                        "Base légale",
                        "L'exécution du contrat (article 6.1.b du RGPD) : ces \
                         informations sont nécessaires pour tenir l'atelier. Elles ne \
                         sont jamais utilisées à d'autres fins, ni transmises à des \
                         tiers, ni vendues."
                            .to_owned(),
                    ),
                    ("Durée de conservation", RETENTION.to_owned()),
                    (
                        "Destinataires",
                        // The hosts are named here rather than pointed at: a visitor
                        // asking who else sees their phone number is owed the answer
                        // on the page they are reading.
                        format!(
                            "{} seule. Aucun autre destinataire, en dehors de {} et de {}, \
                             qui hébergent le site et les données.",
                            OWNER.owner, APP_HOST.name, DATA_HOST.name,
                        ),
                    ),
                    (
                        "Lieu d'hébergement",
                        format!(
                            "Site hébergé sur des serveurs situés à {}, données \
                             enregistrées sur des serveurs situés à {}.",
                            APP_HOST.location, DATA_HOST.location,
                        ),
                    ),
                    ("Transferts hors Union européenne", "Aucun.".to_owned()),
                ]/>
            </section>

            <section class="space-y-4">
                <h2 class="text-2xl font-semibold tracking-tight text-heading">"Vos droits"</h2>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Vous disposez d'un droit d'accès, de rectification, d'effacement,
                     d'opposition, de limitation et de portabilité sur les données qui
                     vous concernent."
                </p>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Pour les exercer, écrivez à "
                    <a
                        href=OWNER.email_link()
                        class="font-medium underline underline-offset-4 text-primary"
                    >
                        {OWNER.email}
                    </a>
                    ". Si la réponse ne vous convient pas, vous pouvez saisir la CNIL,
                     l'autorité française de protection des données, sur "
                    <a
                        href="https://www.cnil.fr"
                        target="_blank"
                        rel="noreferrer noopener"
                        class="font-medium underline underline-offset-4 text-primary"
                    >
                        "cnil.fr"
                    </a>
                    <span class="sr-only">" (nouvel onglet)"</span>
                    "."
                </p>
            </section>

            <section class="space-y-4">
                <h2 class="text-2xl font-semibold tracking-tight text-heading">"Cookies"</h2>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Ce site ne dépose aucun cookie publicitaire et n'utilise aucun
                     outil de mesure d'audience. Il ne suit pas votre navigation."
                </p>
                <p class="text-sm leading-relaxed text-muted-foreground">
                    "Un seul cookie existe : celui qui maintient la connexion à
                     l'espace d'administration, déposé uniquement après une
                     authentification. Strictement nécessaire au fonctionnement du
                     site, il est dispensé de consentement. Votre préférence de thème
                     clair ou sombre est gardée par votre navigateur et ne quitte pas
                     votre appareil."
                </p>
            </section>

            <p class="text-sm leading-relaxed text-muted-foreground">
                "L'identité de l'éditrice du site et les coordonnées complètes des
                 hébergeurs figurent dans les "
                <a
                    href="/mentions-legales"
                    class="font-medium underline underline-offset-4 text-primary"
                >
                    "mentions légales"
                </a>
                "."
            </p>
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
            provide_context(RequestUrl::new("/politique-de-confidentialite"));

            view! { <Router><PrivacyPolicyPage/></Router> }.to_html()
        })
    }

    /// Article 13 asks for the purpose, the legal basis, how long, and the way to
    /// complain. The legal basis is the one most often got wrong, so it is named.
    #[test]
    fn the_policy_answers_what_article_13_asks() {
        let html = page_html();

        assert!(html.contains("6.1.b"), "no legal basis: {html}");
        assert!(html.contains(RETENTION), "no retention period: {html}");
        assert!(html.contains("CNIL"), "no supervisory authority: {html}");
        assert!(html.contains("rectification"), "no list of rights: {html}");
    }

    /// Who else sees the data, and where it sits. Asserted against the constants
    /// rather than copies of them: the legal notice names the same two hosts, and a
    /// test holding its own spelling would let the two pages drift apart.
    #[test]
    fn the_policy_names_who_holds_the_data_and_where() {
        let html = page_html();

        assert!(html.contains(OWNER.owner), "no controller: {html}");
        assert!(html.contains(APP_HOST.name), "no host for the site: {html}");
        assert!(html.contains(DATA_HOST.name), "no host for the data: {html}");
        assert!(html.contains(APP_HOST.location), "no location for the site: {html}");
        assert!(html.contains(DATA_HOST.location), "no location for the data: {html}");
        assert!(html.contains("Aucun"), "nothing said about transfers: {html}");
    }

    /// The site sets one cookie and runs no tracker, which is what lets it do
    /// without a consent banner. Saying so is the other half of that position.
    #[test]
    fn the_policy_accounts_for_the_one_cookie_there_is() {
        let html = page_html();

        assert!(html.contains("Cookies"), "no cookie section: {html}");
        assert!(html.contains("administration"), "the one cookie is unexplained: {html}");
        assert!(html.contains("dispensé de consentement"), "nor why it needs no banner: {html}");
    }

    /// Writing to her is how every right is exercised, so the address has to be
    /// clickable rather than merely printed.
    #[test]
    fn the_rights_are_exercised_through_a_link_that_works() {
        let html = page_html();

        assert!(html.contains(&OWNER.email_link()), "the email does not open: {html}");
    }

    /// The two pages were one and are now two, so each has to point at the other:
    /// a visitor who landed here looking for who runs the site should not have to
    /// go back through the footer.
    #[test]
    fn the_policy_leads_on_to_the_legal_notice() {
        let html = page_html();

        assert!(html.contains(r#"href="/mentions-legales""#), "no way across: {html}");
    }
}
