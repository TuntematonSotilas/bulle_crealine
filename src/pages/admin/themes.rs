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

use crate::api::services::all_services;
use crate::api::themes::{DeleteTheme, all_themes, save_theme, theme_sessions};
use crate::auth::user_message;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;
use crate::components::ui::select::{
    Select, SelectContent, SelectGroup, SelectOption, SelectTrigger, SelectValue,
};
use crate::components::ui::table::*;
use crate::models::{MAX_PHOTO_LABEL, ThemeView};
use crate::pages::admin::AdminShell;

/// The action posting the theme form, which carries a file.
///
/// Dispatched locally rather than through a `ServerAction`: a `FormData` is neither
/// `Send` nor `Sync`, since it only ever exists in the browser.
type SaveAction = Action<FormData, Result<(), ServerFnError>>;

/// How the picker words "no workshop", which posts an empty slug.
const NO_SERVICE: &str = "Aucun";

/// What the filter picks to mean "do not filter".
///
/// Safe as a sentinel: [`crate::models::is_valid_slug`] allows only lowercase
/// letters, digits and hyphens, so no workshop can ever answer to it. The empty
/// string is left to mean the themes attached to nothing, which is what they
/// actually store.
const ALL_SERVICES: &str = "*";

/// How the filter words its two choices that are not a workshop.
const ALL_SERVICES_LABEL: &str = "Tous les services";
const UNATTACHED_LABEL: &str = "Sans service";

/// Which themes the table shows.
#[derive(Clone, Debug, Default, PartialEq)]
enum ServiceFilter {
    #[default]
    All,
    /// Only the themes attached to this workshop.
    Workshop(String),
    /// Only the themes attached to none.
    Unattached,
}

impl ServiceFilter {
    /// Reads back what the picker chose. Anything unexpected means no filter,
    /// which is the harmless reading: showing too much, never too little.
    fn from_choice(choice: Option<String>) -> Self {
        match choice.as_deref() {
            None | Some(ALL_SERVICES) => Self::All,
            Some("") => Self::Unattached,
            Some(slug) => Self::Workshop(slug.to_owned()),
        }
    }

    /// The value the picker has to open on to be showing this filter.
    fn choice(&self) -> String {
        match self {
            Self::All => ALL_SERVICES.to_owned(),
            Self::Workshop(slug) => slug.clone(),
            Self::Unattached => String::new(),
        }
    }

    fn keeps(&self, theme: &ThemeView) -> bool {
        match self {
            Self::All => true,
            Self::Workshop(slug) => &theme.service_slug == slug,
            Self::Unattached => theme.service_slug.is_empty(),
        }
    }
}

/// The rows a filter leaves on screen.
///
/// A filter matching nothing is treated as no filter at all: the last theme of a
/// workshop can be deleted while that workshop is the one being shown, and a table
/// emptied by a choice that is no longer offered would be a dead end.
fn matching(rows: &[ThemeView], filter: &ServiceFilter) -> Vec<ThemeView> {
    let kept: Vec<ThemeView> = rows.iter().filter(|theme| filter.keeps(theme)).cloned().collect();

    if kept.is_empty() { rows.to_vec() } else { kept }
}

/// The workshops the filter offers, as `(value, label)`, without the "all" entry
/// the picker adds itself.
///
/// Read off the themes rather than off [`all_services`]: a workshop carrying no
/// theme would be a choice leading nowhere, and a dead link -- the slug of a
/// workshop deleted since -- has no workshop left to be read from, yet its themes
/// still need grouping. Each label carries its count, which is the whole reason to
/// look at this list before picking from it.
fn filter_choices(rows: &[ThemeView]) -> Vec<(String, String)> {
    let mut groups: Vec<(String, String, usize)> = Vec::new();

    for theme in rows {
        match groups.iter_mut().find(|(slug, _, _)| *slug == theme.service_slug) {
            Some((_, _, count)) => *count += 1,
            None => groups.push((theme.service_slug.clone(), theme.service_label.clone(), 1)),
        }
    }

    // Alphabetically, the themes arriving in an order that is none of the picker's
    // business. The unattached go last whatever their wording sorts to: they are
    // the odd entry out, not one workshop among the others.
    groups.sort_by(|left, right| {
        left.0.is_empty().cmp(&right.0.is_empty()).then_with(|| left.1.cmp(&right.1))
    });

    groups
        .into_iter()
        .map(|(slug, label, count)| {
            let wording = if slug.is_empty() { UNATTACHED_LABEL } else { label.as_str() };
            (slug, format!("{wording} ({count})"))
        })
        .collect()
}

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

    // Held here rather than in the table, which is rebuilt every time a write
    // reloads the list: editing a theme would otherwise drop the admin back to the
    // whole list, right after they narrowed it down to find that theme.
    let filter = RwSignal::new(ServiceFilter::default());

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
                        Ok(rows) => {
                            Either::Right(
                                view! { <ThemeTable rows=rows filter=filter editing=editing/> },
                            )
                        }
                    }
                })}
            </Transition>

        </AdminShell>
    }
}

