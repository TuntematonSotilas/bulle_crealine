use icons::MapPin;
use leptos::either::EitherOf3;
use leptos::prelude::*;
use crate::api::sessions::upcoming_sessions;
use crate::auth::user_message;
use crate::components::ui::{button::Button, carousel::*};
use crate::models::ServiceType;

pub struct DetPerAge {
    pub prefix: String,
    pub age: String,
    pub description: String,
}

#[component]
pub fn ServiceBlock(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(optional, default = Vec::new())] details_per_age: Vec<DetPerAge>,
    #[prop(into)] schedule: String,
    #[prop(into)] place: String,
    #[prop(into)] age: String,
    #[prop(into)] place_link: String,
    #[prop(optional)] steps: Vec<String>,
    #[prop(into)] pictures: Vec<String>,
    /// Which workshop this block describes. Without one there is neither a
    /// booking page to point at nor sessions to list, so both are left out.
    #[prop(optional)] service: Option<ServiceType>,
) -> impl IntoView {

    let sessions_section = service.map(|service| view! { <UpcomingSessions service=service/> });

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
                <p class="text-sm text-[var(--gray-text)] pt-0.5">{step}</p>
            </div>
        }.into_any())
        .collect::<Vec<_>>();

    let carousel_view = pictures
        .into_iter()
        .map(|step| view! { 
            <CarouselItem>
                <img src={step} />
            </CarouselItem>
        }.into_any())
        .collect::<Vec<_>>();

    let details_per_age_view = details_per_age
        .into_iter()
        .map(|d| view! {
            <li class="text-sm">
                {d.prefix}
                <span class="font-semibold">{d.age}</span>
                {d.description}
            </li>
        }.into_any())
        .collect::<Vec<_>>();

    view! {
        <article class="rounded-xl border border-(--border) shadow-(var(--shadow)) bg-[var(--card)] text-[var(--card-foreground)] overflow-hidden">
            <div class="flex flex-col lg:flex-row gap-0">

                /* Left Border */
                <div class="lg:w-[1rem] flex-shrink-0 bg-[var(--primary)]/50">
                </div>

                /* Content Section */
                <div class="flex-1 p-6 lg:p-8 flex flex-col">
                    /* Header */
                    <div class="mb-6">
                        <h3 class="text-2xl lg:text-3xl font-bold mb-3">{title.clone()}</h3>
                        <p class="text-base leading-relaxed">{description}</p>
                        <ul class="mt-4 ml-4 list-disc space-y-2">
                            {details_per_age_view}
                        </ul>
                    </div>

                    /* Info Cards + Carousel */
                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                        /* Info Cards */
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="p-4 bg-[var(--primary)] rounded-lg border border-slate-200 dark:border-slate-700">
                                <div class="text-xs font-semibold text-[var(--gray-text-light)] uppercase tracking-wide">Horaire</div>
                                <div class="text-slate-900 dark:text-white font-medium mt-1">{schedule}</div>
                            </div>
                            <div class="p-4 bg-[var(--primary)] rounded-lg border border-slate-200 dark:border-slate-700">
                                <div class="text-xs font-semibold text-[var(--gray-text-light)] uppercase tracking-wide">Âge requis</div>
                                <div class="text-slate-900 dark:text-white font-medium mt-1">{age}</div>
                            </div>
                            <div class="p-4 bg-[var(--primary)] rounded-lg border border-slate-200 dark:border-slate-700 md:col-span-2">
                                <div class="text-xs font-semibold text-[var(--gray-text-light)] uppercase tracking-wide">Lieu</div>
                                <a href={place_link} target="_blank" rel="noreferrer" class="text-slate-900 dark:text-white font-medium mt-1 inline-flex items-center gap-2 hover:text-secondary transition-colors">
                                    <MapPin class="w-4 h-4 flex-shrink-0" />
                                    <span class="underline">{place}</span>
                                </a>
                            </div>
                        </div>

                        /* Carousel */
                        <div class="flex justify-center items-center px-12">
                            <Carousel class="w-full max-w-xs">
                                <CarouselContent>
                                    {carousel_view}
                                </CarouselContent>
                                <CarouselPrevious />
                                <CarouselNext />
                            </Carousel>
                        </div>
                    </div>

                    /* Steps */
                    <div class="mb-6">
                        <h4 class="text-lg font-bold mb-4">{"Déroulement de l'atelier"}</h4>
                        <div class="space-y-3">
                            {steps_view}
                        </div>
                    </div>
            
                    /* Sessions */
                    {sessions_section}

                    /* Register Button */
                    {register_button}
                </div>
            </div>
        </article>
    }.into_any()
}

/// The dates open to booking for this workshop, read from the database.
///
/// Only the sessions still to come are listed, and each carries what is left of
/// its capacity, so the block and the booking page never disagree.
#[component]
fn UpcomingSessions(service: ServiceType) -> impl IntoView {
    let sessions = Resource::new(
        move || service.slug().to_owned(),
        |slug| async move { upcoming_sessions(slug).await },
    );

    view! {
        <div class="mb-6">
            <h4 class="text-lg font-bold mb-4">{"Prochaines séances"}</h4>

            <Transition fallback=|| {
                view! {
                    <p class="text-sm text-[var(--gray-text)]">"Chargement des séances…"</p>
                }
            }>
                {move || Suspend::new(async move {
                    match sessions.await {
                        // The dates are a nicety on a page that mostly presents the
                        // workshop, so a storage failure says so and lets the rest be.
                        Err(error) => {
                            EitherOf3::A(
                                view! {
                                    <p class="text-sm text-[var(--gray-text)]">
                                        {user_message(&error)}
                                    </p>
                                },
                            )
                        }
                        Ok(available) if available.is_empty() => {
                            EitherOf3::B(
                                view! {
                                    <p class="text-sm text-[var(--gray-text)]">
                                        "Aucune date n'est programmée pour le moment."
                                    </p>
                                },
                            )
                        }
                        Ok(available) => {
                            EitherOf3::C(
                                view! {
                                    <div class="space-y-3">
                                        {available
                                            .into_iter()
                                            .map(|session| {
                                                let price = session.price_label();
                                                let availability = session.availability_label();
                                                let full = session.is_full();

                                                view! {
                                                    <div class="p-4 border-l-4 border-primary bg-gradient-to-r from-primary/10 dark:from-primary/20 to-transparent rounded-r-lg hover:shadow-md dark:hover:shadow-primary/20 transition-shadow">
                                                        <div class="font-semibold text-[var(--gray-text)]">
                                                            {session.date_label}
                                                        </div>
                                                        <div class="text-sm text-[var(--gray-text)] mt-1">
                                                            "Thème : "{session.theme}
                                                        </div>
                                                        <div class="flex flex-wrap gap-x-3 items-baseline mt-2">
                                                            <span class="text-sm font-medium">"Prix : "{price}</span>
                                                            <span class=if full {
                                                                "text-sm font-medium text-destructive"
                                                            } else {
                                                                "text-sm text-[var(--gray-text)]"
                                                            }>{availability}</span>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                },
                            )
                        }
                    }
                })}
            </Transition>
        </div>
    }
}