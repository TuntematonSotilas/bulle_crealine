use leptos::prelude::*;
use crate::components::ui::marquee::*;

struct Review {
    pub stars: i32,
    pub name: String,
    pub comment: String,
}

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {

    let reviews = vec![
        Review {
            stars: 3,
            name: "Alice".to_string(),
            comment: "Excellent atelier !".to_string(),
        },
        Review {
            stars: 3,
            name: "Bob".to_string(),
            comment: "Très bonnes séances.".to_string(),
        },
    ];

    let reviews_view = reviews
        .into_iter()
        .map(|review| {
            let stars = (0..review.stars)
                .map(|_| {
                    view! {
                        <img class="w-5 h-5" src="assets/star.svg" alt="Star" />
                    }
                }.into_any())
                .collect::<Vec<_>>();

            view! {
                <MarqueeRow class="p-4 bg-[var(--card)] text-[var(--card-foreground)] rounded-r-lg">
                    <div class="flex flex-col gap-3">
                        <div class="flex items-center gap-3">
                            <span class="font-semibold">{review.name}</span>
                            <div class="flex gap-1">{stars}</div>
                        </div>
                        <p class="text-sm leading-6">{review.comment}</p>
                    </div>
                </MarqueeRow>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="home-hero border border-(--border) rounded-[2rem] shadow-(--shadow) max-w-4xl mx-auto">
            <h2 class="home-hero-title">"Bulle Créaline"</h2>
            <h3>"Ma source de créativité"</h3>
            <h3>"Ateliers créatifs bien-être"</h3>
        </div>

        <MarqueeWrapper class="p-1 h-32 mt-8">
            <Marquee>
                {reviews_view}
            </Marquee>
        </MarqueeWrapper>
        
    }
}