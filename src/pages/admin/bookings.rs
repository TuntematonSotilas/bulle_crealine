use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::bookings::{DeleteBooking, SaveAdminComment, all_bookings};
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
    let delete_booking = ServerAction::<DeleteBooking>::new();

    // Reloads after either write, so the tables show what was stored rather than
    // what was typed, and a deleted booking moves down on its own.
    let bookings = Resource::new(
        move || (save_comment.version().get(), delete_booking.version().get()),
        |_| async move { all_bookings().await },
    );

    let write_error = move || {
        let comment_failure = save_comment.value().get().and_then(|outcome| outcome.err());
        let delete_failure = delete_booking.value().get().and_then(|outcome| outcome.err());

        comment_failure
            .or(delete_failure)
            .map(|error| user_message(&error))
    };

    view! {
        <Title text="Réservations — Administration"/>

        <AdminShell title="Réservations" current="/admin/bookings">

            {move || {
                write_error()
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
                            let (deleted, live): (Vec<_>, Vec<_>) = rows
                                .into_iter()
                                .partition(|booking| booking.is_deleted);

                            Either::Right(
                                view! {
                                    <div class="space-y-10">
                                        <BookingTable
                                            rows=live
                                            comment_action=save_comment
                                            delete_action=delete_booking
                                        />
                                        <DeletedBookingTable rows=deleted/>
                                    </div>
                                },
                            )
                        }
                    }
                })}
            </Transition>

        </AdminShell>
    }
}

/// The live bookings: editable note, and a deletion that demands a reason.
#[component]
fn BookingTable(
    rows: Vec<BookingView>,
    comment_action: ServerAction<SaveAdminComment>,
    delete_action: ServerAction<DeleteBooking>,
) -> impl IntoView {
    if rows.is_empty() {
        return Either::Left(view! {
            <section class="space-y-3">
                <h2 class="text-lg font-semibold text-heading">"Réservations"</h2>
                <Alert>
                    <AlertTitle>"Aucune réservation"</AlertTitle>
                    <AlertDescription>
                        "Les réservations prises sur le site apparaîtront ici."
                    </AlertDescription>
                </Alert>
            </section>
        });
    }

    let total_persons: u32 = rows.iter().map(|booking| booking.persons).sum();
    let count = rows.len();

    let body = rows
        .into_iter()
        .map(|booking| {
            // One per form: each `ActionForm` closes over the id it posts, so a
            // single clone would be moved into the first of the two.
            let comment_id = booking.id.clone();
            let delete_id = booking.id.clone();

            view! {
                <TableRow>
                    <SessionCell booking=booking.clone()/>
                    <ClientCell booking=booking.clone()/>

                    <TableCell class="align-top text-center">{booking.persons}</TableCell>

                    <TableCell class="align-top max-w-48">
                        <p class="text-xs whitespace-pre-wrap text-muted-foreground">
                            {or_dash(&booking.comment)}
                        </p>
                    </TableCell>

                    <TableCell class="align-top whitespace-nowrap text-xs text-muted-foreground">
                        {booking.created_label.clone()}
                    </TableCell>

                    <TableCell class="align-top">
                        <ActionForm action=comment_action>
                            <input type="hidden" name="id" value=comment_id/>
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

                    <TableCell class="align-top">
                        <ActionForm action=delete_action>
                            <input type="hidden" name="id" value=delete_id/>
                            <div class="flex flex-col gap-2 min-w-56">
                                <Textarea
                                    name="reason"
                                    rows=2
                                    maxlength=2000
                                    required=true
                                    placeholder="Motif de suppression (obligatoire)…"
                                    class="text-xs"
                                />
                                <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm>
                                    "Supprimer"
                                </Button>
                            </div>
                        </ActionForm>
                    </TableCell>
                </TableRow>
            }
        })
        .collect::<Vec<_>>();

    Either::Right(view! {
        <section class="space-y-3">
            <h2 class="text-lg font-semibold text-heading">"Réservations"</h2>
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
                            <TableHead>"Supprimer"</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>{body}</TableBody>
                    <TableCaption>
                        {format!("{count} réservation(s) · {total_persons} personne(s) au total")}
                    </TableCaption>
                </Table>
            </TableContainer>
        </section>
    })
}

