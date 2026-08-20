use icons::MapPin;
use leptos::prelude::*;
use crate::components::ui::button::Button;
use crate::models::ServiceType;

#[component]
pub fn ServiceBlock(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] schedule: String,
    #[prop(into)] place: String,
    #[prop(into)] age: String,
    #[prop(into)] place_link: String,
    #[prop(optional)] steps: Vec<String>,
    /// Which workshop this block describes. Without one there is neither a
    /// booking page to point at nor sessions to list, so both are left out.
    #[prop(optional)] service: Option<ServiceType>,
) -> impl IntoView {

    let register_button = service.map(|service| view! {
        <div class="mt-auto">
            <Button class="w-full md:w-auto" href=format!("/booking/{}", service.slug())>
                "S'inscrire"
            </Button>
        </div>
    });

    let steps_view = steps
        .into_iter()
        .enumerate()
        .map(|(idx, step)| view! { 
            <div class="flex gap-3 items-start">
                <div class="flex-shrink-0 w-6 h-6 rounded-full bg-gradient-to-br from-primary to-secondary flex items-center justify-center text-white text-sm font-semibold">
                    {idx + 1}
                </div>
                <p class="text-sm text-muted-foreground pt-0.5">{step}</p>
            </div>
        }.into_any())
        .collect::<Vec<_>>();

    view! {
        <article class="rounded-[2rem] border border-border shadow-(--shadow) text-card-foreground overflow-hidden">
            <div class="flex flex-col lg:flex-row gap-0">

                /* Left Border */
                <div class="lg:w-[1rem] flex-shrink-0 bg-primary/50">
                </div>

                /* Content Section */
                <div class="flex-1 p-6 lg:p-8 flex flex-col">
                    /* Header */
                    <div class="mb-6">
                        <h3 class="text-2xl lg:text-3xl font-bold mb-3 text-heading">
                            {title.clone()}
                        </h3>
                        <span class="block h-1 w-10 rounded-full bg-heading-soft mb-3"></span>
                        <p class="text-base leading-relaxed text-muted-foreground">{description}</p>
                    </div>

                    /* Register Button */
                    <div class="mb-6">
                        {register_button}
                    </div>

                        /* Info Cards */
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                            <div class="p-4 bg-surface text-surface-foreground rounded-2xl border border-border">
                                <div class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Horaire</div>
                                <div class="font-medium mt-1">{schedule}</div>
                            </div>
                            <div class="p-4 bg-surface text-surface-foreground rounded-2xl border border-border">
                                <div class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Âge requis</div>
                                <div class="font-medium mt-1">{age}</div>
                            </div>
                            <div class="p-4 bg-surface text-surface-foreground rounded-2xl border border-border md:col-span-2">
                                <div class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Lieu</div>
                                <a href={place_link} target="_blank" rel="noreferrer" class="font-medium mt-1 inline-flex items-center gap-2 hover:text-heading transition-colors">
                                    <MapPin class="w-4 h-4 flex-shrink-0" />
                                    <span class="underline">{place}</span>
                                </a>
                            </div>
                        </div>


                    /* Steps */
                    <div class="mb-6">
                        <h4 class="text-lg font-bold mb-4 text-heading">
                            {"Déroulement de l'atelier"}
                        </h4>
                        <div class="space-y-3">
                            {steps_view}
                        </div>
                    </div>
            
                </div>
            </div>
        </article>
    }.into_any()
}
