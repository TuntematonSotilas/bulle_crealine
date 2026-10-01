use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::api::service_photos::service_photos;
use crate::api::services::all_services;
use crate::api::sessions::upcoming_offer;
use crate::components::blocks::service_block::ServiceBlock;
use crate::components::blocks::service_gallery::ServiceGallery;
use crate::components::blocks::upcoming_themes::{Showing, UpcomingThemes};
use crate::models::group_by_theme;
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

    // Created here, beside the one above, rather than inside the `Suspend` below:
    // a resource born while its parent suspense is already resolving misses the
    // server's render altogether, and its section never reaches the page.
    let offer = Resource::new(slug, |slug| async move { upcoming_offer(slug).await });

    // The gallery of a workshop run for a structure, and the counterpart of the
    // schedule above: one section or the other, never both. Created here for the
    // same reason, and empty for a bookable workshop -- the server decides that,
    // not this page.
    let photos = Resource::new(slug, |slug| async move { service_photos(slug).await });

    // `?session=<id>` is how a "voir +" on the home page hands over the date that
    // was clicked. Read once: it decides where the page opens, and from then on
    // the visitor's clicks do.
    let showing = use_query_map()
        .get_untracked()
        .get("session")
        .map_or(Showing::Themes, Showing::Session);
    let showing = RwSignal::new(showing);

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
                            let title = format!("{} — Bulle Créaline (E.I)", service.label);

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

        // A sibling of the block above rather than a child: an unknown slug gives
        // no sessions either, so this simply draws nothing beneath the 404.
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                // A storage failure leaves the workshop's presentation standing
                // rather than putting an error under it.
                // `None` is a workshop run for a structure, or an unknown slug:
                // neither has a schedule to be empty. Only a bookable workshop
                // reaches the section, which is what lets it say "nothing yet"
                // without claiming that of a workshop that never has dates.
                offer
                    .await
                    .ok()
                    .flatten()
                    .map(|offer| {
                        let groups = group_by_theme(offer.sessions);

                        view! { <UpcomingThemes groups=groups showing=showing/> }
                    })
            })}
        </Transition>

        // The third sibling, on the same terms as the second: an unknown slug and a
        // bookable workshop both come back with no photo, and the gallery draws
        // nothing on an empty list -- so this needs no condition of its own.
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                photos
                    .await
                    .ok()
                    .map(|photos| view! { <ServiceGallery photos=photos/> })
            })}
        </Transition>
    }
}
