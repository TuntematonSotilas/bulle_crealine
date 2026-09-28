use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::ui::button::{Button, ButtonSize};

#[cfg(feature = "ssr")]
use leptos_actix::ResponseOptions;

/// Catch-all for a URL that matches no route.
///
/// The menu already links to pages that are not built yet, so this reads as "not
/// ready" rather than "wrong address".
#[component]
pub fn NotFound() -> impl IntoView {
    // The status stays 404 whatever the wording says. A crawler, a link checker
    // or a cache reads the status, not the page, and the resource really is
    // absent; answering 200 would have these treat the placeholder as the real
    // page and index it.
    //
    // Only settable during server-side rendering: navigating here afterwards
    // makes no new request for the server to answer.
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <Title text="Bientôt disponible — Bulle Créaline (E.I)"/>

        <div class="flex flex-col gap-4 items-center py-16 mx-auto max-w-2xl text-center">
            <h1 class="text-4xl font-semibold tracking-tight text-heading">
                "Bientôt disponible"
            </h1>
            <span class="block w-10 h-1 rounded-full bg-heading-soft"></span>
            <p class="text-lg leading-8 text-muted-foreground">
                "Cette page est en cours de préparation. Elle arrive bientôt."
            </p>
            <Button size=ButtonSize::Pill href="/">
                "Revenir à l'accueil"
            </Button>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// The wording softened, but the answer is still a 404. A page that says
    /// "coming soon" over a 200 would be indexed as though it were the real one.
    #[test]
    fn says_it_is_coming_while_still_answering_404() {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        let response = ResponseOptions::default();

        let html = Owner::new().with({
            let response = response.clone();
            move || {
                provide_context(response);
                provide_context(RequestUrl::new("/moi-et-mon-atelier/mon-atelier"));

                // The button is a link, and a link resolves `aria-current` against
                // the location, which server-side is the request being rendered.
                view! {
                    <Router>
                        <NotFound/>
                    </Router>
                }
                .to_html()
            }
        });

        assert!(html.contains("Bientôt disponible"), "wrong wording: {html}");
        assert!(html.contains(r#"href="/""#), "no way back to the home page: {html}");

        // No getter on `ResponseOptions`, so the status is read off the parts it
        // wraps.
        let status = response.0.read().expect("the response parts should be readable").status;

        assert_eq!(
            status,
            Some(actix_web::http::StatusCode::NOT_FOUND),
            "the status should stay 404 whatever the page says"
        );
    }
}
