use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::sessions::{
    DeleteSession, SaveSession, all_sessions, session_contacts,
};
use crate::auth::user_message;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;
use crate::components::ui::select::Select;
use crate::components::ui::table::*;
use crate::models::{ServiceType, SessionView};
use crate::pages::admin::AdminShell;

/// What the admin is doing to the session list right now.
#[derive(Clone, Debug, PartialEq)]
enum Editing {
    /// Just looking.
    None,
    /// Filling in a brand new session.
    New,
    /// Changing an existing one.
    Session(SessionView),
    /// About to drop one.
    Deleting(SessionView),
}

/// Session management, at `/admin/sessions`.
#[component]
pub fn AdminSessionsPage() -> impl IntoView {
    let save = ServerAction::<SaveSession>::new();
    let delete = ServerAction::<DeleteSession>::new();
    let editing = RwSignal::new(Editing::None);

    // Reloads whenever either action reports back, so the table follows the writes.
    let sessions = Resource::new(
        move || (save.version().get(), delete.version().get()),
        |_| async move { all_sessions().await },
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

    // Hoisted so both form branches below share one closure type; two inline
    // copies would be two distinct types and could not share a `match` arm.
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
        <Title text="Séances — Administration"/>

        <AdminShell title="Séances" current="/admin/sessions">

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
                                    "Nouvelle séance"
                                </Button>
                            </div>
                        },
                    )
                }
                Editing::New | Editing::Session(_) => {
                    let session = match editing.get() {
                        Editing::Session(session) => Some(session),
                        _ => None,
                    };

                    EitherOf3::B(
                        view! {
                            <SessionForm
                                action=save
                                session=session
                                error=Signal::derive(save_error)
                                on_cancel=cancel
                            />
                        },
                    )
                }
                Editing::Deleting(session) => {
                    EitherOf3::C(
                        view! {
                            <DeleteConfirmation
                                action=delete
                                session=session
                                on_cancel=cancel
                            />
                        },
                    )
                }
            }}

            <Transition fallback=|| {
                view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
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
                        Ok(rows) => Either::Right(view! { <SessionTable rows=rows editing=editing/> }),
                    }
                })}
            </Transition>

        </AdminShell>
    }
}

