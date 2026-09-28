//! Administration of the kinds of workshop, at `/admin/services`.
//!
//! Same shape as [`crate::pages::admin::themes`], minus the file upload: this form
//! is plain fields, so it posts through an `<ActionForm>` and works without the
//! WASM bundle.
//!
//! The one thing here that is not an ordinary CRUD screen is the slug. It is a
//! workshop's identity -- every session and every booking is filed under it, and
//! it is what `/services/<slug>` and `/booking/<slug>` resolve -- so it is chosen
//! once, on creation, and shown as plain text from then on.

use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::services::{
    DeleteService, SaveService, all_services, service_session_count,
};
use crate::auth::user_message;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::number_field::NumberField;
use crate::components::ui::select::{
    Select, SelectContent, SelectGroup, SelectOption, SelectTrigger, SelectValue,
};
use crate::components::ui::service_icon::ServiceIcon;
use crate::components::ui::table::*;
use crate::components::ui::textarea::Textarea;
use crate::models::{SERVICE_ICONS, ServiceView, icon_label, section_title, slugify};
use crate::pages::admin::AdminShell;

/// What the admin is doing to the workshop list right now.
#[derive(Clone, Debug, PartialEq)]
enum Editing {
    /// Just looking.
    None,
    /// Filling in a brand new workshop.
    New,
    /// Changing an existing one.
    Service(ServiceView),
    /// About to drop one.
    Deleting(ServiceView),
}

/// Workshop management.
#[component]
pub fn AdminServicesPage() -> impl IntoView {
    let save = ServerAction::<SaveService>::new();
    let delete = ServerAction::<DeleteService>::new();
    let editing = RwSignal::new(Editing::None);

    // Reloads whenever either action reports back, so the table follows the writes.
    let services = Resource::new(
        move || (save.version().get(), delete.version().get()),
        |_| async move { all_services().await },
    );

    // A successful write closes the form; a failed one keeps it open with its
    // message, so nothing typed is lost.
    Effect::new(move |_| {
        if matches!(save.value().get(), Some(Ok(()))) {
            editing.set(Editing::None);
        }
    });
    Effect::new(move |_| {
        if matches!(delete.value().get(), Some(Ok(()))) {
            editing.set(Editing::None);
        }
    });

    // Hoisted so both form branches below share one closure type.
    let cancel = move || editing.set(Editing::None);

    let save_error = move || {
        save.value()
            .get()
            .and_then(|outcome| outcome.err())
            .map(|error| user_message(&error))
    };
    let delete_error = move || {
        delete
            .value()
            .get()
            .and_then(|outcome| outcome.err())
            .map(|error| user_message(&error))
    };

    view! {
        <Title text="Services — Administration"/>

        <AdminShell title="Services" current="/admin/services">

            {move || {
                delete_error()
                    .map(|message| {
                        view! { <Alert variant=AlertVariant::Destructive>{message}</Alert> }
                    })
            }}

            {move || match editing.get() {
                Editing::None => {
                    EitherOf3::A(
                        view! {
                            <div>
                                <Button on:click=move |_| editing.set(Editing::New)>
                                    "Nouveau service"
                                </Button>
                            </div>
                        },
                    )
                }
                Editing::New | Editing::Service(_) => {
                    let service = match editing.get() {
                        Editing::Service(service) => Some(service),
                        _ => None,
                    };

                    EitherOf3::B(
                        view! {
                            <ServiceForm
                                action=save
                                service=service
                                error=Signal::derive(save_error)
                                on_cancel=cancel
                            />
                        },
                    )
                }
                Editing::Deleting(service) => {
                    EitherOf3::C(
                        view! {
                            <DeleteConfirmation action=delete service=service on_cancel=cancel/>
                        },
                    )
                }
            }}

            <Transition fallback=|| {
                view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
            }>
                {move || Suspend::new(async move {
                    match services.await {
                        Err(error) => {
                            Either::Left(
                                view! {
                                    <Alert variant=AlertVariant::Destructive>
                                        {user_message(&error)}
                                    </Alert>
                                },
                            )
                        }
                        Ok(rows) => {
                            Either::Right(view! { <ServiceTable rows=rows editing=editing/> })
                        }
                    }
                })}
            </Transition>

        </AdminShell>
    }
}

