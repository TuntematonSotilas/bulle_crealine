use icons::{Minus, Plus};
use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

/// A number field framed by a minus and a plus button.
///
/// The native spinner is hidden: its arrows are tiny, they sit oddly against the
/// rest of the design, and they are close to unusable on a touch screen. The two
/// buttons replace them with real targets.
///
/// The `<input>` underneath is a plain form control, so the field posts with the
/// form and, without the WASM bundle, still accepts a typed value — only the two
/// buttons go quiet.
#[component]
pub fn NumberField(
    // Styling
    #[prop(into, optional)] class: String,

    // Common HTML attributes
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(optional)] required: bool,

    /// Smallest value the buttons will go down to.
    #[prop(default = 0.0)]
    min: f64,
    /// Largest value the buttons will go up to; unbounded when absent.
    #[prop(optional)]
    max: Option<f64>,
    /// How much one press adds or removes.
    #[prop(default = 1.0)]
    step: f64,
    /// Value the field starts on.
    #[prop(optional)]
    value: Option<f64>,
) -> impl IntoView {
    let field = NodeRef::<html::Input>::new();

    // Moves the value by one step and writes it back, clamped. Reading the number
    // off the element rather than a signal keeps the browser as the single owner
    // of the value, which is what plain form submission expects.
    let nudge = move |direction: f64| {
        let Some(input) = field.get_untracked() else {
            return;
        };

        let current = input.value_as_number();
        let current = if current.is_nan() { min } else { current };

        let moved = current + direction * step;
        // Snap back onto the grid `min` defines: repeatedly adding a step such as
        // 0.1 would otherwise drift into values like 65.30000000000001.
        let snapped = min + ((moved - min) / step).round() * step;
        let clamped = snapped.max(min).min(max.unwrap_or(f64::INFINITY));

        input.set_value_as_number(clamped);
    };

    let button_class = "flex items-center justify-center px-3 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-inset disabled:pointer-events-none disabled:opacity-50";

    let frame_class = tw_merge!(
        "flex h-9 w-full min-w-0 items-stretch overflow-hidden rounded-md border border-input bg-transparent shadow-xs transition-[color,box-shadow] dark:bg-input/30",
        "focus-within:border-ring focus-within:ring-ring/50 focus-within:ring-2",
        "has-[:disabled]:pointer-events-none has-[:disabled]:opacity-50",
        class
    );

    view! {
        <div data-name="NumberField" class=frame_class>

            <button
                type="button"
                aria-label="Diminuer"
                class=button_class
                on:click=move |_| nudge(-1.0)
            >
                <Minus class="size-4"/>
            </button>

            <input
                data-name="NumberFieldInput"
                type="number"
                class="w-full min-w-0 bg-transparent text-center text-base outline-none md:text-sm [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
                name=name
                id=id
                required=required
                min=format_number(min)
                max=max.map(format_number)
                step=format_number(step)
                value=value.map(format_number)
                node_ref=field
            />

            <button
                type="button"
                aria-label="Augmenter"
                class=button_class
                on:click=move |_| nudge(1.0)
            >
                <Plus class="size-4"/>
            </button>

        </div>
    }
}

/// Renders a number for an HTML attribute, without a pointless `.0`.
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The field has to stay a plain form control: the value is posted by the
    /// browser, so a missing `name` or `value` would silently break submission.
    #[test]
    fn renders_a_submittable_input() {
        let owner = Owner::new();
        owner.set();

        let html = view! {
            <NumberField name="persons" id="persons" min=1.0 max=20.0 value=1.0 required=true/>
        }
        .to_html();

        assert!(html.contains(r#"type="number""#), "{html}");
        assert!(html.contains(r#"name="persons""#), "{html}");
        assert!(html.contains(r#"min="1""#), "{html}");
        assert!(html.contains(r#"max="20""#), "{html}");
        assert!(html.contains(r#"value="1""#), "{html}");
        assert!(html.contains("required"), "{html}");
    }

    /// Both buttons must opt out of submitting, or pressing one would send the
    /// surrounding form instead of changing the number.
    #[test]
    fn renders_two_non_submitting_buttons() {
        let owner = Owner::new();
        owner.set();

        let html = view! { <NumberField name="persons" min=1.0/> }.to_html();

        assert_eq!(html.matches(r#"type="button""#).count(), 2, "{html}");
        assert!(html.contains(r#"aria-label="Diminuer""#), "{html}");
        assert!(html.contains(r#"aria-label="Augmenter""#), "{html}");
    }

    /// A fractional step must survive into the attribute, otherwise the browser
    /// rejects a price like 62.5 as off-grid.
    #[test]
    fn keeps_a_fractional_step_in_the_markup() {
        let owner = Owner::new();
        owner.set();

        let html = view! { <NumberField name="price" min=0.0 step=0.5 value=62.5/> }.to_html();

        assert!(html.contains(r#"step="0.5""#), "{html}");
        assert!(html.contains(r#"value="62.5""#), "{html}");
    }

    /// Without a `max`, the attribute must be absent rather than empty.
    #[test]
    fn omits_the_max_attribute_when_unbounded() {
        let owner = Owner::new();
        owner.set();

        let html = view! { <NumberField name="max_persons" min=1.0 value=8.0/> }.to_html();

        assert!(!html.contains("max="), "{html}");
    }

    #[test]
    fn drops_the_decimals_of_a_whole_number() {
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(20.0), "20");
        assert_eq!(format_number(0.0), "0");
    }

    #[test]
    fn keeps_the_decimals_of_a_fractional_number() {
        assert_eq!(format_number(0.5), "0.5");
        assert_eq!(format_number(62.5), "62.5");
    }
}
