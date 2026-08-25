use icons::Check;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::bookings::CreateBooking;
use crate::api::sessions::upcoming_sessions;
use crate::auth::user_message;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;
use crate::components::ui::number_field::NumberField;
use crate::components::ui::textarea::Textarea;
use crate::models::{MAX_PERSONS_PER_BOOKING, ServiceType, SessionView};

/// Booking page for one kind of workshop, at `/booking/<slug>`.
#[component]
pub fn BookingPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("service").unwrap_or_default();

    // Re-runs when the slug changes, so moving between workshop kinds does not
    // leave the previous list on screen.
    let sessions = Resource::new(slug, |slug| async move { upcoming_sessions(slug).await });

    view! {
        <Title text="Réserver un atelier — Bulle Créaline"/>

        {move || match ServiceType::from_slug(&slug()) {
            None => Either::Left(view! { <UnknownService/> }),
            Some(service) => {
                Either::Right(
                    view! {
                        <div class="flex flex-col gap-6 mx-auto max-w-2xl">
                            <div>
                                <h1 class="text-2xl font-semibold">"Réserver"</h1>
                                <p class="text-muted-foreground">{service.label()}</p>
                            </div>

                            <Transition fallback=|| {
                                view! {
                                    <p class="text-sm text-muted-foreground">
                                        "Chargement des séances…"
                                    </p>
                                }
                            }>
                                {move || Suspend::new(async move {
                                    match sessions.await {
                                        Err(error) => {
                                            Either::Left(
                                                view! {
                                                    <Alert variant=AlertVariant::Destructive>
                                                        {user_message(&error)}
                                                    </Alert>
                                                },
                                            )
                                        }
                                        Ok(available) => {
                                            Either::Right(
                                                view! { <BookingForm service=service sessions=available/> },
                                            )
                                        }
                                    }
                                })}
                            </Transition>
                        </div>
                    },
                )
            }
        }}
    }
}

/// Shown when the URL carries a workshop kind that does not exist.
#[component]
fn UnknownService() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-2xl">
            <Alert variant=AlertVariant::Destructive>
                <AlertTitle>"Atelier inconnu"</AlertTitle>
                <AlertDescription>
                    "Cette page de réservation n'existe pas. "
                    <a href="/" class="underline underline-offset-4">"Revenir à l'accueil"</a>
                    "."
                </AlertDescription>
            </Alert>
        </div>
    }
}

/// The session picker and the visitor's details.
#[component]
fn BookingForm(service: ServiceType, sessions: Vec<SessionView>) -> impl IntoView {
    let booking = ServerAction::<CreateBooking>::new();
    let pending = booking.pending();
    let result = booking.value();

    let error = move || {
        result
            .get()
            .and_then(|outcome| outcome.err())
            .map(|error| user_message(&error))
    };
    let confirmed = move || result.get().and_then(|outcome| outcome.ok());

    let bookable = sessions.iter().filter(|session| !session.is_full()).count();

    if sessions.is_empty() {
        return Either::Left(view! {
            <Alert>
                <AlertTitle>"Aucune séance programmée"</AlertTitle>
                <AlertDescription>
                    "Il n'y a pas de date à venir pour cet atelier. "
                    <a href=service.page_path() class="underline underline-offset-4">
                        "Voir la présentation de l'atelier"
                    </a>
                    "."
                </AlertDescription>
            </Alert>
        });
    }

    let choices = sessions
        .into_iter()
        .enumerate()
        .map(|(index, session)| {
            let full = session.is_full();
            let input_id = format!("session-{index}");
            let label_for = input_id.clone();

            view! {
                <label
                    r#for=label_for
                    class="flex gap-3 items-start p-4 rounded-lg border transition-colors cursor-pointer has-[:checked]:border-primary has-[:checked]:bg-primary/5 has-[:disabled]:opacity-60 has-[:disabled]:cursor-not-allowed"
                >
                    <input
                        type="radio"
                        id=input_id
                        name="session_id"
                        value=session.id.clone()
                        required=true
                        disabled=full
                        class="mt-1 accent-primary"
                    />
                    <span class="flex flex-col gap-1">
                        <span class="font-medium">{session.date_label.clone()}</span>
                        <span class="text-sm text-muted-foreground">
                            "Thème : "{session.theme_name.clone()}" · "{session.price_label()}
                        </span>
                        <span class=move || {
                            if full {
                                "text-sm font-medium text-destructive"
                            } else {
                                "text-sm text-muted-foreground"
                            }
                        }>{session.availability_label()}</span>
                    </span>
                </label>
            }
        })
        .collect::<Vec<_>>();

    Either::Right(view! {
        {move || {
            confirmed()
                .map(|session_label| {
                    view! { <BookingConfirmed service=service session_label=session_label/> }
                })
        }}

        // Hidden rather than unmounted: the session list is built once from the
        // loaded sessions, so it cannot be rebuilt from inside a reactive closure.
        <div class=move || if confirmed().is_some() { "hidden" } else { "" }>
            <Card>
                <CardHeader>
                    <CardTitle>"Choisissez votre séance"</CardTitle>
                    <CardDescription>
                        {move || {
                            match bookable {
                                0 => "Toutes les séances à venir sont complètes.".to_owned(),
                                1 => "Une séance est encore ouverte.".to_owned(),
                                open => format!("{open} séances sont encore ouvertes."),
                            }
                        }}
                    </CardDescription>
                </CardHeader>

                <CardContent>
                    <ActionForm action=booking>
                        <div class="flex flex-col gap-6">

                            <div class="flex flex-col gap-3">{choices}</div>

                            <div class="grid gap-4 md:grid-cols-2">
                                <div class="grid gap-3">
                                    <Label r#for="name">"Nom"</Label>
                                    <Input id="name" name="name" autocomplete="name" required=true/>
                                </div>

                                <div class="grid gap-3">
                                    <Label r#for="phone">"Téléphone"</Label>
                                    <Input
                                        r#type=InputType::Tel
                                        id="phone"
                                        name="phone"
                                        autocomplete="tel"
                                        required=true
                                    />
                                </div>

                                <div class="grid gap-3">
                                    <Label r#for="email">"Adresse e-mail (facultatif)"</Label>
                                    <Input
                                        r#type=InputType::Email
                                        id="email"
                                        name="email"
                                        placeholder="vous@exemple.fr"
                                        autocomplete="email"
                                    />
                                </div>

                                <div class="grid gap-3">
                                    <Label r#for="persons">"Nombre de personnes"</Label>
                                    <NumberField
                                        id="persons"
                                        name="persons"
                                        min=1.0
                                        max=f64::from(MAX_PERSONS_PER_BOOKING)
                                        value=1.0
                                        required=true
                                    />
                                </div>
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="comment">"Commentaire (facultatif)"</Label>
                                <Textarea
                                    id="comment"
                                    name="comment"
                                    rows=3
                                    maxlength=1000
                                    placeholder="Une question, un besoin particulier…"
                                />
                            </div>

                            {move || {
                                error()
                                    .map(|message| {
                                        view! {
                                            <Alert variant=AlertVariant::Destructive>{message}</Alert>
                                        }
                                    })
                            }}

                            <Button class="w-full md:w-auto">
                                {move || {
                                    if pending.get() { "Envoi…" } else { "Réserver" }
                                }}
                            </Button>

                        </div>
                    </ActionForm>
                </CardContent>
            </Card>
        </div>
    })
}

