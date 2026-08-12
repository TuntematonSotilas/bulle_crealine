//! Calendar for picking a date and a time, in place of `<input type="datetime-local">`.
//!
//! Ported from rust-ui.com's date picker, "with time picker" variant. Three things
//! differ from that demo: its start/end pair of time fields collapses into the single
//! instant a session has, the month and weekday names are French, and the cells take
//! keyboard input — the native input it replaces was reachable without a mouse, and a
//! grid of click-only `<td>` would have lost that.
//!
//! What the form posts stays in the `datetime-local` shape (`2026-07-05T14:00`) that
//! [`crate::db::datetime::parse_input`] already reads, so the server side is unchanged.

use icons::{CalendarDays, ChevronLeft, ChevronRight, Clock2};
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_ui::clx;
use time::{Date, Month, OffsetDateTime};
use tw_merge::tw_merge;

use crate::components::hooks::use_dismiss::use_dismiss;
use crate::components::hooks::use_random::use_random_id_for;
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;

/// Hour proposed for a session being created from scratch, workshops rarely being
/// held at midnight.
const DEFAULT_TIME: &str = "14:00";

/// How tall the open panel is: measured at 417px on a six-row month with the hour row,
/// rounded up. With less room than this below the trigger it opens upwards instead of
/// running off the bottom of the window.
const PANEL_HEIGHT_PX: f64 = 420.0;

const MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// Column headers, paired with the name screen readers announce.
const WEEKDAYS: [(&str, &str); 7] = [
    ("lu", "lundi"),
    ("ma", "mardi"),
    ("me", "mercredi"),
    ("je", "jeudi"),
    ("ve", "vendredi"),
    ("sa", "samedi"),
    ("di", "dimanche"),
];

mod components {
    use super::*;
    clx! {DatePicker, div, "flex flex-col gap-4 p-3 rounded-lg border bg-card text-card-foreground shadow-sm w-fit"}
    clx! {DatePickerNavButton, button, "inline-flex items-center justify-center p-0 text-sm font-medium transition-colors bg-transparent border rounded-md opacity-50 whitespace-nowrap focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border-input hover:bg-accent hover:text-accent-foreground size-7 hover:opacity-100 [&_svg:not([class*='size-'])]:size-4"}
    clx! {DatePickerTitle, span, "text-sm font-medium text-center"}
    clx! {DatePickerHeader, header, "grid grid-cols-[auto_1fr_auto] items-center pt-1"}
    clx! {DatePickerWeekDay, th, "text-muted-foreground rounded-md w-9 font-normal text-[0.8rem]"}
    clx! {DatePickerRow, tr, "flex w-full mt-2"}
}

pub use components::*;

/* ========================================================== */
/*                     ✨ FUNCTIONS ✨                        */
/* ========================================================== */

/// One cell of a month grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatePickerDay {
    pub day: u8,
    /// Belongs to a neighbouring month: shown faded, and not selectable.
    pub disabled: bool,
}

/// The cells to lay out for `month`, padded at both ends with the neighbouring
/// months' days so that every row is a full week.
pub fn calendar_days(year: i32, month: Month) -> Vec<DatePickerDay> {
    let Ok(first_day) = Date::from_calendar_date(year, month, 1) else {
        return vec![];
    };

    // Monday is column one, so Monday contributes no leading days.
    let leading = first_day.weekday().number_from_monday() as usize - 1;

    let previous = month.previous();
    let previous_year = if month == Month::January { year - 1 } else { year };
    let days_in_previous = previous.length(previous_year);

    let mut days = Vec::with_capacity(42);

    for offset in 0..leading {
        let day = days_in_previous - leading as u8 + offset as u8 + 1;
        days.push(DatePickerDay { day, disabled: true });
    }

    for day in 1..=month.length(year) {
        days.push(DatePickerDay { day, disabled: false });
    }

    let trailing = (7 - days.len() % 7) % 7;
    for day in 1..=trailing as u8 {
        days.push(DatePickerDay { day, disabled: true });
    }

    days
}

