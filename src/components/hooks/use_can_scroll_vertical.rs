use leptos::prelude::*;

/// Vertical scroll state of one scrollable element, as tracked by
/// [`use_can_scroll_vertical`].
#[derive(Clone, Copy)]
pub struct CanScrollVertical<OnScroll, Refresh> {
    /// Wire to the element's `on:scroll`.
    pub on_scroll: OnScroll,
    /// Call whenever the element becomes visible or its contents change: no scroll
    /// event fires for either, so the signals would otherwise stay stale until the
    /// user actually scrolls.
    pub refresh: Refresh,
    pub can_scroll_up: RwSignal<bool>,
    pub can_scroll_down: RwSignal<bool>,
}

/// Tracks whether a scrollable element still hides content above or below it, so
/// a caller can show scroll affordances only when they mean something.
pub fn use_can_scroll_vertical()
-> CanScrollVertical<impl Fn(web_sys::Event) + Copy, impl Fn(&web_sys::Element) + Copy> {
    let can_scroll_up = RwSignal::new(false);
    let can_scroll_down = RwSignal::new(false);

    let refresh = move |element: &web_sys::Element| {
        let scroll_top = element.scroll_top();
        let scroll_height = element.scroll_height();
        let client_height = element.client_height();

        can_scroll_up.set(scroll_top > 0);
        // The spare pixel absorbs sub-pixel rounding, which otherwise leaves the
        // "more below" hint showing when the element is already at the bottom.
        can_scroll_down.set(scroll_top < scroll_height - client_height - 1);
    };

    CanScrollVertical {
        on_scroll: move |ev: web_sys::Event| refresh(&event_target::<web_sys::Element>(&ev)),
        refresh,
        can_scroll_up,
        can_scroll_down,
    }
}
