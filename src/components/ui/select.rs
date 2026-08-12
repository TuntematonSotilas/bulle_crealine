//! A dropdown built from ordinary elements rather than a native `<select>`.
//!
//! Firefox paints a native `<select>` popup with the operating system's palette,
//! which shows up white over the dark theme however the element itself is styled.
//! Owning the panel is the only way to theme it, so the list lives in a `<div>`
//! and a hidden input carries the selection into the surrounding form.

use icons::{Check, ChevronDown, ChevronUp};
use leptos::context::Provider;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::tw_merge;

use crate::components::hooks::use_can_scroll_vertical::use_can_scroll_vertical;
use crate::components::hooks::use_random::use_random_id_for;

/// Where the panel opens relative to its trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectPosition {
    #[default]
    Below,
    Above,
}

impl SelectPosition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Below => "Below",
            Self::Above => "Above",
        }
    }
}

/// Panel height below which the dropdown would be cramped, and flips above the
/// trigger instead. Kept under `max-h-[300px]` on the panel so a list that fits
/// comfortably never flips.
const MIN_PANEL_SPACE_PX: f64 = 200.0;

mod components {
    use super::*;
    clx! {SelectLabel, span, "px-2 py-1.5 text-sm font-medium data-inset:pl-8", "mb-1"}
    clx! {SelectItem, li, "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm no-underline transition-colors duration-200 text-popover-foreground hover:bg-accent hover:text-accent-foreground [&_svg:not([class*='size-'])]:size-4"}
}

pub use components::*;

/* ========================================================== */
/*                         CONTEXT                             */
/* ========================================================== */

#[derive(Clone, Copy)]
struct SelectContext {
    /// Id of the panel, so the trigger can point `aria-controls` at it.
    content_id: StoredValue<String>,
    /// Id of the trigger, filled in by [`SelectTrigger`]. Empty when it has none.
    trigger_id: StoredValue<String>,
    open: RwSignal<bool>,
    position: RwSignal<SelectPosition>,
    /// What the hidden input posts: the chosen option's `value`.
    value: RwSignal<Option<String>>,
    /// What the trigger shows, which is usually not the posted value.
    label: RwSignal<Option<String>>,
    on_change: Option<Callback<Option<String>>>,
}

/* ========================================================== */
/*                     ✨ FUNCTIONS ✨                        */
/* ========================================================== */

#[component]
pub fn Select(
    children: Children,
    #[prop(optional, into)] class: String,

    /// Posts the selection under this name with the surrounding form. Without it
    /// the dropdown is display-only and `on_change` is the way to read it.
    #[prop(optional, into)]
    name: Option<String>,

    #[prop(optional, into)] default_value: Option<String>,
    /// Label for `default_value`. Falls back to the value itself.
    #[prop(optional, into)]
    default_label: Option<String>,

    #[prop(optional)] on_change: Option<Callback<Option<String>>>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let position = RwSignal::new(SelectPosition::default());
    let value = RwSignal::new(default_value.clone());
    let label = RwSignal::new(default_label.or_else(|| default_value.clone()));

    let trigger_id = StoredValue::new(String::new());
    let ctx = SelectContext {
        content_id: StoredValue::new(use_random_id_for("select")),
        trigger_id,
        open,
        position,
        value,
        label,
        on_change,
    };

    let wrapper: NodeRef<html::Div> = NodeRef::new();

    // Dismissal is document-wide: a click anywhere outside, or Escape from
    // wherever focus happens to sit. Registered in an effect so it only ever runs
    // in the browser, and torn down with the component.
    Effect::new(move |_| {
        let click = window_event_listener(ev::click, move |ev| {
            if !open.get_untracked() {
                return;
            }
            let target = event_target::<web_sys::Element>(&ev);
            let inside =
                wrapper.get_untracked().is_some_and(|node| node.contains(Some(&target)));

            // A `<label for>` aimed at the trigger forwards its click to it once
            // this handler has run, so closing here would be undone a moment later
            // and the panel would look stuck open.
            let labels_trigger = target.tag_name() == "LABEL"
                && target.get_attribute("for").is_some_and(|for_id| {
                    !for_id.is_empty() && for_id == trigger_id.get_value()
                });

            if !inside && !labels_trigger {
                open.set(false);
            }
        });
        let keydown = window_event_listener(ev::keydown, move |ev| {
            if open.get_untracked() && ev.key() == "Escape" {
                ev.prevent_default();
                open.set(false);
            }
        });

        on_cleanup(move || {
            click.remove();
            keydown.remove();
        });
    });

    let merged_class = tw_merge!("relative w-fit min-w-0", class);

    view! {
        <Provider value=ctx>
            <div data-name="Select" class=merged_class node_ref=wrapper>
                {children()}
                {name.map(|name| {
                    // `prop:value` is what FormData reads, so the posted value follows
                    // the signal; the plain attribute covers the server-rendered markup
                    // before hydration.
                    view! {
                        <input
                            type="hidden"
                            name=name
                            value=move || value.get().unwrap_or_default()
                            prop:value=move || value.get().unwrap_or_default()
                        />
                    }
                })}
            </div>
        </Provider>
    }
}

