use leptos::prelude::*;

use crate::components::blocks::studio_place::StudioPlace;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
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
    //
    // And left out of a workshop run for a structure whatever it carries: the age
    // is a condition of signing up, and there is no signing up here -- the group is
    // whoever the structure brings.
    let age = (bookable && !service.age.is_empty()).then(|| {
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

    // Gated on `bookable` for the same reason as the age above: how a session
    // unfolds is agreed with the structure, so the atelier's own run-through would
    // be a promise made about someone else's.
    let steps = (bookable && !service.steps.is_empty()).then(|| {
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
                <h2 class="mb-4 text-lg font-bold text-heading">"Déroulement de l'atelier"</h2>
                <div class="space-y-3">{items}</div>
            </div>
        }
    });

    // The counterpart of the schedule a bookable workshop carries below it: there
    // is nothing to sign up for here, and agreeing on a date is a conversation.
    let contact = (!bookable).then(|| {
        view! {
            // Spaced by hand: the three slots above are all empty on this page, so
            // the button would otherwise sit flush under the description.
            <div class="mt-2">
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Pill
                    href="/contact"
                >
                    "Me contacter"
                </Button>
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
                            // The page's `h1`, not a card's heading: `ServicePage`
                            // is this block's only caller, and the catalogue draws
                            // its own tiles. Every size here comes from a class,
                            // and Tailwind's preflight flattens the tags, so the
                            // level is free to say what it means.
                            <h1 class="text-2xl font-bold lg:text-3xl text-heading">
                                {service.label.clone()}
                            </h1>
                        </div>
                        <span class="block mb-3 w-10 h-1 rounded-full bg-heading-soft"></span>
                        <p class="text-base leading-relaxed text-muted-foreground">
                            {service.description.clone()}
                        </p>
                    </div>

                    {age} {place} {steps} {contact}

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

    /// The workshop's name is the page's own heading, not a card's inside a list.
    /// `ServicePage` renders this block and nothing else does -- the catalogue draws
    /// its own tiles -- so there is no second context to share the level with.
    #[test]
    fn the_workshop_name_is_the_page_heading() {
        let html = block_html(service(false));

        assert_eq!(html.matches("<h1").count(), 1, "not exactly one h1: {html}");
        assert!(html.contains("Apéros créatifs (adultes)"), "{html}");
    }

    /// A run-through belongs one level under the name, not three. The page used to
    /// open at `h3` and reach `h4` with nothing in between.
    #[test]
    fn the_run_through_sits_directly_under_the_page_heading() {
        let html = block_html(service(false));

        assert!(html.contains("<h2"), "the run-through should be an h2: {html}");
        assert!(!html.contains("<h4"), "nothing here should reach h4: {html}");
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
        use crate::models::OWNER;

        assert!(
            block_html(service(false)).contains(OWNER.address),
            "the address should show on a bookable workshop"
        );
        assert!(
            !block_html(service(true)).contains(OWNER.address),
            "a workshop for structures is not held there"
        );
    }

    /// Both rubrics answer a question nobody asks here. The age is a condition of
    /// signing up, and there is no signing up; the run-through is the atelier's own,
    /// where a session in a structure is agreed with that structure.
    ///
    /// Asserted on a workshop that *carries* both, which is what tells this gate
    /// apart from the emptiness one below: a fixture with neither would pass either
    /// way.
    #[test]
    fn a_workshop_for_structures_drops_the_age_and_the_run_through() {
        let shown = service(true);
        assert!(!shown.age.is_empty() && !shown.steps.is_empty(), "the fixture carries both");

        let html = block_html(shown);

        assert!(!html.contains("Âge requis"), "the age should not show: {html}");
        assert!(
            !html.contains("Déroulement de l'atelier"),
            "nor the run-through: {html}"
        );
        assert!(html.contains("Apéros créatifs (adultes)"), "the name still shows: {html}");
    }

    /// There is no date to sign up for, so the page has to end somewhere: getting
    /// in touch is the one thing left to offer.
    #[test]
    fn a_workshop_for_structures_offers_a_way_to_get_in_touch() {
        let html = block_html(service(true));

        assert!(html.contains("Me contacter"), "no way to get in touch: {html}");
        assert!(html.contains(r#"href="/contact""#), "and nowhere to go: {html}");
    }

    /// The mirror of the two above. Without it an inverted gate would pass the
    /// whole suite: every other test here either loops without asserting on these
    /// or reads a bookable workshop only.
    #[test]
    fn a_bookable_workshop_keeps_its_rubrics_and_offers_no_contact() {
        let html = block_html(service(false));

        assert!(html.contains("Âge requis"), "the age belongs here: {html}");
        assert!(html.contains("Déroulement de l'atelier"), "and the run-through: {html}");
        assert!(!html.contains("Me contacter"), "booking is the way in here: {html}");
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