/// The listing itself, under the workshop filter.
#[component]
fn ThemeTable(
    rows: Vec<ThemeView>,
    filter: RwSignal<ServiceFilter>,
    editing: RwSignal<Editing>,
) -> impl IntoView {
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

    let total = rows.len();
    let choices = filter_choices(&rows);
    // A single choice covers every theme on the page, so the picker would filter
    // nothing: one more control to read past, and no way to change what is shown.
    let worth_filtering = choices.len() > 1;

    // Stored rather than moved into the closure: the filter re-runs it, and each
    // run needs the full list again to narrow it down afresh.
    let rows = StoredValue::new(rows);
    let body = move || {
        rows.with_value(|rows| matching(rows, &filter.get()))
            .into_iter()
            .map(|theme| view! { <ThemeRow theme=theme editing=editing/> })
            .collect::<Vec<_>>()
    };

    Either::Right(view! {
        {worth_filtering
            .then(|| view! { <ThemeFilter choices=choices total=total filter=filter/> })}

        <TableContainer>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>"Photo"</TableHead>
                        <TableHead>"Nom"</TableHead>
                        <TableHead>"Service"</TableHead>
                        <TableHead class="text-right">"Actions"</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>{body}</TableBody>
            </Table>
        </TableContainer>
    })
}

/// One theme in the listing.
#[component]
fn ThemeRow(theme: ThemeView, editing: RwSignal<Editing>) -> impl IntoView {
    let for_edit = theme.clone();
    let for_delete = theme.clone();

    // Read out of `theme` before the closures below capture it.
    let name = theme.name.clone();
    let photo_url = theme.photo_url.clone();
    let alt = theme.name.clone();
    // An em dash rather than a blank cell: the link is optional, and a gap reads
    // as missing data rather than as a deliberate "none".
    let service = if theme.service_label.is_empty() {
        "—".to_owned()
    } else {
        theme.service_label.clone()
    };

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
            <TableCell class="text-muted-foreground">{service}</TableCell>
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
}

