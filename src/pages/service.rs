use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::services::all_services;
use crate::components::blocks::service_block::ServiceBlock;
use crate::pages::not_found::NotFound;

/// One workshop's own page, at `/services/<slug>` and `/pro/<slug>`.
///
/// One component for every workshop: they are rows the admin edits, so there is
/// nothing left to write per workshop. The four hand-written pages this replaces
/// had already drifted from the model -- one of them carried a title the model
/// disagreed with, and one had no booking button at all.
///
/// The two routes are served by the same lookup, on the slug alone. A workshop
/// that switches section therefore keeps answering on its old path rather than
/// turning every link to it into a 404.
#[component]
pub fn ServicePage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    let services = Resource::new(|| (), |()| async move { all_services().await });

    view! {
        <Transition fallback=|| {
            view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
        }>
            {move || {
                let slug = slug();

                Suspend::new(async move {
                    // A storage failure reads as "not here" rather than as an error
                    // page: the visitor came for a workshop, and the failure is
                    // already logged server-side.
                    let found = services
                        .await
                        .ok()
                        .and_then(|list| {
                            list.into_iter().find(|service| service.slug == slug)
                        });

                    match found {
                        None => Either::Left(view! { <NotFound/> }),
                        Some(service) => {
                            let title = format!("{} — Bulle Créaline", service.label);

                            Either::Right(
                                view! {
                                    <Title text=title/>
                                    <ServiceBlock service=service/>
                                },
                            )
                        }
                    }
                })
            }}
        </Transition>
    }
}