/// The bookings the admin removed, with the reason given at the time.
///
/// Read-only: these no longer count towards a session's capacity, and nothing
/// here brings one back.
#[component]
fn DeletedBookingTable(rows: Vec<BookingView>) -> impl IntoView {
    if rows.is_empty() {
        return Either::Left(view! {
            <section class="space-y-3">
                <h2 class="text-lg font-semibold text-heading">"Réservations supprimées"</h2>
                <p class="text-sm text-muted-foreground">"Aucune réservation supprimée."</p>
            </section>
        });
    }

    let total_persons: u32 = rows.iter().map(|booking| booking.persons).sum();
    let count = rows.len();

    let body = rows
        .into_iter()
        .map(|booking| {
            view! {
                <TableRow>
                    <SessionCell booking=booking.clone()/>
                    <ClientCell booking=booking.clone()/>

                    <TableCell class="align-top text-center">{booking.persons}</TableCell>

                    <TableCell class="align-top max-w-64">
                        <p class="text-xs whitespace-pre-wrap">
                            {or_dash(&booking.deletion_comment)}
                        </p>
                    </TableCell>

                    <TableCell class="align-top max-w-48">
                        <p class="text-xs whitespace-pre-wrap text-muted-foreground">
                            {or_dash(&booking.admin_comment)}
                        </p>
                    </TableCell>

                    <TableCell class="align-top whitespace-nowrap text-xs text-muted-foreground">
                        {booking.created_label.clone()}
                    </TableCell>
                </TableRow>
            }
        })
        .collect::<Vec<_>>();

    Either::Right(view! {
        <section class="space-y-3">
            <h2 class="text-lg font-semibold text-heading">"Réservations supprimées"</h2>
            <TableContainer>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>"Séance"</TableHead>
                            <TableHead>"Client"</TableHead>
                            <TableHead class="text-center">"Pers."</TableHead>
                            <TableHead>"Motif de suppression"</TableHead>
                            <TableHead>"Note interne"</TableHead>
                            <TableHead>"Reçue le"</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>{body}</TableBody>
                    <TableCaption>
                        {format!(
                            "{count} réservation(s) supprimée(s) · {total_persons} personne(s), qui ne comptent plus dans les places prises",
                        )}
                    </TableCaption>
                </Table>
            </TableContainer>
        </section>
    })
}

/// Booked session, shared by both tables.
#[component]
fn SessionCell(booking: BookingView) -> impl IntoView {
    view! {
        <TableCell class="align-top">
            <div class="font-medium">{booking.session_date_label.clone()}</div>
            <div class="text-xs text-muted-foreground">
                {booking.service_type.label()}
                {(!booking.session_theme.is_empty())
                    .then(|| format!(" · {}", booking.session_theme))}
            </div>
        </TableCell>
    }
}

/// Contact details, shared by both tables.
#[component]
fn ClientCell(booking: BookingView) -> impl IntoView {
    view! {
        <TableCell class="align-top">
            <div class="font-medium">{booking.name.clone()}</div>
            // Skipped when absent: the address is optional now, and a mailto:
            // pointing at nothing would still look like a link.
            {(!booking.email.is_empty())
                .then(|| {
                    let email = booking.email.clone();
                    view! {
                        <div class="text-xs">
                            <a
                                href=format!("mailto:{email}")
                                class="underline underline-offset-4"
                            >
                                {email.clone()}
                            </a>
                        </div>
                    }
                })}
            <div class="text-xs">
                <a
                    href=format!("tel:{}", booking.phone.replace(' ', ""))
                    class="underline underline-offset-4"
                >
                    {booking.phone.clone()}
                </a>
            </div>
        </TableCell>
    }
}