/// The listing itself.
#[component]
fn ServiceTable(rows: Vec<ServiceView>, editing: RwSignal<Editing>) -> impl IntoView {
    if rows.is_empty() {
        return Either::Left(view! {
            <Alert>
                <AlertTitle>"Aucun service"</AlertTitle>
                <AlertDescription>
                    "Créez un service pour qu'il apparaisse dans le menu, dans le catalogue et dans le formulaire des séances."
                </AlertDescription>
            </Alert>
        });
    }

    let body = rows
        .into_iter()
        .map(|service| {
            let for_edit = service.clone();
            let for_delete = service.clone();

            // Read out of `service` before the closures below capture it.
            let icon = service.icon.clone();
            let label = service.label.clone();
            let slug = service.slug.clone();
            let section = section_title(service.pro);

            view! {
                <TableRow>
                    <TableCell>
                        <ServiceIcon name=icon class="w-5 h-5 text-heading-soft"/>
                    </TableCell>
                    <TableCell class="font-medium">{label}</TableCell>
                    <TableCell class="text-muted-foreground">{slug}</TableCell>
                    <TableCell class="whitespace-nowrap">{section}</TableCell>
                    <TableCell>
                        <div class="flex gap-2 justify-end">
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                on:click=move |_| editing.set(Editing::Service(for_edit.clone()))
                            >
                                "Modifier"
                            </Button>
                            <Button
                                variant=ButtonVariant::Destructive
                                size=ButtonSize::Sm
                                on:click=move |_| {
                                    editing.set(Editing::Deleting(for_delete.clone()))
                                }
                            >
                                "Supprimer"
                            </Button>
                        </div>
                    </TableCell>
                </TableRow>
            }
        })
        .collect::<Vec<_>>();

    Either::Right(view! {
        <TableContainer>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>"Picto"</TableHead>
                        <TableHead>"Nom"</TableHead>
                        <TableHead>"Identifiant"</TableHead>
                        <TableHead>"Rubrique"</TableHead>
                        <TableHead class="text-right">"Actions"</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>{body}</TableBody>
            </Table>
        </TableContainer>
    })
}