/// Narrows the listing to one workshop, or to the themes attached to none.
///
/// Filters the rows already in hand rather than asking the server again: the list
/// is a page of an admin screen, not a catalogue, and a round trip would cost more
/// than the whole table is worth.
#[component]
fn ThemeFilter(
    /// `(value, label)` per workshop, from [`filter_choices`].
    choices: Vec<(String, String)>,
    /// How many themes there are in all, to word the "all" entry.
    total: usize,
    filter: RwSignal<ServiceFilter>,
) -> impl IntoView {
    let all_label = format!("{ALL_SERVICES_LABEL} ({total})");

    // Falls back to showing everything when the chosen workshop is no longer among
    // them, which is what `matching` does with those rows: the trigger must not
    // claim a filter the table is not applying.
    let current = filter.get_untracked().choice();
    let (value, label) = choices
        .iter()
        .find(|(value, _)| *value == current)
        .cloned()
        .unwrap_or_else(|| (ALL_SERVICES.to_owned(), all_label.clone()));

    let pick = Callback::new(move |choice: Option<String>| {
        filter.set(ServiceFilter::from_choice(choice));
    });

    view! {
        <div class="flex flex-wrap gap-3 items-center mb-4">
            <Label r#for="theme-filter">"Filtrer par service"</Label>

            // No `name`: this picker sits outside any form and posts nothing, the
            // choice being read through `on_change` alone.
            <Select default_value=value default_label=label on_change=pick>
                <SelectTrigger id="theme-filter">
                    <SelectValue placeholder=ALL_SERVICES_LABEL/>
                </SelectTrigger>
                <SelectContent>
                    <SelectGroup>
                        <SelectOption value=ALL_SERVICES.to_string() label=all_label.clone()>
                            {all_label}
                        </SelectOption>
                        // Built here rather than above: an option reads the
                        // `Select`'s context, which only exists inside this view.
                        {choices
                            .into_iter()
                            .map(|(value, label)| {
                                view! {
                                    <SelectOption value=value label=label.clone()>
                                        {label}
                                    </SelectOption>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </SelectGroup>
                </SelectContent>
            </Select>
        </div>
    }
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

    // The workshop this theme already points at, if it still points at a live one.
    let linked = theme.as_ref().and_then(|t| {
        (!t.service_slug.is_empty())
            .then(|| (t.service_slug.clone(), t.service_label.clone()))
    });
    let services = Resource::new(|| (), |()| async move { all_services().await });

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

                            <div class="grid gap-3">
                                <Label r#for="service">"Service"</Label>
                                <Transition fallback=|| {
                                    view! {
                                        <p class="text-sm text-muted-foreground">"Chargement…"</p>
                                    }
                                }>
                                    {move || {
                                        let selected = linked.clone();

                                        Suspend::new(async move {
                                            match services.await {
                                                Err(error) => Either::Left(view! {
                                                    <Alert variant=AlertVariant::Destructive>
                                                        {user_message(&error)}
                                                    </Alert>
                                                }),
                                                Ok(list) => Either::Right(view! {
                                                    <ServicePicker services=list selected=selected/>
                                                }),
                                            }
                                        })
                                    }}
                                </Transition>
                                <p class="text-sm text-muted-foreground">
                                    "Facultatif. Sert à ranger le thème sous un service dans le catalogue."
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

/// The workshop picker itself, split from the fetch above so it can be rendered
/// from a plain `Vec`: what a `Transition` wraps never resolves under a
/// synchronous `to_html`, which would leave the picker untested.
///
/// `selected` is the workshop the theme already points at, as `(slug, label)`.
#[component]
fn ServicePicker(
    services: Vec<crate::models::ServiceView>,
    selected: Option<(String, String)>,
) -> impl IntoView {
    // Only the bookable ones. A workshop run for a structure agrees on its dates
    // directly and has no catalogue of themes to be ranged under, which is the
    // one thing this link is for.
    let services: Vec<_> = services.into_iter().filter(|service| !service.pro).collect();

    // Falls back to no link when the stored slug is not among them -- a workshop
    // deleted since, or one moved into the section for structures.
    let (value, label) = selected
        .filter(|(slug, _)| services.iter().any(|service| &service.slug == slug))
        .unwrap_or_else(|| (String::new(), NO_SERVICE.to_owned()));

    view! {
        <Select
            class="w-full max-w-sm"
            name="service".to_string()
            default_value=value
            default_label=label
        >
            <SelectTrigger id="service">
                <SelectValue placeholder="Service"/>
            </SelectTrigger>
            <SelectContent>
                <SelectGroup>
                    // The link is optional, so "none" has to be pickable and not
                    // merely the state the form starts in.
                    <SelectOption value=String::new() label=NO_SERVICE.to_string()>
                        {NO_SERVICE}
                    </SelectOption>
                    // Built here rather than above: an option reads the `Select`'s
                    // context, which only exists inside this view.
                    {services
                        .into_iter()
                        .map(|service| {
                            view! {
                                <SelectOption value=service.slug label=service.label.clone()>
                                    {service.label}
                                </SelectOption>
                            }
                        })
                        .collect::<Vec<_>>()}
                </SelectGroup>
            </SelectContent>
        </Select>
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
            service_slug: "aperos-creatifs".to_owned(),
            service_label: "Apéros créatifs".to_owned(),
        }
    }

    /// A theme left unattached, which is the ordinary case: the link is optional.
    fn unlinked() -> ThemeView {
        ThemeView {
            service_slug: String::new(),
            service_label: String::new(),
            ..theme()
        }
    }

    /// A theme under another workshop, so a filter can be seen to leave one out.
    fn other() -> ThemeView {
        ThemeView {
            id: "651d1f0a0000000000000002".to_owned(),
            name: "Bijoux en résine".to_owned(),
            service_slug: "ateliers-parents-enfants".to_owned(),
            service_label: "Ateliers parents-enfants".to_owned(),
            ..theme()
        }
    }

    fn table_html(rows: Vec<ThemeView>) -> String {
        Owner::new().with(|| {
            let filter = RwSignal::new(ServiceFilter::default());
            let editing = RwSignal::new(Editing::None);

            view! { <ThemeTable rows=rows filter=filter editing=editing/> }.to_html()
        })
    }

    /// A creation has no photo to show yet, and must ask for one.
    #[test]
    fn creation_form_requires_a_photo_and_shows_none() {
        // The workshop picker owns a `Resource`, on a creation as on an edit.
        crate::pages::admin::init_test_executor();

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

    fn workshop(slug: &str, label: &str, pro: bool) -> crate::models::ServiceView {
        crate::models::ServiceView {
            id: "651d1f0a0000000000000009".to_owned(),
            slug: slug.to_owned(),
            label: label.to_owned(),
            description: String::new(),
            age: String::new(),
            steps: Vec::new(),
            icon: String::new(),
            pro,
            position: 0,
        }
    }

    /// One of each section, so an assertion about the bookable one cannot pass on
    /// the other.
    fn workshops() -> Vec<crate::models::ServiceView> {
        vec![
            workshop("aperos-creatifs", "Apéros créatifs", false),
            workshop("en-institution", "Ateliers en institution", true),
        ]
    }

    fn picker_html(selected: Option<(String, String)>) -> String {
        Owner::new()
            .with(|| view! { <ServicePicker services=workshops() selected=selected/> }.to_html())
    }

    /// The link is optional, so "none" has to be pickable rather than merely the
    /// state the form starts in. Asserted on the wording: `SelectOption` keeps its
    /// value in a closure and renders no `value` attribute at all.
    #[test]
    fn the_picker_offers_no_workshop_and_every_workshop() {
        let html = picker_html(None);

        assert!(html.contains(NO_SERVICE), "no way to leave it unattached: {html}");
        assert!(html.contains("Apéros créatifs"), "no workshop offered: {html}");
        assert!(html.contains(r#"name="service""#), "the form must post it: {html}");
    }

    /// A workshop run for a structure agrees on its dates directly and has no
    /// catalogue of themes to be ranged under.
    #[test]
    fn the_picker_leaves_out_the_workshops_for_structures() {
        let html = picker_html(None);

        assert!(
            !html.contains("Ateliers en institution"),
            "a workshop for structures should not be offered: {html}"
        );
    }

    /// A theme can outlive the section its workshop was in. Keeping the stored
    /// slug preselected would post back a workshop the picker no longer offers.
    #[test]
    fn a_link_to_a_workshop_for_structures_falls_back_to_none() {
        let selected = Some(("en-institution".to_owned(), "Ateliers en institution".to_owned()));

        assert!(
            !picker_html(selected).contains(r#"value="en-institution""#),
            "a workshop moved out of reach should not stay preselected"
        );
    }

    /// An edit has to open on the link already stored, or saving without touching
    /// the picker would quietly drop it.
    #[test]
    fn the_picker_opens_on_the_workshop_already_linked() {
        let selected = Some(("aperos-creatifs".to_owned(), "Apéros créatifs".to_owned()));

        assert!(
            picker_html(selected).contains(r#"value="aperos-creatifs""#),
            "the stored link should be the one posted back"
        );
    }

    /// A slug naming no workshop -- deleted since -- must not be posted back as if
    /// it were still a choice.
    #[test]
    fn a_dead_link_falls_back_to_no_workshop() {
        let selected = Some(("supprime".to_owned(), "Service supprimé".to_owned()));

        assert!(
            !picker_html(selected).contains(r#"value="supprime""#),
            "a workshop that no longer exists should not be preselected"
        );
    }

    /// The field has to say it can be left alone; nothing else on this form can.
    #[test]
    fn the_form_says_the_link_is_optional() {
        crate::pages::admin::init_test_executor();

        let html = Owner::new().with(|| {
            let action: SaveAction = Action::new_local(|_: &FormData| async { Ok(()) });
            let error: Signal<Option<String>> = Signal::derive(|| None);
            view! { <ThemeForm action=action theme=None error=error on_cancel=|| {}/> }.to_html()
        });

        assert!(html.contains("Service"), "no field: {html}");
        assert!(html.contains("Facultatif"), "it should say it is optional: {html}");
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
        let html = table_html(vec![theme()]);

        assert!(html.contains(&theme().photo_url), "the thumbnail should render: {html}");
        assert!(html.contains("Aquarelle"), "next to the name: {html}");
    }

    /// The column is what the whole field is for on this page.
    #[test]
    fn the_table_names_the_workshop_a_theme_belongs_to() {
        let html = table_html(vec![theme()]);

        assert!(html.contains("Service"), "no column heading: {html}");
        assert!(html.contains("Apéros créatifs"), "no workshop: {html}");
    }

    /// Most themes carry no link, and an empty cell would read as data missing
    /// rather than as a deliberate "none".
    #[test]
    fn an_unlinked_theme_shows_a_dash() {
        let html = table_html(vec![unlinked()]);

        assert!(html.contains("—"), "no dash: {html}");
        assert!(html.contains("Aquarelle"), "the theme still shows: {html}");
    }

    #[test]
    fn table_says_so_when_there_is_nothing_yet() {
        assert!(
            table_html(vec![]).contains("Aucun thème"),
            "the empty state should show"
        );
    }

    /// One entry per workshop, each saying how many themes are behind it -- which
    /// is the whole reason to read the list before picking from it.
    #[test]
    fn the_filter_counts_the_themes_of_each_workshop() {
        let choices = filter_choices(&[theme(), other(), unlinked(), other()]);

        assert_eq!(choices.len(), 3, "one entry per workshop was expected: {choices:?}");
        assert!(
            choices.contains(&("aperos-creatifs".to_owned(), "Apéros créatifs (1)".to_owned())),
            "the lone theme of its workshop is miscounted: {choices:?}"
        );
        assert!(
            choices.contains(&(
                "ateliers-parents-enfants".to_owned(),
                "Ateliers parents-enfants (2)".to_owned(),
            )),
            "two themes under one workshop should count as two: {choices:?}"
        );
    }

    /// The themes attached to nothing are a choice of their own, worded rather than
    /// left nameless, and last: they are the odd entry out, not one workshop among
    /// the others.
    #[test]
    fn the_unattached_themes_are_a_choice_and_come_last() {
        let choices = filter_choices(&[unlinked(), other(), theme()]);

        let (slug, label) = choices.last().expect("no choice at all");
        assert!(slug.is_empty(), "the unattached should come last: {choices:?}");
        assert!(label.starts_with(UNATTACHED_LABEL), "and be worded: {choices:?}");
    }

    /// The themes arrive in an order of the server's own, which would leave the
    /// choices in no order at all.
    #[test]
    fn the_filter_lists_the_workshops_alphabetically() {
        let choices = filter_choices(&[other(), theme()]);

        let labels: Vec<_> = choices.iter().map(|(_, label)| label.clone()).collect();
        let mut sorted = labels.clone();
        sorted.sort();
        assert_eq!(labels, sorted, "the workshops are not in order");
    }

    #[test]
    fn a_filter_keeps_only_the_themes_of_its_workshop() {
        let rows = vec![theme(), other(), unlinked()];
        let kept = matching(&rows, &ServiceFilter::Workshop("aperos-creatifs".to_owned()));

        assert_eq!(kept, vec![theme()], "the other workshops leaked through");
    }

    #[test]
    fn the_unattached_filter_keeps_the_themes_under_no_workshop() {
        let rows = vec![theme(), unlinked()];
        let kept = matching(&rows, &ServiceFilter::Unattached);

        assert_eq!(kept, vec![unlinked()], "an attached theme leaked through");
    }

    /// The last theme of a workshop can be deleted while that workshop is the one
    /// being shown, and the filter outlives the reload. An empty table would then
    /// be a dead end.
    #[test]
    fn a_filter_matching_nothing_shows_everything() {
        let rows = vec![theme(), other()];
        let kept = matching(&rows, &ServiceFilter::Workshop("supprime-depuis".to_owned()));

        assert_eq!(kept.len(), 2, "a stale filter should not empty the table");
    }

    /// What the picker chose has to read back as the filter that was picked, or the
    /// table would narrow to something other than the trigger shows.
    #[test]
    fn every_filter_survives_the_trip_through_the_picker() {
        for filter in [
            ServiceFilter::All,
            ServiceFilter::Workshop("aperos-creatifs".to_owned()),
            ServiceFilter::Unattached,
        ] {
            let read_back = ServiceFilter::from_choice(Some(filter.choice()));
            assert_eq!(read_back, filter, "{filter:?} did not survive");
        }
    }

    /// Asserted on the wording: `SelectOption` keeps its value in a closure and
    /// renders no `value` attribute at all.
    #[test]
    fn the_table_offers_a_filter_over_the_workshops_it_lists() {
        let html = table_html(vec![theme(), other()]);

        assert!(html.contains("Filtrer par service"), "no filter: {html}");
        assert!(html.contains(ALL_SERVICES_LABEL), "no way back to the whole list: {html}");
        assert!(html.contains("Apéros créatifs (1)"), "a workshop is missing: {html}");
        assert!(html.contains("Ateliers parents-enfants (1)"), "and so is the other: {html}");
    }

    /// With every theme under the same workshop the picker would filter nothing.
    #[test]
    fn a_single_workshop_carries_no_filter() {
        let html = table_html(vec![theme(), theme()]);

        assert!(!html.contains("Filtrer par service"), "a useless filter is shown: {html}");
        assert!(html.contains("Aquarelle"), "the table itself should still render: {html}");
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
