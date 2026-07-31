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
            <h3 class="home-hero-title">"Bulle Créaline"</h3>
            <h1 class="home-hero-subtitle">"Médiation artistique en relation d'aide."</h1>
            <h2 class="home-hero-words">
                <span class="home-hero-word">"Créer"</span>
                <span class="home-hero-word">" 🎨 "</span>
                <span class="home-hero-word">"Se ressourcer"</span>
                <span class="home-hero-word">" 💡 "</span>
                <span class="home-hero-word">"Partager"</span>
            </h2>
        </div>

        <MarqueeWrapper class="p-1 h-32 mt-8">
            <Marquee>
                {reviews_view}
            </Marquee>
        </MarqueeWrapper>
        
    }
}