/// An em dash stands in for an empty note, so a cell never looks unfinished.
fn or_dash(text: &str) -> String {
    if text.is_empty() {
        "—".to_owned()
    } else {
        text.to_owned()
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn booking() -> BookingView {
        BookingView {
            id: "651d1f0a0000000000000001".to_owned(),
            session_id: "651d1f0a0000000000000002".to_owned(),
            service_type: crate::models::ServiceType::AperosCreatifs,
            session_date_label: "samedi 12 avril à 14h".to_owned(),
            session_theme: "Aquarelle".to_owned(),
            name: "Alice Martin".to_owned(),
            email: "alice@example.com".to_owned(),
            phone: "06 12 34 56 78".to_owned(),
            persons: 2,
            comment: "Je viens avec ma fille.".to_owned(),
            admin_comment: "Rappeler la veille".to_owned(),
            created_label: "12/03 à 09:14".to_owned(),
            is_deleted: false,
            deletion_comment: String::new(),
        }
    }

    fn render_live(rows: Vec<BookingView>) -> String {
        Owner::new().with(|| {
            let comment_action = ServerAction::<SaveAdminComment>::new();
            let delete_action = ServerAction::<DeleteBooking>::new();
            view! {
                <BookingTable
                    rows=rows
                    comment_action=comment_action
                    delete_action=delete_action
                />
            }
            .to_html()
        })
    }

    fn render_deleted(rows: Vec<BookingView>) -> String {
        Owner::new().with(|| view! { <DeletedBookingTable rows=rows/> }.to_html())
    }

    /// The reason is the whole point of the deletion form, so it has to be posted
    /// under the name the server function reads, and be marked required.
    #[test]
    fn every_row_offers_a_deletion_that_demands_a_reason() {
        let html = render_live(vec![booking()]);

        assert!(html.contains(r#"name="reason""#), "the reason field should render: {html}");
        assert!(html.contains("required"), "and be required: {html}");
        assert!(html.contains("Supprimer"), "with a button to submit it: {html}");
        assert!(
            html.contains("Motif de suppression"),
            "and say what it is for: {html}"
        );
    }

    /// The admin note and the deletion reason are two separate forms on one row;
    /// both must carry the booking id or one of them would post nothing.
    #[test]
    fn both_row_forms_carry_the_booking_id() {
        let html = render_live(vec![booking()]);
        let occurrences = html.matches("651d1f0a0000000000000001").count();

        assert!(
            occurrences >= 2,
            "the id should appear in both forms, found {occurrences}: {html}"
        );
    }

    /// The reason given at deletion is the one thing the dedicated list adds over
    /// the main one.
    #[test]
    fn the_deleted_list_shows_the_reason() {
        let deleted = BookingView {
            is_deleted: true,
            deletion_comment: "Annulation par téléphone".to_owned(),
            ..booking()
        };

        let html = render_deleted(vec![deleted]);

        assert!(
            html.contains("Annulation par téléphone"),
            "the reason should show: {html}"
        );
        assert!(html.contains("Motif de suppression"), "under its own column: {html}");
        assert!(html.contains("Alice Martin"), "next to who booked: {html}");
    }

    /// A deleted booking no longer holds its seats, and the caption is where the
    /// admin reads that.
    #[test]
    fn the_deleted_list_says_its_seats_are_free() {
        let deleted = BookingView {
            is_deleted: true,
            deletion_comment: "Doublon".to_owned(),
            ..booking()
        };

        let html = render_deleted(vec![deleted]);

        assert!(
            html.contains("ne comptent plus dans les places prises"),
            "the caption should spell that out: {html}"
        );
    }

    /// Both lists always render, so an empty one has to say so rather than vanish
    /// and leave the page looking broken.
    #[test]
    fn each_list_states_when_it_is_empty() {
        let live = render_live(Vec::new());
        assert!(live.contains("Aucune réservation"), "{live}");

        let deleted = render_deleted(Vec::new());
        assert!(
            deleted.contains("Aucune réservation supprimée"),
            "{deleted}"
        );
        assert!(
            deleted.contains("Réservations supprimées"),
            "the heading should stay put: {deleted}"
        );
    }
}