/// Replaces the form once the booking is recorded.
#[component]
fn BookingConfirmed(service: ServiceType, session_label: String) -> impl IntoView {
    let heading_label = session_label.clone();

    view! {
        <Card>
            <CardHeader>
                <div class="flex gap-3 items-start">
                    <span class="flex justify-center items-center mt-0.5 w-8 h-8 rounded-full shrink-0 bg-primary/15 text-primary">
                        <Check class="w-5 h-5"/>
                    </span>
                    <div class="grid gap-1.5">
                        <CardTitle>"Votre réservation est enregistrée"</CardTitle>
                        <CardDescription>
                            "Nous vous attendons le "{heading_label}"."
                        </CardDescription>
                    </div>
                </div>
            </CardHeader>

            <CardContent>
                <div class="flex flex-col gap-6">
                    <dl class="grid gap-3 p-4 text-sm rounded-lg border sm:grid-cols-2">
                        <div class="grid gap-1">
                            <dt class="text-muted-foreground">"Atelier"</dt>
                            <dd class="font-medium">{service.label()}</dd>
                        </div>
                        <div class="grid gap-1">
                            <dt class="text-muted-foreground">"Séance"</dt>
                            <dd class="font-medium">{session_label}</dd>
                        </div>
                    </dl>

                    <p class="text-sm text-muted-foreground">
                        "Vous recevrez une confirmation par e-mail. En cas d'empêchement, "
                        "prévenez-nous afin que la place profite à quelqu'un d'autre."
                    </p>

                    <div class="flex flex-col gap-3 sm:flex-row">
                        <Button class="w-full sm:w-auto" href=service.page_path()>
                            "Revenir à l'atelier"
                        </Button>
                        <a
                            href="/"
                            class="inline-flex justify-center items-center px-4 h-9 text-sm rounded-md border transition-colors hover:bg-accent"
                        >
                            "Retour à l'accueil"
                        </a>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn session() -> SessionView {
        SessionView {
            id: "651d1f0a0000000000000001".to_owned(),
            service_type: ServiceType::AperosCreatifs,
            date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000002".to_owned(),
            theme_name: "Aquarelle".to_owned(),
            price: 65.0,
            max_persons: 8,
            booked_persons: 2,
        }
    }

    fn form_html() -> String {
        Owner::new().with(|| {
            view! {
                <BookingForm service=ServiceType::AperosCreatifs sessions=vec![session()]/>
            }
            .to_html()
        })
    }

    /// The rendered `<input>` carrying this id, so an assertion about one field
    /// cannot pass on an attribute belonging to another.
    fn input_with_id(html: &str, id: &str) -> String {
        let needle = format!(r#"id="{id}""#);

        html.split('<')
            .find(|element| element.starts_with("input") && element.contains(&needle))
            .unwrap_or_else(|| panic!("no input carries id {id:?}: {html}"))
            .to_owned()
    }

    /// The phone number replaced the address as the field we insist on, so the two
    /// have to disagree about being required.
    #[test]
    fn the_phone_is_required_and_the_email_is_not() {
        let html = form_html();

        let phone = input_with_id(&html, "phone");
        assert!(phone.contains("required"), "the phone should be required: {phone}");

        let email = input_with_id(&html, "email");
        assert!(!email.contains("required"), "the email should be optional: {email}");
    }

    /// A field that no longer refuses to submit has to say so, or it still reads
    /// as mandatory.
    #[test]
    fn the_email_label_says_it_is_optional() {
        let html = form_html();

        assert!(html.contains("Adresse e-mail (facultatif)"), "{html}");
    }
}