#[component]
pub fn SelectTrigger(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let trigger: NodeRef<html::Button> = NodeRef::new();

    ctx.trigger_id.set_value(id.clone());

    let button_class = tw_merge!(
        // `group` so the chevron below can react to this button's own `data-state`.
        "group w-full p-2 h-9 inline-flex items-center justify-between text-sm font-medium whitespace-nowrap rounded-md transition-colors focus:outline-none focus:ring-1 focus:ring-ring focus-visible:outline-hidden focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not(:last-child)]:mr-2 [&_svg:not(:first-child)]:ml-2 [&_svg:not([class*='size-'])]:size-4 border bg-background border-input hover:bg-accent hover:text-accent-foreground",
        class
    );

    let toggle = move |ev: ev::MouseEvent| {
        // Without this the click reaches the window listener that closes the panel,
        // and opening would immediately undo itself.
        ev.stop_propagation();

        let opening = !ctx.open.get_untracked();
        if opening {
            if let Some(node) = trigger.get_untracked() {
                let rect = node.get_bounding_client_rect();
                let viewport =
                    window().inner_height().ok().and_then(|h| h.as_f64()).unwrap_or_default();
                let space_below = viewport - rect.bottom();

                ctx.position.set(
                    if space_below < MIN_PANEL_SPACE_PX && rect.top() > space_below {
                        SelectPosition::Above
                    } else {
                        SelectPosition::Below
                    },
                );
            }
        }
        ctx.open.set(opening);
    };

    view! {
        <button
            type="button"
            data-name="SelectTrigger"
            class=button_class
            id=id
            node_ref=trigger
            aria-haspopup="listbox"
            aria-controls=ctx.content_id.get_value()
            aria-expanded=move || ctx.open.get().to_string()
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            on:click=toggle
        >
            {children()}
            <ChevronDown class="text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180" />
        </button>
    }
}

#[component]
pub fn SelectContent(
    children: Children,
    #[prop(optional, into)] class: String,
    /// Preferred side. Overridden when there is not enough room there.
    #[prop(default = SelectPosition::default())]
    position: SelectPosition,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    ctx.position.set(position);

    let merged_class = tw_merge!(
        "w-full overflow-auto z-50 p-1 rounded-md border bg-popover text-popover-foreground shadow-md h-fit max-h-[300px] absolute top-[calc(100%+4px)] left-0 data-[position=Above]:top-auto data-[position=Above]:bottom-[calc(100%+4px)] transition-all duration-200 data-[state=closed]:pointer-events-none data-[state=closed]:opacity-0 data-[state=closed]:scale-95 data-[state=open]:opacity-100 data-[state=open]:scale-100 data-[position=Below]:origin-top data-[position=Above]:origin-bottom [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
        class
    );

    let content: NodeRef<html::Div> = NodeRef::new();
    let scroll = use_can_scroll_vertical();

    // Opening changes what is visible without firing a scroll event, so the
    // affordances have to be recomputed by hand.
    Effect::new(move |_| {
        if ctx.open.get() {
            if let Some(node) = content.get() {
                (scroll.refresh)(&node);
            }
        }
    });

    view! {
        <div
            data-name="SelectContent"
            class=merged_class
            id=ctx.content_id.get_value()
            node_ref=content
            data-state=move || if ctx.open.get() { "open" } else { "closed" }
            data-position=move || ctx.position.get().as_str()
            on:scroll=scroll.on_scroll
        >
            <div
                data-scroll-up="true"
                class=move || {
                    if scroll.can_scroll_up.get() {
                        "sticky -top-1 z-10 flex items-center justify-center py-1 bg-popover"
                    } else {
                        "hidden"
                    }
                }
            >
                <ChevronUp class="size-4 text-muted-foreground" />
            </div>
            {children()}
            <div
                data-scroll-down="true"
                class=move || {
                    if scroll.can_scroll_down.get() {
                        "sticky -bottom-1 z-10 flex items-center justify-center py-1 bg-popover"
                    } else {
                        "hidden"
                    }
                }
            >
                <ChevronDown class="size-4 text-muted-foreground" />
            </div>
        </div>
    }
}