/// The listing itself.
#[component]
fn SessionTable(rows: Vec<SessionView>, editing: RwSignal<Editing>) -> impl IntoView {
    if rows.is_empty() {
        return Either::Left(view! {
            <Alert>
                <AlertTitle>"Aucune séance"</AlertTitle>
                <AlertDescription>
                    "Créez une séance pour qu'elle apparaisse sur la page de réservation."
                </AlertDescription>
            </Alert>
        });
    }

    let body = rows
        .into_iter()
        .map(|session| {
            let for_edit = session.clone();
            let for_delete = session.clone();

            // Read out of `session` before the closures below capture it.
            let date_label = session.date_label.clone();
            let service_label = session.service_type.label();
            let theme = session.theme.clone();
            let price_label = session.price_label();
            let capacity = format!("{} / {}", session.booked_persons, session.max_persons);

            view! {
                <TableRow>
                    <TableCell class="font-medium">{date_label}</TableCell>
                    <TableCell>{service_label}</TableCell>
                    <TableCell>{theme}</TableCell>
                    <TableCell class="whitespace-nowrap">{price_label}</TableCell>
                    <TableCell class="whitespace-nowrap">{capacity}</TableCell>
                    <TableCell>
                        <div class="flex gap-2 justify-end">
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                on:click=move |_| editing.set(Editing::Session(for_edit.clone()))
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
                        <TableHead>"Date"</TableHead>
                        <TableHead>"Atelier"</TableHead>
                        <TableHead>"Thème"</TableHead>
                        <TableHead>"Prix"</TableHead>
                        <TableHead>"Inscrits"</TableHead>
                        <TableHead class="text-right">"Actions"</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>{body}</TableBody>
            </Table>
        </TableContainer>
    })
}

/// Create or edit form. `session` being `None` means a creation.
#[component]
fn SessionForm(
    action: ServerAction<SaveSession>,
    session: Option<SessionView>,
    #[prop(into)] error: Signal<Option<String>>,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    let existing = session.clone();
    let id = session.as_ref().map(|s| s.id.clone()).unwrap_or_default();
    let editing_existing = !id.is_empty();

    let options = ServiceType::ALL
        .into_iter()
        .map(|kind| {
            let selected = existing
                .as_ref()
                .is_some_and(|session| session.service_type == kind);

            view! {
                <option value=kind.slug() selected=selected>
                    {kind.label()}
                </option>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <Card>
            <CardHeader>
                <CardTitle>
                    {if editing_existing { "Modifier la séance" } else { "Nouvelle séance" }}
                </CardTitle>
                <CardDescription>
                    "L'heure saisie est l'heure locale, telle qu'elle sera affichée aux visiteurs."
                </CardDescription>
            </CardHeader>

            <CardContent>
                {editing_existing
                    .then(|| view! { <AffectedBookings session_id=id.clone()/> })}

                <ActionForm action=action>
                    <input type="hidden" name="id" value=id/>

                    <div class="flex flex-col gap-6">
                        <div class="grid gap-4 md:grid-cols-2">

                            <div class="grid gap-3">
                                <Label r#for="service">"Type d'atelier"</Label>
                                <Select id="service" name="service" required=true>
                                    {options}
                                </Select>
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="date">"Date et heure"</Label>
                                <Input
                                    r#type=InputType::DatetimeLocal
                                    id="date"
                                    name="date"
                                    required=true
                                    attr:value=session
                                        .as_ref()
                                        .map(|s| s.date_input.clone())
                                        .unwrap_or_default()
                                />
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="theme">"Thème"</Label>
                                <Input
                                    id="theme"
                                    name="theme"
                                    required=true
                                    attr:value=session
                                        .as_ref()
                                        .map(|s| s.theme.clone())
                                        .unwrap_or_default()
                                />
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="price">"Prix (€)"</Label>
                                <Input
                                    r#type=InputType::Number
                                    id="price"
                                    name="price"
                                    min="0"
                                    step="0.5"
                                    required=true
                                    attr:value=session
                                        .as_ref()
                                        .map(|s| s.price.to_string())
                                        .unwrap_or_else(|| "0".to_owned())
                                />
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="max_persons">"Nombre de places (personnes)"</Label>
                                <Input
                                    r#type=InputType::Number
                                    id="max_persons"
                                    name="max_persons"
                                    min="1"
                                    step="1"
                                    required=true
                                    attr:value=session
                                        .as_ref()
                                        .map(|s| s.max_persons.to_string())
                                        .unwrap_or_else(|| "8".to_owned())
                                />
                            </div>

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

/// Confirmation asked before a session is dropped.
#[component]
fn DeleteConfirmation(
    action: ServerAction<DeleteSession>,
    session: SessionView,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    let id = session.id.clone();

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Supprimer cette séance ?"</CardTitle>
                <CardDescription>
                    {format!(
                        "{} · {} · {}",
                        session.date_label,
                        session.service_type.label(),
                        session.theme,
                    )}
                </CardDescription>
            </CardHeader>

            <CardContent>
                <AffectedBookings session_id=id.clone()/>

                <ActionForm action=action>
                    <input type="hidden" name="id" value=id/>
                    <div class="flex gap-2">
                        <Button variant=ButtonVariant::Destructive>
                            "Oui, supprimer la séance"
                        </Button>
                        <Button
                            variant=ButtonVariant::Ghost
                            attr:r#type="button"
                            on:click=move |_| on_cancel()
                        >
                            "Annuler"
                        </Button>
                    </div>
                </ActionForm>
            </CardContent>
        </Card>
    }
}

/// The loud warning shown when a session already carries bookings.
///
/// Renders nothing at all when nobody has signed up, so a routine change is not
/// dressed up as a dangerous one.
#[component]
fn AffectedBookings(session_id: String) -> impl IntoView {
    let contacts = Resource::new(
        move || session_id.clone(),
        |id| async move { session_contacts(id).await },
    );

    view! {
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                match contacts.await {
                    Err(_) => EitherOf3::A(()),
                    Ok(people) if people.is_empty() => EitherOf3::B(()),
                    Ok(people) => {
                        let total: u32 = people.iter().map(|person| person.persons).sum();
                        let rows = people
                            .iter()
                            .map(|person| {
                                view! {
                                    <li>
                                        <span class="font-medium">{person.name.clone()}</span>
                                        " · "
                                        <a
                                            href=format!("mailto:{}", person.email)
                                            class="underline underline-offset-4"
                                        >
                                            {person.email.clone()}
                                        </a>
                                        " · "
                                        <a
                                            href=format!("tel:{}", person.phone.replace(' ', ""))
                                            class="underline underline-offset-4"
                                        >
                                            {person.phone.clone()}
                                        </a>
                                        {format!(" · {} pers.", person.persons)}
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>();

                        EitherOf3::C(
                            view! {
                                <Alert variant=AlertVariant::Destructive class="mb-6">
                                    <AlertTitle class="text-base">
                                        "⚠ Opération dangereuse"
                                    </AlertTitle>
                                    <AlertDescription>
                                        <p class="mb-2">
                                            {format!(
                                                "{} réservation(s), soit {total} personne(s), portent déjà sur cette séance. Pensez à contacter les personnes concernées.",
                                                people.len(),
                                            )}
                                        </p>
                                        <ul class="space-y-1 list-disc list-inside">{rows}</ul>
                                    </AlertDescription>
                                </Alert>
                            },
                        )
                    }
                }
            })}
        </Transition>
    }
}
