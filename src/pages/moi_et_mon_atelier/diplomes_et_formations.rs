use leptos::prelude::*;

/// One entry of the qualifications list.
struct Formation {
    /// Small lead-in shown above the qualification, e.g. "Diplôme d'".
    lead: &'static str,
    /// The qualification itself.
    title: &'static str,
    /// Qualifier shown under the title; empty when there is none.
    detail: &'static str,
    /// Acronym the qualification is known by; empty when there is none.
    acronym: &'static str,
}

/// Renders the "Diplômes et formations" page.
#[component]
pub fn DiplomesEtFormationsPage() -> impl IntoView {
    // Wording follows design/maquette-diplomes.png, with the accents and
    // spelling restored; the acronyms are left exactly as written there.
    let formations = [
        Formation {
            lead: "Bac pro",
            title: "Communication",
            detail: "Gestion humaine",
            acronym: "",
        },
        Formation {
            lead: "Formation",
            title: "Communication non verbale",
            detail: "",
            acronym: "",
        },
        Formation {
            lead: "Formation",
            title: "Médiation artistique",
            detail: "En relation d'aide",
            acronym: "",
        },
        Formation {
            lead: "Fac d'histoire",
            title: "Histoire de l'art",
            detail: "",
            acronym: "",
        },
        Formation {
            lead: "Diplôme d'",
            title: "Accompagnant médico-social",
            detail: "",
            acronym: "DAES",
        },
        Formation {
            lead: "Diplôme d'",
            title: "Animatrice sociale",
            detail: "",
            acronym: "BP JEPS AS",
        },
    ];

    let cards = formations
        .into_iter()
        .map(|formation| view! {
            <li class="flex flex-col items-center gap-2 rounded-[2rem] border border-border bg-card p-6 text-center shadow-sm">
                <p class="text-sm text-muted-foreground">{formation.lead}</p>
                <p class="text-lg font-semibold text-heading">{formation.title}</p>
                {(!formation.detail.is_empty())
                    .then(|| view! {
                        <p class="text-sm text-muted-foreground">{formation.detail}</p>
                    })}
                {(!formation.acronym.is_empty())
                    .then(|| view! {
                        <span class="mt-1 rounded-full bg-surface px-3 py-1 text-xs font-semibold tracking-wide text-surface-foreground">
                            {formation.acronym}
                        </span>
                    })}
            </li>
        }.into_any())
        .collect::<Vec<_>>();

    view! {
        <div class="mx-auto max-w-6xl space-y-10 py-6">
            <div class="space-y-4 text-center">
                <p class="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
                    "Quelques mots sur mes"
                </p>
                <h1 class="text-4xl md:text-5xl font-semibold tracking-tight text-heading">
                    "Diplômes et formations"
                </h1>
                <span class="mx-auto block h-1 w-10 rounded-full bg-heading-soft"></span>
            </div>

            <ul class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
                {cards}
            </ul>
        </div>
    }
}
