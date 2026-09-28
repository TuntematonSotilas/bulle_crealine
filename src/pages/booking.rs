use icons::Check;
use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::api::bookings::CreateBooking;
use crate::api::sessions::upcoming_offer;
use crate::auth::user_message;
use crate::components::blocks::studio_place::StudioPlace;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;
use crate::components::ui::number_field::NumberField;
use crate::components::ui::textarea::Textarea;
use crate::models::{MAX_PERSONS_PER_BOOKING, ServiceView, SessionView};

/// Booking page for one kind of workshop, at `/booking/<slug>`.
///
/// The workshop is fetched rather than derived from the slug: workshops are rows
/// the admin edits, so only the server can say whether one exists and whether it
/// takes online bookings at all.
#[component]
pub fn BookingPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("service").unwrap_or_default();

    // Re-runs when the slug changes, so moving between workshops does not leave
    // the previous list on screen.
    let offer = Resource::new(slug, |slug| async move { upcoming_offer(slug).await });

    // `?session=<id>` is what a "Réserver" carries: the date was already chosen
    // on the page it was clicked from, so the picker opens on it.
    let chosen = use_query_map().get_untracked().get("session");

    view! {
        <Title text="Réserver un atelier — Bulle Créaline (E.I)"/>

        <Transition fallback=|| {
            view! {
                <p class="text-sm text-muted-foreground">"Chargement des séances…"</p>
            }
        }>
            {move || {
                // Cloned per run rather than moved: the closure re-runs, and an
                // async block that swallowed the `String` could only run once.
                let chosen = chosen.clone();

                Suspend::new(async move {
                match offer.await {
                    Err(error) => {
                        EitherOf3::A(
                            view! {
                                <div class="mx-auto max-w-2xl">
                                    <Alert variant=AlertVariant::Destructive>
                                        {user_message(&error)}
                                    </Alert>
                                </div>
                            },
                        )
                    }
                    // Either no such workshop, or one run for a structure, which
                    // agrees on its dates directly and has nothing to offer here.
                    Ok(None) => EitherOf3::B(view! { <UnknownService/> }),
                    Ok(Some(offer)) => {
                        let label = offer.service.label.clone();

                        EitherOf3::C(
                            view! {
                                <div class="flex flex-col gap-6 mx-auto max-w-2xl">
                                    <div>
                                        <h1 class="text-2xl font-semibold">"Réserver"</h1>
                                        <p class="text-muted-foreground">{label}</p>
                                    </div>

                                    <BookingForm
                                        service=offer.service
                                        sessions=offer.sessions
                                        chosen=chosen
                                    />
                                </div>
                            },
                        )
                    }
                }
                })
            }}
        </Transition>
    }
}