#[component]
pub fn DatePickerCell(
    day: u8,
    year: i32,
    month: Month,
    disabled: bool,
    /// The picked day, which this cell highlights when it is the one.
    selected: RwSignal<Date>,
    // `Copy` because the cell wires it to both a click and a keydown, and the grid
    // hands the same closure to all 42 cells.
    on_click: impl Fn(u8) + Copy + 'static,
    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let date = if disabled { None } else { Date::from_calendar_date(year, month, day).ok() };

    let is_selected = move || date.is_some_and(|date| date == selected.get());

    let merged_class = tw_merge!(
        "inline-flex items-center justify-center text-sm size-9 rounded-md select-none",
        "hover:cursor-pointer hover:bg-accent",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        "aria-disabled:pointer-events-none aria-disabled:opacity-50 aria-disabled:cursor-not-allowed",
        "aria-current:bg-primary aria-current:hover:bg-primary aria-current:text-primary-foreground",
        class
    );

    let select = move || {
        if !disabled {
            on_click(day);
        }
    };

    // The cells are `<td>`, not buttons, so being focusable and answering Enter or
    // Space is on us. Days from the neighbouring months stay out of the tab order.
    let keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" || ev.key() == " " {
            ev.prevent_default();
            select();
        }
    };

    view! {
        <td
            data-name="DatePickerCell"
            class=merged_class
            role="gridcell"
            tabindex=if disabled { "-1" } else { "0" }
            aria-current=move || is_selected().to_string()
            aria-disabled=move || disabled.to_string()
            on:click=move |_| select()
            on:keydown=keydown
        >
            {day}
        </td>
    }
}

/// A month grid plus an hour, posting the two together under `name`.
#[component]
pub fn DateTimeField(
    /// Posts the instant under this name, shaped like a `datetime-local` value.
    #[prop(into)]
    name: String,

    /// Id given to the button that opens the calendar, which is what the surrounding
    /// `<Label for>` should point at.
    #[prop(into, optional)]
    id: String,

    /// Current value, e.g. `2026-07-05T14:00`. Anything unparseable, empty included,
    /// starts the calendar on today at [`DEFAULT_TIME`].
    #[prop(into, optional)]
    value: String,

    #[prop(into, optional)] class: String,
) -> impl IntoView {
    let (initial_date, initial_time) = parse_value(&value)
        .unwrap_or_else(|| (OffsetDateTime::now_utc().date(), DEFAULT_TIME.to_owned()));

    let selected_date = RwSignal::new(initial_date);
    // Which month the grid shows: the arrows move it without picking anything.
    let display_date = RwSignal::new(initial_date);
    let time = RwSignal::new(initial_time.clone());

    let open = RwSignal::new(false);
    let above = RwSignal::new(false);
    let wrapper: NodeRef<html::Div> = NodeRef::new();
    let trigger: NodeRef<html::Button> = NodeRef::new();
    let panel_id = use_random_id_for("date_picker");

    use_dismiss(open, wrapper, StoredValue::new(id.clone()));

    let toggle = move |ev: ev::MouseEvent| {
        // Without this the click reaches the window listener from `use_dismiss`, and
        // opening would immediately undo itself.
        ev.stop_propagation();

        let opening = !open.get_untracked();
        if opening {
            if let Some(node) = trigger.get_untracked() {
                let rect = node.get_bounding_client_rect();
                let viewport =
                    window().inner_height().ok().and_then(|height| height.as_f64()).unwrap_or_default();
                let space_below = viewport - rect.bottom();

                above.set(space_below < PANEL_HEIGHT_PX && rect.top() > space_below);
            }
        }
        open.set(opening);
    };

    // The panel stays open on a pick, unlike a `Select`: the hour sits in there too, and
    // closing on the day would mean reopening to set it.
    let select_day = move |day: u8| {
        let shown = display_date.get();
        if let Ok(date) = Date::from_calendar_date(shown.year(), shown.month(), day) {
            selected_date.set(date);
        }
    };

    // What the form actually submits. `prop:value` is what FormData reads, so it
    // follows the signals; the plain attribute covers the server-rendered markup
    // before hydration.
    let posted = move || format!("{}T{}", iso_date(selected_date.get()), time.get());

    let merged_class = tw_merge!("relative w-fit min-w-0", class);

    view! {
        <div class=merged_class node_ref=wrapper role="group" aria-label="Date et heure">
            <button
                type="button"
                id=id
                node_ref=trigger
                class="group inline-flex gap-2 items-center px-3 w-full h-9 min-w-0 text-sm font-medium whitespace-nowrap rounded-md border transition-colors bg-background border-input hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                aria-haspopup="dialog"
                aria-controls=panel_id.clone()
                aria-expanded=move || open.get().to_string()
                on:click=toggle
            >
                <CalendarDays class="size-4 shrink-0 text-muted-foreground" />
                <span class="min-w-0 truncate">
                    {move || {
                        let date = selected_date.get();
                        format!(
                            "{} {} {} à {}",
                            date.day(),
                            month_label(date.month()),
                            date.year(),
                            time.get(),
                        )
                    }}
                </span>
            </button>

            // Mounted only while open, so the calendar's focusable cells stay out of
            // the tab order the rest of the time.
            <Show when=move || open.get()>
                <div
                    id=panel_id.clone()
                    role="dialog"
                    aria-label="Choisir une date"
                    class=move || {
                        let side = if above.get() {
                            "bottom-[calc(100%+4px)]"
                        } else {
                            "top-[calc(100%+4px)]"
                        };
                        format!("absolute left-0 z-50 {side}")
                    }
                >
                    <CalendarPanel
                        selected_date=selected_date
                        display_date=display_date
                        time=time
                        on_pick=select_day
                    />
                </div>
            </Show>

            // What the form actually submits: the two controls above, joined.
            <input type="hidden" name=name value=posted prop:value=posted />
        </div>
    }
}

