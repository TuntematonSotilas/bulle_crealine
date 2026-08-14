//! Ctrl+Alt+G, on the admin dashboard, sends a small ghost pacing along the
//! navigation bar. Press again to send it away.
//!
//! Everything it needs lives in this file, stylesheet included: nothing is added to
//! the page until the shortcut is pressed.

use leptos::ev;
use leptos::prelude::*;

/// The two frames of `assets/gp.png`, side by side, and the walk they make up.
///
/// `steps(2)` on a `background-position` running to `-64px` lands on exactly two
/// values, `0` and `-32px`, which is one frame each.
///
/// The ghost only ever walks right to left, which is the way it is drawn facing, so
/// nothing has to flip it. It fades out as it reaches the far end and back in where
/// it started, which is what hides the jump of a one-way loop; walking off the edges
/// instead would have reached past the navigation and could have given a narrow page
/// a horizontal scrollbar.
///
/// It starts one frame short of the right edge, so it never sticks out either.
const STYLE: &str = r#"
@keyframes gp-patrol {
    0%   { left: calc(100% - 32px); opacity: 0; }
    6%   { opacity: 1; }
    94%  { opacity: 1; }
    100% { left: 0;                 opacity: 0; }
}

@keyframes gp-step {
    from { background-position: 0 0; }
    to   { background-position: -64px 0; }
}

.gp-sprite {
    position: absolute;
    /* Feet on the navigation's bottom border, walking in front of the links. */
    bottom: 2px;
    z-index: 10;
    width: 32px;
    height: 32px;
    background-image: url("/assets/gp.png");
    background-size: 64px 32px;
    background-repeat: no-repeat;
    /* Pixel art: never let the browser smooth it. */
    image-rendering: pixelated;
    /* Decorative, and it walks over the links: it must not swallow their clicks. */
    pointer-events: none;
    /* A stroll: one crossing every 12s, two frames per 0.7s to match that pace. */
    animation: gp-patrol 12s linear infinite, gp-step 0.7s steps(2) infinite;
}
"#;

/// The ghost, and the shortcut that summons it.
///
/// Expects a positioned ancestor to pace along — the navigation is `relative` for
/// this reason.
#[component]
pub fn NavSprite() -> impl IntoView {
    let visible = RwSignal::new(false);

    // Registered in an effect so it only ever runs in the browser, and is torn down
    // with the page.
    Effect::new(move |_| {
        let keydown = window_event_listener(ev::keydown, move |ev| {
            // `code` rather than `key`: holding Alt makes some keyboard layouts report
            // a different character for the same physical key.
            if ev.ctrl_key() && ev.alt_key() && ev.code() == "KeyG" {
                ev.prevent_default();
                visible.update(|shown| *shown = !*shown);
            }
        });

        on_cleanup(move || keydown.remove());
    });

    view! {
        <Show when=move || visible.get()>
            <style>{STYLE}</style>
            <div data-name="NavSprite" class="gp-sprite" aria-hidden="true"></div>
        </Show>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// Nothing reaches the page before the shortcut is pressed — stylesheet included.
    #[test]
    fn stays_out_of_the_way_until_summoned() {
        let html = Owner::new().with(|| view! { <NavSprite/> }.to_html());

        assert!(!html.contains("gp-sprite"), "no sprite yet: {html}");
        assert!(!html.contains("gp-patrol"), "and no stylesheet yet: {html}");
    }

    /// The frames and the walk have to agree on the sheet's geometry: two 32px frames
    /// side by side in a 64px-wide image.
    #[test]
    fn the_stylesheet_matches_the_sprite_sheet() {
        assert!(STYLE.contains("background-size: 64px 32px"), "the whole sheet is 64x32");
        assert!(STYLE.contains("to   { background-position: -64px 0; }"), "it scrolls one sheet");
        assert!(STYLE.contains("steps(2)"), "over two frames");
        assert!(STYLE.contains("width: 32px"), "showing one frame at a time");
        assert!(STYLE.contains("calc(100% - 32px)"), "and stopping one frame short of the end");
    }

    /// One-way only: nothing may mirror the sprite, since it never turns around, and
    /// the walk must stay inside the navigation.
    #[test]
    fn the_ghost_only_ever_walks_one_way() {
        assert!(!STYLE.contains("scaleX"), "nothing should flip it: {STYLE}");
        assert!(!STYLE.contains("alternate"), "and it should not come back: {STYLE}");
        assert!(!STYLE.contains("left: -"), "it should not start off the left edge: {STYLE}");
        assert!(!STYLE.contains("left: 100%"), "nor end past the right one: {STYLE}");
    }
}
