//! Administration of the workshop themes, at `/admin/themes`.
//!
//! Follows the same shape as [`crate::pages::admin::sessions`], with one departure:
//! the form carries a photo, which an `<ActionForm>` cannot post — it serialises to
//! an urlencoded body. So the form is a plain `<form>` whose submit builds a
//! `FormData`, and this page therefore needs JavaScript, unlike the rest of the
//! admin area.

use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use leptos_meta::Title;
use wasm_bindgen::JsCast;
use web_sys::{FormData, HtmlFormElement};

use crate::api::themes::{DeleteTheme, all_themes, save_theme, theme_sessions};
use crate::auth::user_message;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;
use crate::components::ui::table::*;
use crate::models::{MAX_PHOTO_LABEL, ThemeView};
use crate::pages::admin::AdminShell;

/// The action posting the theme form, which carries a file.
///
/// Dispatched locally rather than through a `ServerAction`: a `FormData` is neither
/// `Send` nor `Sync`, since it only ever exists in the browser.
type SaveAction = Action<FormData, Result<(), ServerFnError>>;

/// What the admin is doing to the theme list right now.
#[derive(Clone, Debug, PartialEq)]
enum Editing {
    /// Just looking.
    None,
    /// Filling in a brand new theme.
    New,
    /// Changing an existing one.
    Theme(ThemeView),
    /// About to drop one.
    Deleting(ThemeView),
}

/// Theme management.
#[component]
pub fn AdminThemesPage() -> impl IntoView {
    let save: SaveAction = Action::new_local(|data: &FormData| {
        let data = data.clone();
        save_theme(data.into())
    });
    let delete = ServerAction::<DeleteTheme>::new();
    let editing = RwSignal::new(Editing::None);

    // Reloads whenever either action reports back, so the table follows the writes.
    let themes = Resource::new(
        move || (save.version().get(), delete.version().get()),
        |_| async move { all_themes().await },
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
        <Title text="Thèmes — Administration"/>

        <AdminShell title="Thèmes" current="/admin/themes">

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
                                    "Nouveau thème"
                                </Button>
                            </div>
                        },
                    )
                }
                Editing::New | Editing::Theme(_) => {
                    let theme = match editing.get() {
                        Editing::Theme(theme) => Some(theme),
                        _ => None,
                    };

                    EitherOf3::B(
                        view! {
                            <ThemeForm
                                action=save
                                theme=theme
                                error=Signal::derive(save_error)
                                on_cancel=cancel
                            />
                        },
                    )
                }
                Editing::Deleting(theme) => {
                    EitherOf3::C(
                        view! {
                            <DeleteConfirmation action=delete theme=theme on_cancel=cancel/>
                        },
                    )
                }
            }}

            <Transition fallback=|| {
                view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
            }>
                {move || Suspend::new(async move {
                    match themes.await {
                        Err(error) => {
                            Either::Left(
                                view! {
                                    <Alert variant=AlertVariant::Destructive>
                                        {user_message(&error)}
                                    </Alert>
                                },
                            )
                        }
                        Ok(rows) => Either::Right(view! { <ThemeTable rows=rows editing=editing/> }),
                    }
                })}
            </Transition>

        </AdminShell>
    }
}