#[component]
pub fn SelectGroup(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = "Select options".into(), into)] aria_label: String,
) -> impl IntoView {
    let merged_class = tw_merge!("group", class);

    view! {
        <ul data-name="SelectGroup" role="listbox" aria-label=aria_label class=merged_class>
            {children()}
        </ul>
    }
}

/// One entry in the list.
///
/// Must be built inside its [`Select`], which is where the context it reads comes
/// from. Collecting options into a `let` ahead of the view runs this function too
/// early and panics.
#[component]
pub fn SelectOption(
    children: Children,
    #[prop(optional, into)] class: String,

    /// What the form posts for this option.
    #[prop(into)]
    value: String,
    /// What the trigger shows once this option is picked. Falls back to `value`.
    #[prop(optional, into)]
    label: Option<String>,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();

    let merged_class = tw_merge!(
        "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm cursor-pointer no-underline transition-colors duration-200 text-popover-foreground hover:bg-accent hover:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
        class
    );

    let is_selected = {
        let value = value.clone();
        move || ctx.value.get().is_some_and(|current| current == value)
    };

    let select = {
        let value = value.clone();
        let label = label.clone();
        move |_| {
            ctx.value.set(Some(value.clone()));
            ctx.label.set(Some(label.clone().unwrap_or_else(|| value.clone())));
            ctx.open.set(false);

            if let Some(on_change) = ctx.on_change {
                on_change.run(Some(value.clone()));
            }
        }
    };

    // Options sit outside the tab order: the trigger is the focusable control, and
    // Enter on a focused option is handled the same as a click.
    let keydown = {
        let select = select.clone();
        move |ev: ev::KeyboardEvent| {
            if ev.key() == "Enter" || ev.key() == " " {
                ev.prevent_default();
                select(());
            }
        }
    };

    view! {
        <li
            data-name="SelectOption"
            class=merged_class
            role="option"
            tabindex="0"
            aria-selected=move || is_selected().to_string()
            data-select-option="true"
            on:click=move |_| select(())
            on:keydown=keydown
        >
            {children()}
            <Check class="ml-auto opacity-0 size-4 text-muted-foreground group-aria-selected:opacity-100" />
        </li>
    }
}

#[component]
pub fn SelectValue(#[prop(optional, into)] placeholder: String) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();

    view! {
        <span
            data-name="SelectValue"
            class=move || {
                if ctx.label.get().is_some() {
                    "min-w-0 text-sm text-foreground truncate"
                } else {
                    "min-w-0 text-sm text-muted-foreground truncate"
                }
            }
        >
            {move || ctx.label.get().unwrap_or_else(|| placeholder.clone())}
        </span>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// The options read the context their `Select` provides, so a caller that builds
    /// them too early panics rather than failing to compile. Rendering the whole tree
    /// is what catches that.
    #[test]
    fn renders_options_and_the_posted_value() {
        let html = Owner::new().with(|| {
            view! {
                <Select name=String::from("service") default_value=String::from("apple")>
                    <SelectTrigger id="service">
                        <SelectValue placeholder="Pick one"/>
                    </SelectTrigger>
                    <SelectContent>
                        <SelectGroup>
                            <SelectOption value="apple" label=String::from("Apple")>
                                "Apple"
                            </SelectOption>
                        </SelectGroup>
                    </SelectContent>
                </Select>
            }
            .to_html()
        });

        assert!(html.contains("Apple"), "the option should render: {html}");
        assert!(html.contains(r#"name="service""#), "the value should post: {html}");
    }
}
