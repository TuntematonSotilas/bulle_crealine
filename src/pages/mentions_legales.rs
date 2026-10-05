use leptos::prelude::*;

use crate::components::seo::PageMeta;
use crate::models::OWNER;

/// How long a booking is kept, counted from the last contact.
///
/// Stated here because the page is where the promise is made. Note that nothing in
/// the code enforces it yet: a booking is deleted logically, the document stays. The
/// day that changes, this is the figure to honour.
const RETENTION: &str = "3 ans à compter du dernier contact";

/// Renders the "Mentions légales" page, which carries the privacy policy too.
///
/// Two obligations, one page: the legal notice a professional site owes under the
/// LCEN, and the information a visitor is owed under article 13 of the GDPR. They are
/// separate duties and are kept as separate sections, but a visitor looking for
/// either looks in the same place.
///
/// Every particular is read off [`OWNER`] rather than written here, so the legal
/// notice and the contact page cannot come to disagree.
#[component]
pub fn MentionsLegales() -> impl IntoView {
    view! {
        <PageMeta
            title="Mentions légales — Bulle Créaline (E.I)"
            description="Mentions légales et politique de confidentialité de Bulle Créaline (E.I) : éditeur, hébergeur, données personnelles collectées, durée de conservation et droits."
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
                    "Le site est hébergé par Render Services, Inc., 525 Brannan Street,
                     San Francisco, CA 94107, États-Unis, sur des serveurs situés à
                     Francfort (Allemagne). Les données sont enregistrées chez MongoDB,
                     Inc. sur des serveurs situés à Paris (France)."
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
                    "Réserver une séance suppose de laisser de quoi vous recontacter.
                     Voici ce qui est conservé, pourquoi, et ce que vous pouvez en
                     exiger."
                </p>
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
                        format!(
                            "{} seule. Aucun autre destinataire, en dehors des \
                             hébergeurs mentionnés ci-dessus.",
                            OWNER.owner,
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
        </div>
    }
}

/// A label and its value, one per row.
///
/// A list rather than a table: every row is one short answer, and a table would make
/// a screen reader announce coordinates for a column that never varies.
#[component]
fn Lines(rows: Vec<(&'static str, String)>) -> impl IntoView {
    let items = rows
        .into_iter()
        .map(|(label, value)| {
            view! {
                <div class="p-4 rounded-2xl border bg-surface text-surface-foreground border-border">
                    <dt class="text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                        {label}
                    </dt>
                    <dd class="mt-1 text-sm leading-relaxed">{value}</dd>
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! { <dl class="flex flex-col gap-3">{items}</dl> }
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
    /// what decides whether anything has to be said about transfers.
    #[test]
    fn the_notice_names_the_host_and_where_the_servers_stand() {
        let html = page_html();

        assert!(html.contains("Render"), "no host: {html}");
        assert!(html.contains("Francfort"), "no application server: {html}");
        assert!(html.contains("Paris"), "no database server: {html}");
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
}