/// The listing itself.
#[component]
fn ThemeTable(rows: Vec<ThemeView>, editing: RwSignal<Editing>) -> impl IntoView {
    if rows.is_empty() {
        return Either::Left(view! {
            <Alert>
                <AlertTitle>"Aucun thème"</AlertTitle>
                <AlertDescription>
                    "Créez un thème pour pouvoir l'associer à une séance."
                </AlertDescription>
            </Alert>
        });
    }

    let body = rows
        .into_iter()
        .map(|theme| {
            let for_edit = theme.clone();
            let for_delete = theme.clone();

            // Read out of `theme` before the closures below capture it.
            let name = theme.name.clone();
            let photo_url = theme.photo_url.clone();
            let alt = theme.name.clone();

            view! {
                <TableRow>
                    <TableCell>
                        <img
                            src=photo_url
                            alt=alt
                            class="object-cover w-16 h-16 rounded-md border"
                            loading="lazy"
                        />
                    </TableCell>
                    <TableCell class="font-medium">{name}</TableCell>
                    <TableCell>
                        <div class="flex gap-2 justify-end">
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                on:click=move |_| editing.set(Editing::Theme(for_edit.clone()))
                            >
                                "Modifier"
                            </Button>
                            <Button
                                variant=ButtonVariant::Destructive
                                size=ButtonSize::Sm
                                on:click=move |_| editing.set(Editing::Deleting(for_delete.clone()))
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
                        <TableHead>"Photo"</TableHead>
                        <TableHead>"Nom"</TableHead>
                        <TableHead class="text-right">"Actions"</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>{body}</TableBody>
            </Table>
        </TableContainer>
    })
}

/// Create or edit form. `theme` being `None` means a creation.
#[component]
fn ThemeForm(
    action: SaveAction,
    theme: Option<ThemeView>,
    #[prop(into)] error: Signal<Option<String>>,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    let id = theme.as_ref().map(|t| t.id.clone()).unwrap_or_default();
    let editing_existing = !id.is_empty();
    let name = theme.as_ref().map(|t| t.name.clone()).unwrap_or_default();
    let current_photo = theme.as_ref().map(|t| t.photo_url.clone());

    // Hand-rolled instead of `<ActionForm>`: only a multipart body can carry a file.
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let Some(form) = ev
            .target()
            .and_then(|target| target.dyn_into::<HtmlFormElement>().ok())
        else {
            return;
        };
        if let Ok(data) = FormData::new_with_form(&form) {
            action.dispatch_local(data);
        }
    };

    let pending = action.pending();

    view! {
        <Card>
            <CardHeader>
                <CardTitle>
                    {if editing_existing { "Modifier le thème" } else { "Nouveau thème" }}
                </CardTitle>
                <CardDescription>
                    "Un thème porte un nom et une photo, et se choisit ensuite sur une séance."
                </CardDescription>
            </CardHeader>

            <CardContent>
                {editing_existing.then(|| view! { <AffectedSessions theme_id=id.clone()/> })}

                <form on:submit=submit>
                    <input type="hidden" name="id" value=id/>

                    <div class="flex flex-col gap-6">
                        <div class="grid gap-4 md:grid-cols-2">

                            <div class="grid gap-3">
                                <Label r#for="name">"Nom"</Label>
                                <Input
                                    id="name"
                                    name="name"
                                    required=true
                                    attr:value=name
                                />
                            </div>

                            <div class="grid gap-3">
                                <Label r#for="photo">"Photo"</Label>
                                <Input
                                    r#type=InputType::File
                                    id="photo"
                                    name="photo"
                                    // Only a creation needs a file: leaving this empty
                                    // on an edit keeps the photo already stored.
                                    required=!editing_existing
                                    attr:accept="image/png,image/jpeg,image/webp"
                                />
                                <p class="text-sm text-muted-foreground">
                                    {format!(
                                        "{}PNG, JPEG ou WebP, {MAX_PHOTO_LABEL} au maximum.",
                                        if editing_existing {
                                            "Laissez vide pour garder la photo actuelle. "
                                        } else {
                                            ""
                                        },
                                    )}
                                </p>
                            </div>

                        </div>

                        {current_photo
                            .map(|url| {
                                view! {
                                    <div class="grid gap-2">
                                        <p class="text-sm font-medium">"Photo actuelle"</p>
                                        <img
                                            src=url
                                            alt="Photo actuelle du thème"
                                            class="object-cover w-32 h-32 rounded-md border"
                                        />
                                    </div>
                                }
                            })}

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
                            <Button attr:disabled=move || pending.get()>
                                {move || {
                                    if pending.get() { "Envoi…" } else { "Enregistrer" }
                                }}
                            </Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                attr:r#type="button"
                                on:click=move |_| on_cancel()
                            >
                                "Annuler"
                            </Button>
                        </div>
                    </div>
                </form>
            </CardContent>
        </Card>
    }
}

