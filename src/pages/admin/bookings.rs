use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::bookings::{DeleteBooking, PurgeBooking, SaveAdminComment, all_bookings};
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
    let purge_booking = ServerAction::<PurgeBooking>::new();

    // Which archived booking is waiting on its confirmation, if any. Owned here
    // rather than inside the table: the listing is rebuilt after every write, and a
    // signal born down there would be a new one each time.
    let confirming = RwSignal::new(None::<String>);

    // Reloads after any write, so the tables show what was stored rather than what
    // was typed: a deleted booking moves down on its own, an erased one disappears.
    let bookings = Resource::new(
        move || {
            (
                save_comment.version().get(),
                delete_booking.version().get(),
                purge_booking.version().get(),
            )
        },
        |_| async move { all_bookings().await },
    );

    // An erasure that landed leaves its confirmation open over a row that no longer
    // exists; one that failed keeps it open, which is where the message appears.
    Effect::new(move |_| {
        if purge_booking.value().get().is_some_and(|outcome| outcome.is_ok()) {
            confirming.set(None);
        }
    });

    let write_error = move || {
        let comment_failure = save_comment.value().get().and_then(|outcome| outcome.err());
        let delete_failure = delete_booking.value().get().and_then(|outcome| outcome.err());
        let purge_failure = purge_booking.value().get().and_then(|outcome| outcome.err());

        comment_failure
            .or(delete_failure)
            .or(purge_failure)
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
                                        <DeletedBookingTable
                                            rows=deleted
                                            purge_action=purge_booking
                                            confirming=confirming
                                        />
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
/// These no longer count towards a session's capacity, and nothing here brings one
/// back. What they can do is leave for good: archiving keeps a visitor's name, phone
/// number and comment on file, and the page promises they do not stay forever.
///
/// `confirming` is a prop rather than state born here because the table is rebuilt on
/// every write -- the listing reloads after each action -- and a signal owned inside
/// would forget which row was being confirmed at the worst possible moment. It also
/// lets a test render each of the two steps, which a signal owned inside would put
/// out of reach.
#[component]
fn DeletedBookingTable(
    rows: Vec<BookingView>,
    purge_action: ServerAction<PurgeBooking>,
    confirming: RwSignal<Option<String>>,
) -> impl IntoView {
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
            let id = booking.id.clone();

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

                    <TableCell class="align-top">
                        <PurgeCell id=id action=purge_action confirming=confirming/>
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
                            <TableHead>"Effacer"</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>{body}</TableBody>
                    <TableCaption>
                        {format!(
                            "{count} réservation(s) supprimée(s) · {total_persons} personne(s), qui ne comptent plus dans les places prises. Effacer retire définitivement les coordonnées du client.",
                        )}
                    </TableCaption>
                </Table>
            </TableContainer>
        </section>
    })
}

/// Erasing one archived booking, in two steps.
///
/// Two steps rather than one, and no reason asked for: archiving already took a
/// reason, and this is the step that cannot be taken back. A single destructive
/// button next to five others is a mis-click away from a visitor's record ceasing to
/// exist, with nothing left to say what it was.
#[component]
fn PurgeCell(
    id: String,
    action: ServerAction<PurgeBooking>,
    confirming: RwSignal<Option<String>>,
) -> impl IntoView {
    // `Copy`, so the closure below stays `FnMut`: it runs again on every change to
    // `confirming`, and a captured `String` would be moved out on the first run.
    let id = StoredValue::new(id);

    move || {
        let asked = id.get_value();

        if confirming.get().as_deref() == Some(asked.as_str()) {
            Either::Left(view! {
                <ActionForm action=action>
                    <input type="hidden" name="id" value=asked/>
                    <div class="flex flex-col gap-2">
                        <p class="text-xs text-muted-foreground">
                            "Définitif : le nom, le téléphone et le commentaire seront effacés."
                        </p>
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm>
                                "Oui, effacer"
                            </Button>
                            <Button
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Sm
                                attr:r#type="button"
                                on:click=move |_| confirming.set(None)
                            >
                                "Annuler"
                            </Button>
                        </div>
                    </div>
                </ActionForm>
            })
        } else {
            Either::Right(view! {
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    attr:r#type="button"
                    on:click=move |_| confirming.set(Some(id.get_value()))
                >
                    "Effacer"
                </Button>
            })
        }
    }
}

