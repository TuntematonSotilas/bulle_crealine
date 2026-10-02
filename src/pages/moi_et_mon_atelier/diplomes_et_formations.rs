use leptos::prelude::*;

/// How much room a qualification is given, and so how much weight it carries.
///
/// Two steps rather than a scale: the page says what it is built on and what rounds
/// that out, and a third size would be a distinction nobody reading it would catch.
#[derive(Clone, Copy, PartialEq)]
enum Size {
    /// The qualifications the workshops rest on.
    Medium,
    /// The ones that round out the picture.
    Small,
}

impl Size {
    /// Room around the card's contents.
    fn padding(self) -> &'static str {
        match self {
            Size::Medium => "p-8",
            Size::Small => "p-5",
        }
    }

    /// The qualification's own line, which is what the eye lands on.
    ///
    /// Deliberately not sharing a value with [`Size::aside`]: the two would then be
    /// indistinguishable in the rendered markup, and the test that counts how many
    /// cards are drawn large would be counting lead-ins as well.
    fn title(self) -> &'static str {
        match self {
            Size::Medium => "text-2xl",
            Size::Small => "text-lg",
        }
    }

    /// The lead-in above it and the qualifier below.
    fn aside(self) -> &'static str {
        match self {
            Size::Medium => "text-base",
            Size::Small => "text-xs",
        }
    }
}

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
    /// How much of the page it is given.
    size: Size,
}

/// Renders the "Diplômes et formations" page.
#[component]
pub fn DiplomesEtFormationsPage() -> impl IntoView {
    // Wording follows design/maquette-diplomes.png, with the accents and
    // spelling restored; the acronyms are left exactly as written there.
    //
    // The order is the layout. The grid is two columns wide, so these fall into
    // three rows by themselves -- the two the workshops rest on, the two that
    // explain how they are run, and the two that round out the picture. Placing
    // each one explicitly would have to be undone the first time a seventh is
    // added.
    let formations = [
        Formation {
            lead: "Formation",
            title: "Médiation artistique",
            detail: "En relation d'aide",
            acronym: "",
            size: Size::Medium,
        },
        Formation {
            lead: "Diplôme d'",
            title: "Animatrice sociale",
            detail: "",
            acronym: "BP JEPS AS",
            size: Size::Medium,
        },
        Formation {
            lead: "Bac pro",
            title: "Communication",
            detail: "Gestion humaine",
            acronym: "",
            size: Size::Medium,
        },
        Formation {
            lead: "Formation",
            title: "Communication non verbale",
            detail: "",
            acronym: "",
            size: Size::Medium,
        },
        Formation {
            lead: "Fac d'histoire",
            title: "Histoire de l'art",
            detail: "",
            acronym: "",
            size: Size::Small,
        },
        Formation {
            lead: "Diplôme d'",
            title: "Accompagnant médico-social",
            detail: "",
            acronym: "DAES",
            size: Size::Small,
        },
    ];

    let cards = formations
        .into_iter()
        .map(|formation| {
            let size = formation.size;

            view! {
                <li class=format!(
                    "flex flex-col items-center gap-2 rounded-[2rem] border border-border bg-card text-center shadow-sm {}",
                    size.padding(),
                )>
                    <p class=format!("text-muted-foreground {}", size.aside())>
                        {formation.lead}
                    </p>
                    <p class=format!("font-semibold text-heading {}", size.title())>
                        {formation.title}
                    </p>
                    {(!formation.detail.is_empty())
                        .then(|| view! {
                            <p class=format!("text-muted-foreground {}", size.aside())>
                                {formation.detail}
                            </p>
                        })}
                    {(!formation.acronym.is_empty())
                        .then(|| view! {
                            <span class="mt-1 rounded-full bg-surface px-3 py-1 text-xs font-semibold tracking-wide text-surface-foreground">
                                {formation.acronym}
                            </span>
                        })}
                </li>
            }
            .into_any()
        })
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

            // Two columns rather than three: the rows are the hierarchy, and a
            // third column would scatter them.
            <ul class="grid gap-6 sm:grid-cols-2">
                {cards}
            </ul>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// The qualifications in the order the page is meant to show them.
    const ORDER: [&str; 6] = [
        "Médiation artistique",
        "Animatrice sociale",
        "Communication non verbale",
        "Histoire de l'art",
        "Accompagnant médico-social",
        // "Communication" is left out of this list on purpose: it is a prefix of
        // "Communication non verbale", so looking for it would find whichever of
        // the two comes first and prove nothing. Its place is pinned below.
        "DAES",
    ];

    fn page_html() -> String {
        Owner::new().with(|| view! { <DiplomesEtFormationsPage/> }.to_html())
    }

    /// The order is the layout: the grid is two columns wide and the rows fall out
    /// of it, so a reordered array silently rearranges the page.
    #[test]
    fn the_qualifications_come_out_in_the_order_the_rows_need() {
        let html = page_html();

        let positions: Vec<usize> = ORDER
            .iter()
            .take(5)
            .map(|title| {
                html.find(title)
                    .unwrap_or_else(|| panic!("{title} is missing: {html}"))
            })
            .collect();

        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "the cards were reordered: {positions:?}"
        );
    }

    /// "Communication" opens the second row, between the two cards above it and the
    /// two below. Checked on the one occurrence that is not the start of
    /// "Communication non verbale".
    #[test]
    fn the_bac_pro_opens_the_second_row() {
        let html = page_html();

        let lead = html.find("Bac pro").expect("the lead-in should render");
        let before = html.find("Animatrice sociale").expect("the first row should render");
        let after = html
            .find("Communication non verbale")
            .expect("its neighbour should render");

        assert!(before < lead && lead < after, "out of place: {before}, {lead}, {after}");
    }

    /// Four cards carry the page, two round it out -- which is the whole of what was
    /// asked for, and invisible to the order test above.
    #[test]
    fn the_last_row_is_drawn_smaller_than_the_ones_above() {
        let html = page_html();

        assert_eq!(
            html.matches(Size::Medium.title()).count(),
            4,
            "there should be four cards at the larger size: {html}"
        );
        assert_eq!(
            html.matches(Size::Small.title()).count(),
            2,
            "and two at the smaller: {html}"
        );
    }

    /// Reordering an array is an easy way to drop a line out of it. Every
    /// qualification has to survive the move, acronyms included.
    #[test]
    fn every_qualification_is_still_on_the_page() {
        let html = page_html();

        for title in ORDER {
            assert!(html.contains(title), "{title} is missing: {html}");
        }
        assert!(html.contains("BP JEPS AS"), "an acronym went missing: {html}");
        assert_eq!(html.matches("<li").count(), 6, "not six cards: {html}");
    }
}