/// Confirmation asked before a theme is dropped.
///
/// A theme still used by a session cannot go: the sessions would have nothing to
/// show. The server refuses either way — this only avoids offering a button that
/// cannot work.
#[component]
fn DeleteConfirmation(
    action: ServerAction<DeleteTheme>,
    theme: ThemeView,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    let id = theme.id.clone();
    let for_resource = theme.id.clone();

    let sessions = Resource::new(
        move || for_resource.clone(),
        |id| async move { theme_sessions(id).await },
    );

    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Supprimer ce thème ?"</CardTitle>
                <CardDescription>{theme.name.clone()}</CardDescription>
            </CardHeader>

            <CardContent>
                <Transition fallback=|| ()>
                    {move || {
                        let id = id.clone();
                        Suspend::new(async move {
                            match sessions.await {
                                // In use: say which sessions, and offer no button that
                                // the server would only turn down.
                                Ok(list) if !list.is_empty() => {
                                    EitherOf3::A(
                                        view! {
                                            <UsedBySessions sessions=list/>
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
                                        view! { <ConfirmDeletion action=action id=id on_cancel=on_cancel/> },
                                    )
                                }
                                // The count could not be read; let the attempt through,
                                // since the server checks again anyway.
                                Err(_) => {
                                    EitherOf3::C(
                                        view! { <ConfirmDeletion action=action id=id on_cancel=on_cancel/> },
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
    action: ServerAction<DeleteTheme>,
    id: String,
    on_cancel: impl Fn() + 'static + Send + Sync + Copy,
) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <input type="hidden" name="id" value=id/>
            <div class="flex gap-2">
                <Button variant=ButtonVariant::Destructive>"Oui, supprimer le thème"</Button>
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

/// The warning shown when a theme is already used by sessions.
///
/// Renders nothing when no session uses it, so a routine rename is not dressed up
/// as a dangerous operation.
#[component]
fn AffectedSessions(theme_id: String) -> impl IntoView {
    let sessions = Resource::new(
        move || theme_id.clone(),
        |id| async move { theme_sessions(id).await },
    );

    view! {
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                match sessions.await {
                    Err(_) => EitherOf3::A(()),
                    Ok(list) if list.is_empty() => EitherOf3::B(()),
                    Ok(list) => EitherOf3::C(view! { <UsedBySessions sessions=list/> }),
                }
            })}
        </Transition>
    }
}

/// Spells out which sessions a change to this theme would show up on.
#[component]
fn UsedBySessions(sessions: Vec<crate::models::AffectedSession>) -> impl IntoView {
    let count = sessions.len();
    let rows = sessions
        .iter()
        .map(|session| {
            view! {
                <li>
                    <span class="font-medium">{session.date_label.clone()}</span>
                    " · "
                    {session.service_label.clone()}
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <Alert variant=AlertVariant::Destructive class="mb-6">
            <AlertTitle class="text-base">"⚠ Ce thème est utilisé"</AlertTitle>
            <AlertDescription>
                <p class="mb-2">
                    {format!(
                        "{count} séance(s) portent ce thème : toute modification s'affichera aussi sur elles, et la suppression est impossible tant qu'elles existent.",
                    )}
                </p>
                <ul class="space-y-1 list-disc list-inside">{rows}</ul>
            </AlertDescription>
        </Alert>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn theme() -> ThemeView {
        ThemeView {
            id: "651d1f0a0000000000000001".to_owned(),
            name: "Aquarelle".to_owned(),
            photo_url: "/media/theme/651d1f0a0000000000000001?v=1".to_owned(),
        }
    }

    /// A creation has no photo to show yet, and must ask for one.
    #[test]
    fn creation_form_requires_a_photo_and_shows_none() {
        let html = Owner::new().with(|| {
            // Bound out here: `view!` would read the turbofish as a tag.
            let action: SaveAction = Action::new_local(|_: &FormData| async { Ok(()) });
            let error: Signal<Option<String>> = Signal::derive(|| None);
            view! { <ThemeForm action=action theme=None error=error on_cancel=|| {}/> }.to_html()
        });

        assert!(html.contains(r#"type="file""#), "the photo input should render: {html}");
        assert!(html.contains("required"), "a creation should demand a photo: {html}");
        assert!(!html.contains("Photo actuelle"), "there is no photo yet: {html}");
        assert!(html.contains("Nouveau thème"), "the title should say so: {html}");
    }

    /// An edit shows the stored photo and lets the file input stay empty, which is
    /// what "keep the current photo" means.
    #[test]
    fn edit_form_shows_the_stored_photo() {
        // An edit renders the impact warning, which owns a `Resource`.
        crate::pages::admin::init_test_executor();

        let html = Owner::new().with(|| {
            let action: SaveAction = Action::new_local(|_: &FormData| async { Ok(()) });
            let error: Signal<Option<String>> = Signal::derive(|| None);
            view! {
                <ThemeForm action=action theme=Some(theme()) error=error on_cancel=|| {}/>
            }
            .to_html()
        });

        assert!(html.contains("Photo actuelle"), "the stored photo should show: {html}");
        assert!(html.contains(&theme().photo_url), "and point at the media route: {html}");
        assert!(html.contains("Aquarelle"), "the name should be filled in: {html}");
        assert!(
            html.contains("garder la photo actuelle"),
            "and the hint should say the file may stay empty: {html}"
        );
    }

    /// The table is the only place a photo is listed, and it must not be dressed up
    /// as an empty state.
    #[test]
    fn table_lists_a_thumbnail_per_theme() {
        let html = Owner::new().with(|| {
            let editing = RwSignal::new(Editing::None);
            view! { <ThemeTable rows=vec![theme()] editing=editing/> }.to_html()
        });

        assert!(html.contains(&theme().photo_url), "the thumbnail should render: {html}");
        assert!(html.contains("Aquarelle"), "next to the name: {html}");
    }

    #[test]
    fn table_says_so_when_there_is_nothing_yet() {
        let html = Owner::new().with(|| {
            let editing = RwSignal::new(Editing::None);
            view! { <ThemeTable rows=vec![] editing=editing/> }.to_html()
        });

        assert!(html.contains("Aucun thème"), "the empty state should show: {html}");
    }

    /// The warning is what tells the admin a rename is not harmless.
    #[test]
    fn the_warning_lists_every_session_using_the_theme() {
        let html = Owner::new().with(|| {
            let sessions = vec![
                crate::models::AffectedSession {
                    date_label: "dimanche 5 juillet 2026 à 14h00".to_owned(),
                    service_label: "Apéros créatifs (adultes)".to_owned(),
                },
                crate::models::AffectedSession {
                    date_label: "lundi 6 juillet 2026 à 10h00".to_owned(),
                    service_label: "Ateliers parents-enfants (6 à 12 ans)".to_owned(),
                },
            ];
            view! { <UsedBySessions sessions=sessions/> }.to_html()
        });

        assert!(html.contains("2 séance(s)"), "the count should show: {html}");
        assert!(html.contains("dimanche 5 juillet 2026 à 14h00"), "and each session: {html}");
        assert!(html.contains("lundi 6 juillet 2026 à 10h00"), "including the second: {html}");
    }
}