/// Create or edit form. `service` being `None` means a creation.
#[component]
fn ServiceForm(
    action: ServerAction<SaveService>,
    service: Option<ServiceView>,
    #[prop(into)] error: Signal<Option<String>>,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    let existing = service.clone();
    let id = service.as_ref().map(|s| s.id.clone()).unwrap_or_default();
    let editing_existing = !id.is_empty();

    let slug = service.as_ref().map(|s| s.slug.clone()).unwrap_or_default();
    let label = service.as_ref().map(|s| s.label.clone()).unwrap_or_default();
    let description = service
        .as_ref()
        .map(|s| s.description.clone())
        .unwrap_or_default();
    let age = service.as_ref().map(|s| s.age.clone()).unwrap_or_default();
    // One step per line: a textarea stands in for a list without needing a widget
    // that adds and removes rows, and it is what an admin would type anyway.
    let steps = service
        .as_ref()
        .map(|s| s.steps.join("\n"))
        .unwrap_or_default();
    let position = service.as_ref().map(|s| f64::from(s.position)).unwrap_or(0.0);

    let icon = service.as_ref().map(|s| s.icon.clone()).unwrap_or_default();
    let icon_wording = icon_label(&icon).unwrap_or("Aucun").to_owned();

    let pro = existing.as_ref().is_some_and(|s| s.pro);
    // Bound rather than written inline: `if … {} else {}.to_string()` reads as a
    // method on the `else` block at a glance, which is not what it means.
    let pro_value = if pro { "oui" } else { "non" };

    let label_text = RwSignal::new(label.clone());
    let slug_text = RwSignal::new(slug.clone());
    // What the prefill last wrote, to tell its own doing from the admin's typing.
    let generated = RwSignal::new(String::new());

    // Creation only: on an edit the slug is read-only text, there being sessions
    // and bookings filed under it.
    if !editing_existing {
        Effect::new(move |_| {
            let candidate = slugify(&label_text.get());

            // Silent for good once the admin has typed a slug of their own: the
            // field is theirs from then on, and rewording the name must not undo
            // it. Both start empty, so the first comparison holds and it fills.
            if slug_text.get_untracked() == generated.get_untracked() {
                slug_text.set(candidate.clone());
                generated.set(candidate);
            }
        });
    }

    view! {
        <Card>
            <CardHeader>
                <CardTitle>
                    {if editing_existing { "Modifier le service" } else { "Nouveau service" }}
                </CardTitle>
                <CardDescription>
                    "Un service apparaît dans le menu, dans le catalogue et sur sa propre page."
                </CardDescription>
            </CardHeader>

            <CardContent>
                {editing_existing.then(|| view! { <AffectedSessions service_id=id.clone()/> })}

                <ActionForm action=action>
                    <input type="hidden" name="id" value=id/>

                    <div class="flex flex-col gap-6">
                        <div class="grid gap-4 md:grid-cols-2">

                            <div class="grid gap-3">
                                <Label r#for="label">"Nom"</Label>
                                // Bound only on a creation, where the slug follows
                                // what is typed here. `bind:value` renders no
                                // `value` attribute server-side, so an edit keeps
                                // the plain attribute or it would open blank.
                                {if editing_existing {
                                    Either::Left(
                                        view! {
                                            <Input
                                                id="label"
                                                name="label"
                                                required=true
                                                attr:value=label
                                            />
                                        },
                                    )
                                } else {
                                    Either::Right(
                                        view! {
                                            <Input
                                                id="label"
                                                name="label"
                                                required=true
                                                bind_value=label_text
                                            />
                                        },
                                    )
                                }}
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="slug">"Identifiant (URL)"</Label>
                                {if editing_existing {
                                    // Shown rather than editable: sessions and
                                    // bookings are filed under it, and changing it
                                    // would leave them pointing at nothing.
                                    Either::Left(
                                        view! {
                                            <input type="hidden" name="slug" value=slug.clone()/>
                                            <p class="px-3 py-2 font-mono text-sm rounded-md border bg-muted text-muted-foreground">
                                                {slug}
                                            </p>
                                            <p class="text-sm text-muted-foreground">
                                                "L'identifiant ne peut plus changer : les séances et les réservations y sont rattachées."
                                            </p>
                                        },
                                    )
                                } else {
                                    Either::Right(
                                        view! {
                                            <Input
                                                id="slug"
                                                name="slug"
                                                required=true
                                                bind_value=slug_text
                                                attr:placeholder="aperos-creatifs"
                                            />
                                            <p class="text-sm text-muted-foreground">
                                                "Minuscules, chiffres et tirets. Il apparaît dans l'adresse de la page et ne pourra plus être modifié."
                                            </p>
                                        },
                                    )
                                }}
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="pro">"Rubrique"</Label>
                                <Select
                                    class="w-full max-w-sm"
                                    name="pro".to_string()
                                    default_value=pro_value.to_string()
                                    default_label=section_title(pro).to_string()
                                >
                                    <SelectTrigger id="pro">
                                        <SelectValue placeholder="Rubrique"/>
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectGroup>
                                            <SelectOption
                                                value="non"
                                                label=section_title(false).to_string()
                                            >
                                                {section_title(false)}
                                            </SelectOption>
                                            <SelectOption
                                                value="oui"
                                                label=section_title(true).to_string()
                                            >
                                                {section_title(true)}
                                            </SelectOption>
                                        </SelectGroup>
                                    </SelectContent>
                                </Select>
                                <p class="text-sm text-muted-foreground">
                                    "« Autres Ateliers » désigne les services menés pour une structure : ils ne sont pas réservables en ligne."
                                </p>
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="icon">"Picto"</Label>
                                <Select
                                    class="w-full max-w-sm"
                                    name="icon".to_string()
                                    default_value=icon.clone()
                                    default_label=icon_wording
                                >
                                    <SelectTrigger id="icon">
                                        <SelectValue placeholder="Picto"/>
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectGroup>
                                            // Built inside the view rather than
                                            // hoisted: a `SelectOption` reads the
                                            // context its `Select` provides.
                                            <SelectOption value="" label="Aucun".to_string()>
                                                "Aucun"
                                            </SelectOption>
                                            {SERVICE_ICONS
                                                .into_iter()
                                                .map(|(name, wording)| {
                                                    view! {
                                                        <SelectOption
                                                            value=name
                                                            label=wording.to_string()
                                                        >
                                                            <span class="inline-flex gap-2 items-center">
                                                                <ServiceIcon
                                                                    name=name
                                                                    class="w-4 h-4 text-heading-soft"
                                                                />
                                                                {wording}
                                                            </span>
                                                        </SelectOption>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </SelectGroup>
                                    </SelectContent>
                                </Select>
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="age">"Âge requis (facultatif)"</Label>
                                <Input
                                    id="age"
                                    name="age"
                                    attr:value=age
                                    attr:placeholder="De 0 à 6 ans"
                                />
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="position">"Ordre d'affichage"</Label>
                                <NumberField
                                    id="position"
                                    name="position"
                                    min=0.0
                                    required=true
                                    value=position
                                />
                                <p class="text-sm text-muted-foreground">
                                    "Du plus petit au plus grand, à l'intérieur de la rubrique."
                                </p>
                            </div>

                        </div>

                        <div class="grid gap-3">
                            <Label r#for="description">"Description"</Label>
                            <Textarea
                                id="description"
                                name="description"
                                rows=3
                                required=true
                                value=description
                            />
                            <p class="text-sm text-muted-foreground">
                                "Reprise sur la page du service, dans le catalogue et sur les cartes de l'accueil."
                            </p>
                        </div>

                        <div class="grid gap-3">
                            <Label r#for="steps">"Déroulement (facultatif)"</Label>
                            <Textarea id="steps" name="steps" rows=8 value=steps/>
                            <p class="text-sm text-muted-foreground">
                                "Une étape par ligne. Elles sont numérotées automatiquement sur la page du service."
                            </p>
                        </div>

                        {move || {
                            error
                                .get()
                                .map(|message| {
                                    view! {
                                        <Alert variant=AlertVariant::Destructive>{message}</Alert>
                                    }
                                })
                        }}

                        <div class="flex gap-2">
                            <Button>"Enregistrer"</Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                attr:r#type="button"
                                on:click=move |_| on_cancel()
                            >
                                "Annuler"
                            </Button>
                        </div>
                    </div>
                </ActionForm>
            </CardContent>
        </Card>
    }
}

/// Confirmation asked before a workshop is dropped.
///
/// A workshop still carrying sessions cannot go: those sessions store its slug and
/// would lose their name everywhere at once. The server refuses either way -- this
/// only avoids offering a button that cannot work.
#[component]
fn DeleteConfirmation(
    action: ServerAction<DeleteService>,
    service: ServiceView,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    let id = service.id.clone();
    let for_resource = service.id.clone();

    let sessions = Resource::new(
        move || for_resource.clone(),
        |id| async move { service_session_count(id).await },
    );

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Supprimer ce service ?"</CardTitle>
                <CardDescription>{service.label.clone()}</CardDescription>
            </CardHeader>

            <CardContent>
                <Transition fallback=|| ()>
                    {move || {
                        let id = id.clone();
                        Suspend::new(async move {
                            match sessions.await {
                                // In use: say how many, and offer no button that the
                                // server would only turn down.
                                Ok(count) if count > 0 => {
                                    EitherOf3::A(
                                        view! {
                                            <UsedBySessions count=count/>
                                            <Button
                                                variant=ButtonVariant::Ghost
                                                attr:r#type="button"
                                                on:click=move |_| on_cancel()
                                            >
                                                "Fermer"
                                            </Button>
                                        },
                                    )
                                }
                                Ok(_) => {
                                    EitherOf3::B(
                                        view! {
                                            <ConfirmDeletion
                                                action=action
                                                id=id
                                                on_cancel=on_cancel
                                            />
                                        },
                                    )
                                }
                                // The count could not be read; let the attempt
                                // through, since the server checks again anyway.
                                Err(_) => {
                                    EitherOf3::C(
                                        view! {
                                            <ConfirmDeletion
                                                action=action
                                                id=id
                                                on_cancel=on_cancel
                                            />
                                        },
                                    )
                                }
                            }
                        })
                    }}
                </Transition>
            </CardContent>
        </Card>
    }
}

/// The buttons that actually post the deletion.
#[component]
fn ConfirmDeletion(
    action: ServerAction<DeleteService>,
    id: String,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <input type="hidden" name="id" value=id/>
            <div class="flex gap-2">
                <Button variant=ButtonVariant::Destructive>"Oui, supprimer le service"</Button>
                <Button
                    variant=ButtonVariant::Ghost
                    attr:r#type="button"
                    on:click=move |_| on_cancel()
                >
                    "Annuler"
                </Button>
            </div>
        </ActionForm>
    }
}

/// The warning shown when a workshop already carries sessions.
///
/// Renders nothing when none does, so a routine edit is not dressed up as a
/// dangerous operation.
#[component]
fn AffectedSessions(service_id: String) -> impl IntoView {
    let sessions = Resource::new(
        move || service_id.clone(),
        |id| async move { service_session_count(id).await },
    );

    view! {
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                match sessions.await {
                    Err(_) => EitherOf3::A(()),
                    Ok(0) => EitherOf3::B(()),
                    Ok(count) => EitherOf3::C(view! { <UsedBySessions count=count/> }),
                }
            })}
        </Transition>
    }
}

