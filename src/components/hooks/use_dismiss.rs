use leptos::ev;
use leptos::html;
use leptos::prelude::*;

/// Closes `open` on a click anywhere outside `wrapper`, or on Escape from wherever
/// focus happens to sit.
///
/// `trigger_id` names the control a `<label for>` may point at. Such a label forwards
/// its click to the trigger once this handler has run, so closing on it would be undone
/// a moment later and the panel would look stuck open. Leave it empty when no label
/// points at anything inside.
///
/// The listeners are registered in an effect, so they only ever run in the browser, and
/// are torn down with the calling component.
pub fn use_dismiss(open: RwSignal<bool>, wrapper: NodeRef<html::Div>, trigger_id: StoredValue<String>) {
    Effect::new(move |_| {
        let click = window_event_listener(ev::click, move |ev| {
            if !open.get_untracked() {
                return;
            }

            let target = event_target::<web_sys::Element>(&ev);
            let inside = wrapper.get_untracked().is_some_and(|node| node.contains(Some(&target)));

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
}
