use leptos::prelude::*;

use crate::components::blocks::studio_place::StudioPlace;
use crate::components::ui::service_icon::ServiceIcon;
use crate::models::ServiceView;

/// One workshop, as its own page presents it.
///
/// Everything shown comes off the [`ServiceView`]: the page behind it is a single
/// route matching `/services/:slug` and `/pro/:slug`, so nothing here may depend
/// on which workshop it is.
#[component]
pub fn ServiceBlock(service: ServiceView) -> impl IntoView {
    // A workshop run for a structure agrees on its dates directly, so it has no
    // booking page -- and is not held at the studio either.
    let bookable = service.booking_path().is_some();

    // Left out rather than shown empty: a workshop open to everyone has no age to
    // announce, and an empty card would read as missing information.
    let age = (!service.age.is_empty()).then(|| {
        view! {
            <div class="grid grid-cols-1 gap-4 mb-6">
                <div class="p-4 rounded-2xl border bg-surface text-surface-foreground border-border">
                    <div class="text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                        "Âge requis"
                    </div>
                    <div class="mt-1 font-medium">{service.age.clone()}</div>
                </div>
            </div>
        }
    });

    // Only where the visitor is the one who travels. A workshop run for a
    // structure is held at the structure's address, so announcing the studio's
    // would not merely be unhelpful -- it would be wrong.
    let place = bookable.then(|| {
        view! {
            <div class="mb-6">
                <StudioPlace/>
            </div>
        }
    });

    let steps = (!service.steps.is_empty()).then(|| {
        let items = service
            .steps
            .iter()
            .enumerate()
            .map(|(rank, step)| {
                view! {
                    <div class="flex gap-3 items-start">
                        <div class="flex flex-shrink-0 justify-center items-center w-6 h-6 text-sm font-semibold text-white rounded-full bg-gradient-to-br from-primary to-secondary">
                            {rank + 1}
                        </div>
                        <p class="text-sm text-muted-foreground pt-0.5">{step.clone()}</p>
                    </div>
                }
            })
            .collect::<Vec<_>>();

        view! {
            <div class="mb-6">
                <h4 class="mb-4 text-lg font-bold text-heading">"Déroulement de l'atelier"</h4>
                <div class="space-y-3">{items}</div>
            </div>
        }
    });

    view! {
        <article class="rounded-[2rem] border border-border shadow-(--shadow) text-card-foreground overflow-hidden">
            <div class="flex flex-col gap-0 lg:flex-row">

                <div class="lg:w-[1rem] flex-shrink-0 bg-primary/50"></div>

                <div class="flex flex-col flex-1 p-6 lg:p-8">

                    <div class="mb-6">
                        // The picto sits beside the title rather than above it: it
                        // stands for the workshop, and a mark floating on its own
                        // line reads as decoration.
                        <div class="flex gap-3 items-center mb-3">
                            <ServiceIcon
                                name=service.icon.clone()
                                class="w-8 h-8 shrink-0 text-heading-soft"
                            />
                            <h3 class="text-2xl font-bold lg:text-3xl text-heading">
                                {service.label.clone()}
                            </h3>
                        </div>
                        <span class="block mb-3 w-10 h-1 rounded-full bg-heading-soft"></span>
                        <p class="text-base leading-relaxed text-muted-foreground">
                            {service.description.clone()}
                        </p>
                    </div>

                    {age} {place} {steps}

                </div>
            </div>
        </article>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn service(pro: bool) -> ServiceView {
        ServiceView {
            id: "651d1f0a0000000000000001".to_owned(),
            slug: "aperos-creatifs".to_owned(),
            label: "Apéros créatifs (adultes)".to_owned(),
            description: "Une soirée entre adultes autour d'un verre.".to_owned(),
            age: "À partir de 18 ans".to_owned(),
            steps: vec!["Accueil".to_owned(), "Réalisation du projet".to_owned()],
            icon: "Wine".to_owned(),
            pro,
            position: 0,
            min_persons: 1,
        }
    }

    fn block_html(service: ServiceView) -> String {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        Owner::new().with(|| {
            // The booking button is a link, and a link resolves `aria-current`
            // against the location, which server-side is the request being rendered.
            provide_context(RequestUrl::new("/services/aperos-creatifs"));

            view! {
                <Router>
                    <ServiceBlock service=service/>
                </Router>
            }
            .to_html()
        })
    }

    #[test]
    fn a_workshop_shows_its_name_description_age_and_run_through() {
        let html = block_html(service(false));

        assert!(html.contains("Apéros créatifs (adultes)"), "no name: {html}");
        assert!(html.contains("Une soirée entre adultes"), "no description: {html}");
        assert!(html.contains("À partir de 18 ans"), "no age: {html}");
        assert!(html.contains("Déroulement de l'atelier"), "no run-through: {html}");
        assert!(html.contains("Réalisation du projet"), "no step: {html}");
        // Named rather than counted as "an svg": the studio address draws one too,
        // and a bare `<svg` check would pass on it.
        assert!(html.contains(r#"data-name="Wine""#), "no picto: {html}");
    }

    /// Booking left this block for the schedule below it, where each date carries
    /// its own button. A single "S'inscrire" here only led to a page listing the
    /// same dates again, one step further away.
    #[test]
    fn the_presentation_offers_no_booking_of_its_own() {
        for pro in [false, true] {
            let html = block_html(service(pro));

            assert!(!html.contains("/booking/"), "booking belongs below: {html}");
            assert!(!html.contains("S'inscrire"), "the button should be gone: {html}");
        }
    }

    /// The studio address belongs to the workshops a visitor travels to. One run
    /// for a structure is held at the structure's, so showing this one would be a
    /// wrong answer rather than a missing one.
    #[test]
    fn only_a_bookable_workshop_announces_the_studio() {
        use crate::models::STUDIO_ADDRESS;

        assert!(
            block_html(service(false)).contains(STUDIO_ADDRESS),
            "the address should show on a bookable workshop"
        );
        assert!(
            !block_html(service(true)).contains(STUDIO_ADDRESS),
            "a workshop for structures is not held there"
        );
    }

    /// The fields are optional in the admin form, so a workshop may legitimately
    /// carry neither; the headings must then go too, rather than stand over
    /// nothing.
    #[test]
    fn leaves_out_what_the_workshop_does_not_carry() {
        let mut bare = service(false);
        bare.age = String::new();
        bare.steps = Vec::new();
        bare.icon = String::new();

        let html = block_html(bare);

        assert!(!html.contains("Âge requis"), "an empty age should not show: {html}");
        assert!(
            !html.contains("Déroulement de l'atelier"),
            "an empty run-through should not show: {html}"
        );
        assert!(
            !html.contains(r#"data-name="Wine""#),
            "an empty picto should draw nothing: {html}"
        );
        assert!(html.contains("Apéros créatifs (adultes)"), "the name still shows: {html}");
    }
}