/// Spells out how much a change to this workshop would show up on.
#[component]
fn UsedBySessions(count: u64) -> impl IntoView {
    view! {
        <Alert variant=AlertVariant::Destructive class="mb-6">
            <AlertTitle class="text-base">"⚠ Ce service est utilisé"</AlertTitle>
            <AlertDescription>
                <p>
                    {format!(
                        "{count} séance(s) portent ce service : toute modification s'affichera aussi sur elles, et la suppression est impossible tant qu'elles existent.",
                    )}
                </p>
            </AlertDescription>
        </Alert>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn service() -> ServiceView {
        ServiceView {
            id: "651d1f0a0000000000000001".to_owned(),
            slug: "aperos-creatifs".to_owned(),
            label: "Apéros créatifs (adultes)".to_owned(),
            description: "Une soirée entre adultes.".to_owned(),
            age: "À partir de 18 ans".to_owned(),
            steps: vec!["Accueil".to_owned(), "Réalisation".to_owned()],
            icon: "Wine".to_owned(),
            pro: false,
            position: 2,
        }
    }

    fn form_html(service: Option<ServiceView>) -> String {
        // The impact warning owns a `Resource`, and so does nothing else here.
        crate::pages::admin::init_test_executor();

        Owner::new().with(|| {
            // Bound out here: `view!` would read the turbofish as a tag.
            let action = ServerAction::<SaveService>::new();
            let error: Signal<Option<String>> = Signal::derive(|| None);

            view! {
                <ServiceForm action=action service=service error=error on_cancel=|| {}/>
            }
            .to_html()
        })
    }

    /// The slug is chosen once. On a creation it is a field the admin fills in.
    #[test]
    fn a_creation_asks_for_the_slug() {
        let html = form_html(None);

        assert!(html.contains(r#"id="slug""#), "the slug field should render: {html}");
        assert!(html.contains("Nouveau service"), "the title should say so: {html}");
        assert!(
            !html.contains("ne peut plus changer"),
            "nothing is filed under it yet: {html}"
        );
    }

    /// Changing it would leave every session and booking filed under the old one
    /// pointing at nothing, so an edit shows it and posts it back untouched.
    #[test]
    fn an_edit_shows_the_slug_without_letting_it_change() {
        let html = form_html(Some(service()));

        assert!(
            html.contains(r#"<input type="hidden" name="slug" value="aperos-creatifs""#),
            "the slug should be posted back as it stands: {html}"
        );
        assert!(
            !html.contains(r#"id="slug""#),
            "and not offered as an editable field: {html}"
        );
        assert!(html.contains("ne peut plus changer"), "the reason should be given: {html}");
    }

    /// The form is the only way these reach the database, so each one has to be
    /// on it -- and filled in from the row when there is one.
    #[test]
    fn an_edit_fills_in_what_the_workshop_carries() {
        let html = form_html(Some(service()));

        assert!(html.contains("Apéros créatifs (adultes)"), "no name: {html}");
        assert!(html.contains("Une soirée entre adultes."), "no description: {html}");
        assert!(html.contains("À partir de 18 ans"), "no age: {html}");
        // One per line, which is how the server reads them back.
        assert!(html.contains("Accueil\nRéalisation"), "no run-through: {html}");
        assert!(html.contains(r#"value="2""#), "no rank: {html}");
    }

    /// The picker is the only place the icon names are offered, and a name it does
    /// not offer is refused server-side -- so anything missing here is unreachable.
    ///
    /// Asserted on the wording rather than on the value: a `SelectOption` keeps its
    /// value in a closure and renders none of it as an attribute.
    #[test]
    fn the_picker_offers_every_icon_and_the_option_of_none() {
        let html = form_html(None);

        for (name, wording) in SERVICE_ICONS {
            assert!(html.contains(wording), "{name} is not offered: {html}");
        }

        assert!(html.contains("Aucun"), "a workshop may have no picto: {html}");
        // Each option draws its own icon, so the picker shows what it is offering
        // rather than a list of names.
        assert!(
            html.matches("data-name=\"SelectOption\"").count() > SERVICE_ICONS.len(),
            "the options should render: {html}"
        );
    }

    /// The flag decides both the menu section and whether the workshop can be
    /// booked at all, so the form has to say what it does.
    #[test]
    fn the_section_is_offered_as_the_two_headings_the_menu_uses() {
        let html = form_html(None);

        assert!(html.contains(section_title(false)), "{html}");
        assert!(html.contains(section_title(true)), "{html}");
        assert!(html.contains("pas réservables en ligne"), "the consequence: {html}");
    }

    /// The dropdown posts through a hidden input, so what it starts on is what an
    /// edit that leaves the field alone will save.
    #[test]
    fn an_edit_starts_the_section_on_the_one_the_workshop_is_in() {
        let mut for_structures = service();
        for_structures.pro = true;

        assert!(
            form_html(Some(for_structures)).contains(r#"value="oui""#),
            "a workshop for structures should start there"
        );
        assert!(
            form_html(Some(service())).contains(r#"value="non""#),
            "and a bookable one on the other"
        );
    }

    #[test]
    fn the_table_lists_a_workshop_with_its_slug_and_section() {
        let html = Owner::new().with(|| {
            let editing = RwSignal::new(Editing::None);
            view! { <ServiceTable rows=vec![service()] editing=editing/> }.to_html()
        });

        assert!(html.contains("Apéros créatifs (adultes)"), "no name: {html}");
        assert!(html.contains("aperos-creatifs"), "no slug: {html}");
        assert!(html.contains(section_title(false)), "no section: {html}");
        assert!(html.contains("<svg"), "no picto: {html}");
    }

    /// An empty collection takes the menu and the booking pages down with it, so
    /// it must read as something to fix rather than as a blank table.
    #[test]
    fn the_table_says_so_when_there_is_nothing_yet() {
        let html = Owner::new().with(|| {
            let editing = RwSignal::new(Editing::None);
            view! { <ServiceTable rows=vec![] editing=editing/> }.to_html()
        });

        assert!(html.contains("Aucun service"), "{html}");
    }

    /// The warning is what tells the admin an edit is not harmless.
    #[test]
    fn the_warning_counts_the_sessions_at_stake() {
        let html = Owner::new().with(|| view! { <UsedBySessions count=3/> }.to_html());

        assert!(html.contains("3 séance(s)"), "the count should show: {html}");
        assert!(html.contains("suppression est impossible"), "and the consequence: {html}");
    }
}