/// Shown when the URL carries a workshop that cannot be booked.
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
///
/// `chosen` is the id carried by `?session=`, which is how "Réserver" hands over
/// the date that was clicked -- on the home page and on a workshop's own page
/// alike. A date already picked is shown on its own rather than as one option
/// among a list the visitor would have to read through again.
#[component]
fn BookingForm(
    service: ServiceView,
    sessions: Vec<SessionView>,
    chosen: Option<String>,
) -> impl IntoView {
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

    // Only an id that names one of the dates on offer. A link outlives the date
    // it carries -- deleted, or simply past -- and a stale one falls back to the
    // list rather than to a form pointing at nothing.
    let picked = RwSignal::new(
        chosen.filter(|id| sessions.iter().any(|session| &session.id == id)),
    );

    // Held rather than moved: the two arrangements are built on demand, and the
    // visitor can go from one back to the other.
    let sessions = StoredValue::new(sessions);

    let choices = move || match picked.get() {
        Some(id) => {
            let session = sessions
                .get_value()
                .into_iter()
                .find(|session| session.id == id)
                .expect("filtered against the list above");

            Either::Left(view! { <ChosenSession session=session picked=picked/> })
        }
        None => Either::Right(view! { <SessionChoices sessions=sessions.get_value()/> }),
    };

    Either::Right(view! {
        {move || {
            // Cloned per run rather than moved: the closure re-runs on every change
            // to the action's value, and a `ServiceView` carries strings.
            let service = service.clone();

            confirmed()
                .map(|session_label| {
                    view! { <BookingConfirmed service=service session_label=session_label/> }
                })
        }}

        // Hidden rather than unmounted, so that what the visitor typed survives a
        // refusal from the server and comes back filled in.
        <div class=move || if confirmed().is_some() { "hidden" } else { "" }>
            <Card>
                <CardHeader>
                    <CardTitle>
                        {move || {
                            if picked.get().is_some() {
                                "Votre séance"
                            } else {
                                "Choisissez votre séance"
                            }
                        }}
                    </CardTitle>
                    <CardDescription>
                        {move || {
                            // Counting what is open is an answer to "which one?",
                            // a question already settled when a date was clicked.
                            if picked.get().is_some() {
                                return "Vérifiez la date, puis laissez-nous vos coordonnées."
                                    .to_owned();
                            }

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

                            // Above the dates rather than below: the address holds
                            // for every one of them, so it is read before picking.
                            <StudioPlace/>

                            {choices}

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
                                        placeholder="06 12 34 56 78"
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

/// The one date the visitor already picked, and the way back to the others.
///
/// Its id travels in a hidden field: the form posts `session_id` either way, so
/// the server sees the same thing whichever arrangement was shown.
#[component]
fn ChosenSession(session: SessionView, picked: RwSignal<Option<String>>) -> impl IntoView {
    // All read before the view takes the fields apart.
    let full = session.is_full();
    let availability = session.availability_label();
    let price = session.price_label();

    let SessionView { id, date_label, theme_name, .. } = session;

    view! {
        <div class="flex flex-col gap-3">
            <input type="hidden" name="session_id" value=id/>

            <div class="flex flex-wrap gap-3 justify-between items-start p-4 rounded-lg border border-primary bg-primary/5">
                <span class="flex flex-col gap-1">
                    <span class="font-medium">{date_label}</span>
                    <span class="text-sm text-muted-foreground">
                        "Thème : "{theme_name}" · "{price}
                    </span>
                    <span class=if full {
                        "text-sm font-medium text-destructive"
                    } else {
                        "text-sm text-muted-foreground"
                    }>{availability}</span>
                </span>

                // `type="button"`, or it would submit the form it sits in.
                <button
                    type="button"
                    class="text-sm font-medium underline-offset-4 text-primary hover:underline"
                    on:click=move |_| picked.set(None)
                >
                    "Choisir une autre date"
                </button>
            </div>
        </div>
    }
}

/// Every date on offer, to pick from.
#[component]
fn SessionChoices(sessions: Vec<SessionView>) -> impl IntoView {
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
                        <span class=if full {
                            "text-sm font-medium text-destructive"
                        } else {
                            "text-sm text-muted-foreground"
                        }>{session.availability_label()}</span>
                    </span>
                </label>
            }
        })
        .collect::<Vec<_>>();

    view! { <div class="flex flex-col gap-3">{choices}</div> }
}

/// Replaces the form once the booking is recorded.
#[component]
fn BookingConfirmed(service: ServiceView, session_label: String) -> impl IntoView {
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
                            <dd class="font-medium">{service.label.clone()}</dd>
                        </div>
                        <div class="grid gap-1">
                            <dt class="text-muted-foreground">"Séance"</dt>
                            <dd class="font-medium">{session_label}</dd>
                        </div>
                    </dl>

                    // Repeated here because this screen replaces the form: without
                    // it the address would vanish at the very moment it is needed.
                    <StudioPlace/>

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

    fn service() -> ServiceView {
        ServiceView {
            id: "651d1f0a0000000000000003".to_owned(),
            slug: "aperos-creatifs".to_owned(),
            label: "Apéros créatifs (adultes)".to_owned(),
            description: "Une soirée entre adultes.".to_owned(),
            age: "À partir de 18 ans".to_owned(),
            steps: Vec::new(),
            icon: "Wine".to_owned(),
            pro: false,
            position: 0,
        }
    }

    fn session() -> SessionView {
        SessionView {
            id: "651d1f0a0000000000000001".to_owned(),
            service_slug: "aperos-creatifs".to_owned(),
            service_label: "Apéros créatifs (adultes)".to_owned(),
            service_description: "Une soirée entre adultes.".to_owned(),
            service_path: "/services/aperos-creatifs".to_owned(),
            date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
            date_input: "2026-07-05T14:00".to_owned(),
            theme_id: "651d1f0a0000000000000002".to_owned(),
            theme_name: "Aquarelle".to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000002?v=1".to_owned(),
            price: 65.0,
            max_persons: 8,
            booked_persons: 2,
        }
    }

    fn form_html() -> String {
        Owner::new().with(|| {
            view! { <BookingForm service=service() sessions=vec![session()] chosen=None/> }
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

    fn form_with(chosen: Option<&str>) -> String {
        let chosen = chosen.map(str::to_owned);
        let dates = vec![session(), SessionView {
            id: "651d1f0a0000000000000009".to_owned(),
            date_label: "dimanche 12 juillet 2026 à 14h00".to_owned(),
            ..session()
        }];

        Owner::new().with(|| {
            view! { <BookingForm service=service() sessions=dates chosen=chosen/> }.to_html()
        })
    }

    /// A date clicked on the home page, or on the workshop's own page, is not a
    /// question to be asked again: the picker opens on it and leaves the others
    /// out.
    #[test]
    fn a_chosen_date_is_shown_alone_and_posted_as_it_stands() {
        let html = form_with(Some("651d1f0a0000000000000001"));

        assert!(html.contains("dimanche 5 juillet 2026 à 14h00"), "no date: {html}");
        assert!(
            !html.contains("dimanche 12 juillet 2026 à 14h00"),
            "the other dates should be out of the way: {html}"
        );
        // The server reads one field either way, so the id still has to be posted.
        assert!(
            html.contains(r#"type="hidden" name="session_id" value="651d1f0a0000000000000001""#),
            "the chosen date is not posted: {html}"
        );
        assert!(!html.contains(r#"type="radio""#), "the list is still there: {html}");
        assert!(html.contains("Choisir une autre date"), "no way back: {html}");
    }

    /// A link outlives the date it carries. An id nobody recognises has to leave
    /// the visitor picking from the list rather than facing an empty form.
    #[test]
    fn an_unknown_date_leaves_the_whole_list_to_pick_from() {
        let html = form_with(Some("651d1f0a0000000000000404"));

        assert!(html.contains(r#"type="radio""#), "no list: {html}");
        assert!(html.contains("dimanche 5 juillet 2026 à 14h00"), "no date: {html}");
        assert!(html.contains("dimanche 12 juillet 2026 à 14h00"), "no date: {html}");
        assert!(!html.contains("Choisir une autre date"), "{html}");
    }

    #[test]
    fn without_a_chosen_date_every_date_is_offered() {
        let html = form_with(None);

        assert!(html.contains("Choisissez votre séance"), "{html}");
        assert!(html.contains(r#"type="radio""#), "no list: {html}");
        assert!(!html.contains(r#"name="session_id" type="hidden""#), "{html}");
    }

    /// Where to turn up is part of choosing a date, not an afterthought.
    #[test]
    fn the_form_says_where_the_session_is_held() {
        use crate::models::{STUDIO_ADDRESS, STUDIO_MAP_URL};

        let html = form_html();

        assert!(html.contains(STUDIO_ADDRESS), "no address: {html}");
        assert!(html.contains(STUDIO_MAP_URL), "no map link: {html}");
    }

    /// This screen replaces the form outright, so an address shown only on the
    /// form disappears exactly when the visitor starts needing it.
    #[test]
    fn the_confirmation_repeats_where_to_go() {
        use crate::models::STUDIO_ADDRESS;
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        let html = Owner::new().with(|| {
            // It carries a link back to the workshop, and a link resolves
            // `aria-current` against the location being rendered.
            provide_context(RequestUrl::new("/booking/aperos-creatifs"));

            view! {
                <Router>
                    <BookingConfirmed
                        service=service()
                        session_label="dimanche 5 juillet 2026 à 14h00".to_owned()
                    />
                </Router>
            }
            .to_html()
        });

        assert!(html.contains("Votre réservation est enregistrée"), "{html}");
        assert!(html.contains(STUDIO_ADDRESS), "no address: {html}");
    }

    /// A field that no longer refuses to submit has to say so, or it still reads
    /// as mandatory.
    #[test]
    fn the_email_label_says_it_is_optional() {
        let html = form_html();

        assert!(html.contains("Adresse e-mail (facultatif)"), "{html}");
    }

    /// With no date to pick, the page's one job is to point back at the workshop,
    /// whose path is now built from the row rather than from an enum.
    #[test]
    fn a_workshop_with_no_date_points_back_at_its_page() {
        let html = Owner::new()
            .with(|| {
                view! { <BookingForm service=service() sessions=vec![] chosen=None/> }.to_html()
            });

        assert!(html.contains("Aucune séance programmée"), "{html}");
        assert!(html.contains(r#"href="/services/aperos-creatifs""#), "{html}");
    }
}