/// Booked session, shared by both tables.
#[component]
fn SessionCell(booking: BookingView) -> impl IntoView {
    view! {
        <TableCell class="align-top">
            <div class="font-medium">{booking.session_date_label.clone()}</div>
            <div class="text-xs text-muted-foreground">
                {booking.service_label.clone()}
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

    const ID: &str = "651d1f0a0000000000000001";
    const OTHER_ID: &str = "651d1f0a0000000000000003";

    fn booking() -> BookingView {
        BookingView {
            id: ID.to_owned(),
            session_id: "651d1f0a0000000000000002".to_owned(),
            service_label: "Apéros créatifs (adultes)".to_owned(),
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

    /// The archive, with `confirming` set to whichever row is mid-confirmation --
    /// `None` for the resting state.
    fn render_deleted_confirming(rows: Vec<BookingView>, confirming: Option<&str>) -> String {
        let confirming = confirming.map(str::to_owned);

        Owner::new().with(move || {
            let purge_action = ServerAction::<PurgeBooking>::new();
            let confirming = RwSignal::new(confirming);

            view! {
                <DeletedBookingTable
                    rows=rows
                    purge_action=purge_action
                    confirming=confirming
                />
            }
            .to_html()
        })
    }

    fn render_deleted(rows: Vec<BookingView>) -> String {
        render_deleted_confirming(rows, None)
    }

    /// The same booking once the admin has removed it.
    fn archived() -> BookingView {
        BookingView {
            is_deleted: true,
            deletion_comment: "Annulation par téléphone".to_owned(),
            ..booking()
        }
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

    /// An archived booking still holds a name, a phone number and a comment. The
    /// archive is the only place they can be made to stop existing, so it has to
    /// offer the way.
    #[test]
    fn an_archived_booking_can_be_erased_for_good() {
        let html = render_deleted(vec![archived()]);

        assert!(html.contains("Effacer"), "no way to erase: {html}");
    }

    /// One click should not erase a visitor's record. The first asks, the second
    /// does it -- and only the second posts anything.
    #[test]
    fn erasing_takes_a_second_click() {
        let resting = render_deleted(vec![archived()]);

        assert!(
            !resting.contains("Oui, effacer"),
            "the confirmation should not be there before it is asked for: {resting}"
        );
        assert!(
            !resting.contains("<form"),
            "nor anything that posts: {resting}"
        );

        let asked = render_deleted_confirming(vec![archived()], Some(ID));

        assert!(asked.contains("Oui, effacer"), "no confirmation: {asked}");
        assert!(asked.contains("Annuler"), "no way back: {asked}");
        assert!(asked.contains(ID), "the form posts no id: {asked}");
    }

    /// Confirming one row must not arm every other: the archive is a list, and two
    /// rows away from the one that was clicked is exactly where a mis-click lands.
    #[test]
    fn only_the_row_being_confirmed_is_armed() {
        let other = BookingView { id: OTHER_ID.to_owned(), ..archived() };

        let html = render_deleted_confirming(vec![archived(), other], Some(ID));

        assert_eq!(
            html.matches("Oui, effacer").count(),
            1,
            "exactly one row should be armed: {html}"
        );
    }

    /// Erasing cannot be taken back, and the row says so before it is confirmed
    /// rather than after.
    #[test]
    fn the_confirmation_says_what_will_be_lost() {
        let html = render_deleted_confirming(vec![archived()], Some(ID));

        assert!(html.contains("Définitif"), "nothing says it is final: {html}");
        assert!(html.contains("téléphone"), "nor what goes with it: {html}");
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