/// The month grid itself, which [`DateTimeField`] mounts while its panel is open.
#[component]
fn CalendarPanel(
    selected_date: RwSignal<Date>,
    display_date: RwSignal<Date>,
    time: RwSignal<String>,
    // `Send + Sync` because the grid is built inside a reactive closure, which has to
    // cross threads when the page is rendered on the server.
    on_pick: impl Fn(u8) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let time_id = use_random_id_for("time");

    // Only a readable hour reaches the signal. This is what keeps the posted instant
    // valid, and it has to: `required` on this input is only enforced while the panel
    // is open, and the panel is closed at submit time.
    let set_time = move |ev: ev::Event| {
        if let Some(valid) = parse_time(&event_target_value(&ev)) {
            time.set(valid);
        }
    };

    let show_previous_month = move |_| {
        let current = display_date.get();
        let year = if current.month() == Month::January { current.year() - 1 } else { current.year() };
        if let Ok(date) = Date::from_calendar_date(year, current.month().previous(), 1) {
            display_date.set(date);
        }
    };

    let show_next_month = move |_| {
        let current = display_date.get();
        let year = if current.month() == Month::December { current.year() + 1 } else { current.year() };
        if let Ok(date) = Date::from_calendar_date(year, current.month().next(), 1) {
            display_date.set(date);
        }
    };

    view! {
        <DatePicker>
                <DatePickerHeader>
                    <DatePickerNavButton
                        class="justify-self-start"
                        attr:r#type="button"
                        attr:aria-label="Mois précédent"
                        on:click=show_previous_month
                    >
                        <ChevronLeft />
                    </DatePickerNavButton>
                    <DatePickerTitle attr:role="presentation">
                        {move || {
                            let shown = display_date.get();
                            format!("{} {}", month_label(shown.month()), shown.year())
                        }}
                    </DatePickerTitle>
                    <DatePickerNavButton
                        class="justify-self-end"
                        attr:r#type="button"
                        attr:aria-label="Mois suivant"
                        on:click=show_next_month
                    >
                        <ChevronRight />
                    </DatePickerNavButton>
                </DatePickerHeader>

                <table class="w-full border-collapse" role="grid">
                    <thead>
                        <tr class="flex">
                            {WEEKDAYS
                                .into_iter()
                                .map(|(short, full)| {
                                    view! {
                                        <DatePickerWeekDay attr:aria-label=full>{short}</DatePickerWeekDay>
                                    }
                                })
                                .collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let shown = display_date.get();
                            let (year, month) = (shown.year(), shown.month());
                            calendar_days(year, month)
                                .chunks(7)
                                .map(|week| {
                                    let cells = week
                                        .iter()
                                        .map(|&DatePickerDay { day, disabled }| {
                                            view! {
                                                <DatePickerCell
                                                    day=day
                                                    year=year
                                                    month=month
                                                    disabled=disabled
                                                    selected=selected_date
                                                    on_click=on_pick
                                                />
                                            }
                                        })
                                        .collect_view();
                                    view! { <DatePickerRow>{cells}</DatePickerRow> }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>

                <div class="flex gap-3 items-center pt-3 border-t">
                    <Label r#for=time_id.clone()>
                        <Clock2 class="size-4 text-muted-foreground" />
                        "Heure"
                    </Label>
                    // Reading the signal rather than a captured initial value: the panel
                    // is built afresh on every open, so this is what carries an hour
                    // already picked back into the field.
                    <Input
                        r#type=InputType::Time
                        id=time_id
                        required=true
                        attr:value=move || time.get()
                        on:input=set_time
                        class="w-auto"
                    />
                </div>
        </DatePicker>
    }
}

fn month_label(month: Month) -> &'static str {
    MONTHS[month as usize - 1]
}

fn iso_date(date: Date) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month() as u8, date.day())
}

