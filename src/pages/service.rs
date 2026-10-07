use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::api::service_photos::service_photos;
use crate::api::services::all_services;
use crate::api::sessions::upcoming_offer;
use crate::components::blocks::service_block::ServiceBlock;
use crate::components::blocks::service_gallery::ServiceGallery;
use crate::components::blocks::upcoming_themes::{Showing, UpcomingThemes};
use crate::components::seo::{PageMeta, clamp_description};
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

    // Blocking, and not an ordinary resource: the page's title, description and
    // canonical are built from the workshop's name, and `leptos_meta` only injects
    // into the `<head>` what was rendered in the first chunk of the stream. An
    // ordinary resource resolves after that chunk has gone out, and every workshop
    // page shipped the site's generic title -- and, for a slug naming no workshop,
    // a 200 rather than the 404 the branch below sets.
    let services = Resource::new_blocking(|| (), |()| async move { all_services().await });

    // Created here, beside the one above, rather than inside the `Suspend` below:
    // a resource born while its parent suspense is already resolving misses the
    // server's render altogether, and its section never reaches the page.
    let offer = Resource::new(slug, |slug| async move { upcoming_offer(slug).await });

    // The gallery of a workshop run for a structure, and the counterpart of the
    // schedule above: one section or the other, never both. Created here for the
    // same reason, and empty for a bookable workshop -- the server decides that,
    // not this page.
    let photos = Resource::new(slug, |slug| async move { service_photos(slug).await });

    // Read once: it decides where the page opens, and from then on the visitor's
    // clicks do.
    let showing = RwSignal::new(opening_state(&use_query_map().get_untracked()));

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
                            let description = clamp_description(&service.description, 160);
                            // `page_path()` rather than the URL that was asked for:
                            // one workshop answers under both prefixes, and the
                            // canonical has to name one of them. Arriving by
                            // `/pro/<slug>` now points the crawler at the section
                            // the workshop actually belongs to.
                            let path = service.page_path();

                            Either::Right(
                                view! {
                                    <PageMeta title=title description=description path=path/>
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

/// Where the page opens, given the query it was reached with.
///
/// `?theme=<id>` is how a "voir +" on the home page hands over the card that was
/// clicked: a card is one workshop on one theme and carries several of its dates, so
/// the page has to unfold that theme rather than single out one date.
///
/// `?session=<id>` is the older form, kept because links shared while a card was one
/// date should go on opening where they said. It wins when both are given, being the
/// more precise of the two.
///
/// A free function rather than a few lines inside the component so that the
/// precedence is pinned by a test without rendering a page around it.
fn opening_state(query: &leptos_router::params::ParamsMap) -> Showing {
    query
        .get("session")
        .map(Showing::Session)
        .or_else(|| query.get("theme").map(Showing::Theme))
        .unwrap_or(Showing::Themes)
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use leptos_router::params::ParamsMap;

    fn query(pairs: &[(&str, &str)]) -> ParamsMap {
        let mut map = ParamsMap::new();
        for (key, value) in pairs {
            map.insert((*key).to_owned(), (*value).to_owned());
        }

        map
    }

    #[test]
    fn a_bare_url_opens_on_the_grid_of_themes() {
        assert_eq!(opening_state(&query(&[])), Showing::Themes);
    }

    /// What a "voir +" on a home card now hands over.
    #[test]
    fn a_theme_opens_unfolded() {
        assert_eq!(
            opening_state(&query(&[("theme", "t1")])),
            Showing::Theme("t1".to_owned())
        );
    }

    /// Links shared before the home page merged its cards name a session, and have
    /// to go on opening on it.
    #[test]
    fn a_session_still_opens_on_its_own_date() {
        assert_eq!(
            opening_state(&query(&[("session", "s1")])),
            Showing::Session("s1".to_owned())
        );
    }

    /// The more precise of the two wins, so adding the newer parameter to an older
    /// link could not quietly change where it lands.
    #[test]
    fn a_session_outranks_the_theme_it_belongs_to() {
        assert_eq!(
            opening_state(&query(&[("theme", "t1"), ("session", "s1")])),
            Showing::Session("s1".to_owned())
        );
    }
}
