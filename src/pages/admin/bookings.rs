use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::bookings::{SaveAdminComment, all_bookings};
use crate::auth::user_message;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::table::*;
use crate::components::ui::textarea::Textarea;
use crate::models::BookingView;
use crate::pages::admin::AdminShell;

/// Booking listing, at `/admin/bookings`.
#[component]
pub fn AdminBookingsPage() -> impl IntoView {
    let save_comment = ServerAction::<SaveAdminComment>::new();

    // Reloads after a comment is saved, so the table shows what was stored rather
    // than what was typed.
    let bookings = Resource::new(
        move || save_comment.version().get(),
        |_| async move { all_bookings().await },
    );

    let comment_error = move || {
        save_comment
            .value()
            .get()
            .and_then(|outcome| outcome.err())
            .map(|error| user_message(&error))
    };

    view! {
        <Title text="Réservations — Administration"/>

        <AdminShell title="Réservations" current="/admin/bookings">

            {move || {
                comment_error()
                    .map(|message| {
                        view! { <Alert variant=AlertVariant::Destructive>{message}</Alert> }
                    })
            }}

            <Transition fallback=|| {
                view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
            }>
                {move || Suspend::new(async move {
                    match bookings.await {
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
                            Either::Right(view! { <BookingTable rows=rows action=save_comment/> })
                        }
                    }
                })}
            </Transition>

        </AdminShell>
    }
}

#[component]
fn BookingTable(
    rows: Vec<BookingView>,
    action: ServerAction<SaveAdminComment>,
) -> impl IntoView {
    if rows.is_empty() {
        return Either::Left(view! {
            <Alert>
                <AlertTitle>"Aucune réservation"</AlertTitle>
                <AlertDescription>
                    "Les réservations prises sur le site apparaîtront ici."
                </AlertDescription>
            </Alert>
        });
    }

    let total_persons: u32 = rows.iter().map(|booking| booking.persons).sum();
    let count = rows.len();

    let body = rows
        .into_iter()
        .map(|booking| {
            view! {
                <TableRow>
                    <TableCell class="align-top">
                        <div class="font-medium">{booking.session_date_label.clone()}</div>
                        <div class="text-xs text-muted-foreground">
                            {booking.service_type.label()}
                            {(!booking.session_theme.is_empty())
                                .then(|| format!(" · {}", booking.session_theme))}
                        </div>
                    </TableCell>

                    <TableCell class="align-top">
                        <div class="font-medium">{booking.name.clone()}</div>
                        <div class="text-xs">
                            <a
                                href=format!("mailto:{}", booking.email)
                                class="underline underline-offset-4"
                            >
                                {booking.email.clone()}
                            </a>
                        </div>
                        <div class="text-xs">
                            <a
                                href=format!("tel:{}", booking.phone.replace(' ', ""))
                                class="underline underline-offset-4"
                            >
                                {booking.phone.clone()}
                            </a>
                        </div>
                    </TableCell>

                    <TableCell class="align-top text-center">{booking.persons}</TableCell>

                    <TableCell class="align-top max-w-48">
                        <p class="text-xs whitespace-pre-wrap text-muted-foreground">
                            {if booking.comment.is_empty() {
                                "—".to_owned()
                            } else {
                                booking.comment.clone()
                            }}
                        </p>
                    </TableCell>

                    <TableCell class="align-top whitespace-nowrap text-xs text-muted-foreground">
                        {booking.created_label.clone()}
                    </TableCell>

                    <TableCell class="align-top">
                        <ActionForm action=action>
                            <input type="hidden" name="id" value=booking.id.clone()/>
                            <div class="flex flex-col gap-2 min-w-56">
                                <Textarea
                                    name="comment"
                                    rows=2
                                    maxlength=2000
                                    placeholder="Note interne…"
                                    value=booking.admin_comment.clone()
                                    class="text-xs"
                                />
                                <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                                    "Enregistrer"
                                </Button>
                            </div>
                        </ActionForm>
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
                        <TableHead>"Séance"</TableHead>
                        <TableHead>"Client"</TableHead>
                        <TableHead class="text-center">"Pers."</TableHead>
                        <TableHead>"Commentaire du client"</TableHead>
                        <TableHead>"Reçue le"</TableHead>
                        <TableHead>"Note interne"</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>{body}</TableBody>
                <TableCaption>
                    {format!("{count} réservation(s) · {total_persons} personne(s) au total")}
                </TableCaption>
            </Table>
        </TableContainer>
    })
}
