use leptos::prelude::*;

/// A label and its value, one per row.
///
/// A list rather than a table: every row is one short answer, and a table would make
/// a screen reader announce coordinates for a column that never varies.
///
/// Shared by the legal notice and the privacy policy, which is why it lives here
/// rather than in either of them: the two pages were one, and the day their rows stop
/// being set the same way is the day they start looking like two unrelated documents.
#[component]
pub fn Lines(rows: Vec<(&'static str, String)>) -> impl IntoView {
    let items = rows
        .into_iter()
        .map(|(label, value)| {
            view! {
                <div class="p-4 rounded-2xl border bg-surface text-surface-foreground border-border">
                    <dt class="text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                        {label}
                    </dt>
                    <dd class="mt-1 text-sm leading-relaxed">{value}</dd>
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! { <dl class="flex flex-col gap-3">{items}</dl> }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// A definition list, so that a label and its value are tied together rather
    /// than merely sitting next to each other.
    #[test]
    fn each_row_pairs_a_label_with_its_value() {
        let html = Owner::new().with(|| {
            view! {
                <Lines rows=vec![
                    ("SIRET", "123 456 789 00012".to_owned()),
                    ("TVA", "Non applicable".to_owned()),
                ]/>
            }
            .to_html()
        });

        assert_eq!(html.matches("<dt").count(), 2, "not one label per row: {html}");
        assert_eq!(html.matches("<dd").count(), 2, "not one value per row: {html}");
        assert!(html.contains("123 456 789 00012"), "{html}");
    }
}