/// Splits a `datetime-local` value into the day to select and the `HH:MM` to show.
///
/// Seconds are dropped rather than rejected: the server accepts them, so a stored
/// value that carries them should still open the form.
fn parse_value(value: &str) -> Option<(Date, String)> {
    let (date, time) = value.trim().split_once('T')?;

    let mut parts = date.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = Month::try_from(parts.next()?.parse::<u8>().ok()?).ok()?;
    let day = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some((Date::from_calendar_date(year, month, day).ok()?, parse_time(time)?))
}

/// Normalises an `<input type="time">` value to `HH:MM`.
///
/// `None` for anything the field cannot post, the empty string included — which is what
/// a cleared field reads as.
fn parse_time(value: &str) -> Option<String> {
    let (hours, rest) = value.trim().split_once(':')?;
    let minutes = rest.get(..2)?;

    if hours.len() != 2 || hours.parse::<u8>().ok()? > 23 || minutes.parse::<u8>().ok()? > 59 {
        return None;
    }

    Some(format!("{hours}:{minutes}"))
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// Renders the panel the way [`DateTimeField`] does once opened.
    fn panel_html(value: &str) -> String {
        let (date, time) = parse_value(value).expect("the fixture should parse");
        Owner::new().with(|| {
            let selected = RwSignal::new(date);
            let time = RwSignal::new(time);
            view! {
                <CalendarPanel
                    selected_date=selected
                    display_date=selected
                    time=time
                    on_pick=|_| {}
                />
            }
            .to_html()
        })
    }

    /// The whole point of the field: what the form posts has to stay in the shape the
    /// server parses, and the closed trigger has to read back the stored instant.
    #[test]
    fn posts_the_picked_instant_and_reads_it_back() {
        let html = Owner::new().with(|| {
            view! { <DateTimeField id="date" name="date" value="2026-08-19T14:30"/> }.to_html()
        });

        assert!(
            html.contains(r#"<input type="hidden" name="date" value="2026-08-19T14:30">"#),
            "the instant should post under one name: {html}"
        );
        assert!(
            html.contains("19 août 2026 à 14:30"),
            "the trigger should read the day and the hour: {html}"
        );
    }

    /// The hour lives in the panel, which is rebuilt on every open — so it has to come
    /// from the signal, not from the value the field started with.
    #[test]
    fn the_panel_carries_the_current_hour() {
        assert!(
            panel_html("2026-08-19T14:30").contains(r#"value="14:30""#),
            "the hour should be in the panel"
        );
    }

    /// Every button here sits inside the session form, where one without an explicit
    /// type submits it. The month arrows are the easy ones to get wrong.
    #[test]
    fn no_button_submits_the_form() {
        let field = Owner::new().with(|| view! { <DateTimeField name="date" value=""/> }.to_html());
        assert_buttons_opt_out(&field, 1, "the trigger");
        assert_buttons_opt_out(&panel_html("2026-08-19T14:30"), 2, "both month arrows");
    }

    fn assert_buttons_opt_out(html: &str, expected: usize, what: &str) {
        let buttons = html.matches("<button").count();
        assert_eq!(buttons, expected, "{what} should be the only button(s): {html}");
        assert_eq!(
            html.matches(r#"type="button""#).count(),
            buttons,
            "{what} must opt out of submitting: {html}"
        );
    }

    /// The calendar only exists while the panel is open, which is what keeps its
    /// focusable day cells out of the tab order the rest of the time.
    #[test]
    fn the_panel_starts_closed() {
        let html = Owner::new()
            .with(|| view! { <DateTimeField name="date" value="2026-08-19T14:30"/> }.to_html());

        assert!(!html.contains("DatePickerCell"), "no day cell should exist yet: {html}");
        assert!(html.contains("19 août 2026"), "the trigger still shows the date: {html}");
        assert!(html.contains(r#"aria-expanded="false""#), "and reports itself closed: {html}");
    }

    #[test]
    fn parses_a_stored_value_with_and_without_seconds() {
        let (date, time) = parse_value("2026-07-05T14:00").expect("a plain value parses");
        assert_eq!(iso_date(date), "2026-07-05");
        assert_eq!(time, "14:00");

        let (date, time) = parse_value("2026-07-05T14:00:30").expect("seconds are dropped");
        assert_eq!(iso_date(date), "2026-07-05");
        assert_eq!(time, "14:00");
    }

    /// The guard behind the posted instant staying valid: a cleared field must not reach
    /// the signal, because nothing else checks it once the panel closes.
    #[test]
    fn an_unusable_hour_never_reaches_the_signal() {
        for value in ["", "  ", "1:30", "24:00", "12:60", "12", "abc"] {
            assert!(parse_time(value).is_none(), "{value:?} should not be accepted");
        }

        assert_eq!(parse_time("09:05").as_deref(), Some("09:05"));
        assert_eq!(parse_time("23:59:30").as_deref(), Some("23:59"), "seconds are dropped");
    }

    #[test]
    fn rejects_values_the_calendar_could_not_open_on() {
        for value in ["", "2026-07-05", "2026-13-05T14:00", "2026-07-32T14:00", "2026-07-05T25:00"] {
            assert!(parse_value(value).is_none(), "{value} should not parse");
        }
    }

    /// The grid is laid out in rows of seven, so a month has to be padded at both
    /// ends — and the padding has to carry the neighbouring months' real day numbers.
    #[test]
    fn pads_a_month_to_whole_weeks() {
        // August 2026 starts on a Saturday and has 31 days.
        let days = calendar_days(2026, Month::August);

        assert_eq!(days.len() % 7, 0, "every row must be a full week");
        assert_eq!(
            days.iter().filter(|day| !day.disabled).count(),
            31,
            "every day of the month is present"
        );

        let leading: Vec<u8> = days.iter().take_while(|day| day.disabled).map(|day| day.day).collect();
        assert_eq!(leading, vec![27, 28, 29, 30, 31], "July's tail fills the first row");

        assert_eq!(days[5], DatePickerDay { day: 1, disabled: false }, "the 1st is a Saturday");
    }

    /// A January grid has to reach back into the previous year for its leading days.
    #[test]
    fn pads_january_from_the_previous_december() {
        let days = calendar_days(2027, Month::January);
        let leading: Vec<u8> = days.iter().take_while(|day| day.disabled).map(|day| day.day).collect();

        assert_eq!(leading, vec![28, 29, 30, 31], "December 2026 ends on a Thursday");
    }

    #[test]
    fn month_labels_are_french() {
        assert_eq!(month_label(Month::January), "janvier");
        assert_eq!(month_label(Month::December), "décembre");
    }
}
