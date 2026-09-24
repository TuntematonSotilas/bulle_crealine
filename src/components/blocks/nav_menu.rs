use icons::{Menu, X};
use leptos::prelude::*;

use crate::api::services::all_services;
use crate::components::ui::service_icon::ServiceIcon;
use crate::components::ui::{navigation_menu::*, theme_toggle::ThemeToggle};
use crate::models::ServiceView;

/// The site's navigation, in its two shapes.
///
/// The workshop entries come from the collection, so adding one in the admin area
/// publishes it in both menus at once. The sections themselves stay written here:
/// they are the menu's structure, not data, and a heading that came and went with
/// its contents would make the bar jump while the list loads.
///
/// One resource for the two menus rather than one each: both are always in the
/// DOM, one hidden by a media query, so fetching twice would double the work on
/// every single page.
#[component]
pub fn NavMenu() -> impl IntoView {
    let services = Resource::new(|| (), |()| async move { all_services().await });

    view! {
        <DesktopNav services=services/>
        <MobileNav services=services/>
    }
}

/// What both menus read. Named so the two components can take it as one prop.
type Services = Resource<Result<Vec<ServiceView>, ServerFnError>>;

/// The workshops of one section, in the order the server sorted them.
fn section(services: &[ServiceView], pro: bool) -> Vec<ServiceView> {
    services
        .iter()
        .filter(|service| service.pro == pro)
        .cloned()
        .collect()
}

/// The bubble mark, as every home link carries it.
///
/// Decorative on purpose: [`HomeWord`] stands beside it and names the link in
/// visible text, so an `alt` repeating "Accueil" would have the destination read
/// out twice.
#[component]
fn HomeLogo() -> impl IntoView {
    view! { <img src="/assets/icon.svg" alt="" aria-hidden="true" class="w-16 h-16"/> }
}

/// The word naming the home link, shown at all times rather than on hover.
///
/// The mark is a bubble with no lettering in it, so without this word nothing
/// tells a visitor where it leads unless they happen to point at it.
///
/// It takes `text-foreground` rather than the `text-foreground/70` the bar's links
/// inherit: this word alone names the link, and does not settle for the 70% its
/// neighbours can afford.
#[component]
fn HomeWord(#[prop(into, optional)] class: String) -> impl IntoView {
    view! { <span class=format!("font-medium text-foreground {class}")>"Accueil"</span> }
}

/// Small wave set between the mobile menu's top-level sections.
///
/// Kept at icon size and centred rather than stretched across the panel, so the
/// shape holds its own proportions and the stroke needs no correction.
#[component]
fn SectionWave() -> impl IntoView {
    view! {
        // `block` because `mx-auto` has nothing to centre on an inline box, which
        // is what an `svg` is by default. The path is inset a unit either side:
        // `stroke-linecap: round` puts half the stroke past each end point, and
        // the viewBox would clip it.
        <svg
            aria-hidden="true"
            viewBox="0 0 26 8"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            class="block mx-auto w-10 h-3 text-heading-soft"
        >
            <path d="M1 4 q 3 -3 6 0 t 6 0 t 6 0 t 6 0"/>
        </svg>
    }
}

/// The hover-driven bar, from `md` up.
#[component]
fn DesktopNav(services: Services) -> impl IntoView {
    view! {
        <div class="hidden justify-center items-center py-8 md:flex">
            <NavigationMenu>
                <NavigationMenuList>
                    <NavigationMenuItem>
                        // `flex` rather than the base `inline-flex`: an inline box
                        // takes its baseline from the image's bottom edge, which
                        // grows the row unevenly and knocks the logo off centre.
                        <NavigationMenuLink
                            href="/"
                            class="flex gap-2 items-center px-3 h-16 rounded-md transition-colors hover:bg-accent"
                        >
                            <HomeLogo/>
                            <HomeWord class="text-sm"/>
                        </NavigationMenuLink>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                        <NavigationMenuTrigger>"Ateliers à domicile"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <div class="w-[320px] p-0">
                                <DesktopServiceLinks services=services pro=false/>
                            </div>
                        </NavigationMenuContent>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                        <NavigationMenuTrigger>"Autres Ateliers"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <div class="w-[320px] p-0">
                                <DesktopServiceLinks services=services pro=true/>
                            </div>
                        </NavigationMenuContent>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                        <NavigationMenuTrigger>"Moi et mon atelier"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <div class="w-[320px] p-0">
                                <ul class="space-y-2">
                                    <li class="p-1">
                                        <a href="/moi-et-mon-atelier/qui-suis-je" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Qui-suis-je?"</div>
                                        </a>
                                    </li>
                                    <li class="p-1">
                                        <a href="/moi-et-mon-atelier/mon-atelier" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Mon atelier"</div>
                                        </a>
                                    </li>
                                    <li class="p-1">
                                        <a href="/moi-et-mon-atelier/photos" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Photos des ateliers"</div>
                                        </a>
                                    </li>
                                    <li class="p-1">
                                        <a href="/moi-et-mon-atelier/catalogue" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Catalogue des ateliers"</div>
                                        </a>
                                    </li>
                                    <li class="p-1">
                                        <a href="/moi-et-mon-atelier/diplomes-et-formations" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Diplômes et formations"</div>
                                        </a>
                                    </li>
                                </ul>
                            </div>
                        </NavigationMenuContent>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                        <NavigationMenuLink href="/newsletter" class=navigation_menu_trigger_style()>
                            "Newsletter"
                        </NavigationMenuLink>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                        <ThemeToggle/>
                    </NavigationMenuItem>
                </NavigationMenuList>
            </NavigationMenu>
        </div>
    }
}

/// The workshops of one section, once they have loaded, in the bar's dropdown.
#[component]
fn DesktopServiceLinks(services: Services, pro: bool) -> impl IntoView {
    view! {
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                // A storage failure leaves the dropdown empty rather than putting an
                // error in the navigation of every page.
                let list = services.await.map(|list| section(&list, pro)).unwrap_or_default();

                view! { <DesktopServiceList services=list/> }
            })}
        </Transition>
    }
}

/// The list itself, split from the fetch above so it can be rendered from a plain
/// `Vec`: what a `Transition` wraps never resolves under a synchronous `to_html`,
/// which would leave these links untested.
#[component]
fn DesktopServiceList(services: Vec<ServiceView>) -> impl IntoView {
    let links = services
        .into_iter()
        .map(|service| {
            // Both read before `label` is moved into the view.
            let href = service.page_path();
            let icon = service.icon;

            view! {
                <li class="p-1">
                    <a
                        href=href
                        class="flex gap-3 items-center p-3 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                    >
                        // Renders nothing when the workshop has no picto, so the
                        // label simply moves left rather than sitting behind a gap.
                        <ServiceIcon name=icon class="w-4 h-4 shrink-0 text-heading-soft"/>
                        <div class="text-sm font-medium leading-none">{service.label}</div>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! { <ul class="space-y-2">{links}</ul> }
}

/// A bar with a hamburger below `md`, opening the whole viewport.
///
/// Dropdowns are a poor fit for a touch screen: everything is laid out flat here
/// instead, so no link is more than one tap away.
#[component]
fn MobileNav(services: Services) -> impl IntoView {
    let open = RwSignal::new(false);
    let close = move |_| open.set(false);

    view! {
        // The page behind must not scroll under the open panel; `:has` keeps that
        // rule next to the markup it depends on rather than in a global sheet.
        <style>"body:has([data-mobile-nav='open']) { overflow: hidden; }"</style>

        <div class="flex justify-between items-center px-4 py-4 md:hidden">
            // No `aria-label`: it would override the visible word it duplicates.
            <a href="/" class="flex gap-2 items-center rounded-md">
                <HomeLogo/>
                <HomeWord/>
            </a>

            <div class="flex gap-1 items-center">
                <ThemeToggle/>
                <button
                    type="button"
                    aria-label="Ouvrir le menu"
                    aria-expanded=move || open.get().to_string()
                    class="inline-flex justify-center items-center w-10 h-10 rounded-md transition-colors hover:bg-accent"
                    on:click=move |_| open.set(true)
                >
                    <Menu class="w-6 h-6"/>
                </button>
            </div>
        </div>

        <div
            data-mobile-nav=move || if open.get() { "open" } else { "closed" }
            class=move || {
                // `invisible` rather than merely transparent, so a closed panel
                // stays out of the tab order and out of screen readers.
                let state = if open.get() { "opacity-100" } else { "opacity-0 invisible" };
                format!(
                    "flex fixed inset-0 z-50 flex-col transition-opacity duration-200 bg-background md:hidden {state}",
                )
            }
        >
            <div class="flex justify-between items-center px-4 py-4 border-b">
                <a href="/" class="flex gap-2 items-center rounded-md" on:click=close>
                    <HomeLogo/>
                    <HomeWord/>
                </a>
                <button
                    type="button"
                    aria-label="Fermer le menu"
                    class="inline-flex justify-center items-center w-10 h-10 rounded-md transition-colors hover:bg-accent"
                    on:click=close
                >
                    <X class="w-6 h-6"/>
                </button>
            </div>

            <nav class="overflow-y-auto flex-1 px-4 py-6">
                <ul class="space-y-2">
                    <li>
                        <div class="px-3 py-2 text-lg font-semibold">"Ateliers à domicile"</div>
                        <MobileServiceLinks services=services pro=false open=open/>
                    </li>

                    // Carried by an `<li>` because a `<ul>` takes no other child,
                    // and hidden from assistive tech: it divides the sections
                    // visually, and the headings already do so structurally.
                    <li aria-hidden="true" class="py-2">
                        <SectionWave/>
                    </li>

                    <li>
                        <div class="px-3 py-2 text-lg font-semibold">"Autres Ateliers"</div>
                        <MobileServiceLinks services=services pro=true open=open/>
                    </li>

                    <li aria-hidden="true" class="py-2">
                        <SectionWave/>
                    </li>

                    <li>
                        <div class="px-3 py-2 text-lg font-semibold">"Moi et mon atelier"</div>
                        <ul class="mt-2 ml-4 space-y-1">
                            <li>
                                <a href="/moi-et-mon-atelier/qui-suis-je" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Qui-suis-je?"
                                </a>
                            </li>
                            <li>
                                <a href="/moi-et-mon-atelier/mon-atelier" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Mon atelier"
                                </a>
                            </li>
                            <li>
                                <a href="/moi-et-mon-atelier/photos" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Photos des ateliers"
                                </a>
                            </li>
                            <li>
                                <a href="/moi-et-mon-atelier/catalogue" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Catalogue des ateliers"
                                </a>
                            </li>
                            <li>
                                <a href="/moi-et-mon-atelier/diplomes-et-formations" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Diplômes et formations"
                                </a>
                            </li>
                        </ul>
                    </li>
                </ul>
            </nav>
        </div>
    }
}

/// The workshops of one section, as the mobile panel lists them.
///
/// Takes the signal rather than a closure so that each link can close the panel:
/// a component prop would otherwise need a `Fn` that is `Copy`, `Send` and `Sync`,
/// which is more ceremony than the one `set` is worth.
#[component]
fn MobileServiceLinks(services: Services, pro: bool, open: RwSignal<bool>) -> impl IntoView {
    view! {
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                let list = services.await.map(|list| section(&list, pro)).unwrap_or_default();

                view! { <MobileServiceList services=list open=open/> }
            })}
        </Transition>
    }
}

/// The list itself, split from the fetch above for the same reason as
/// [`DesktopServiceList`].
#[component]
fn MobileServiceList(services: Vec<ServiceView>, open: RwSignal<bool>) -> impl IntoView {
    let links = services
        .into_iter()
        .map(|service| {
            // Both read before `label` is moved into the view.
            let href = service.page_path();
            let icon = service.icon;

            view! {
                <li>
                    <a
                        href=href
                        class="flex gap-3 items-center px-3 py-2 rounded-md transition-colors hover:bg-accent"
                        on:click=move |_| open.set(false)
                    >
                        <ServiceIcon name=icon class="w-4 h-4 shrink-0 text-heading-soft"/>
                        {service.label}
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! { <ul class="mt-2 ml-4 space-y-1">{links}</ul> }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn service(slug: &str, label: &str, icon: &str, pro: bool) -> ServiceView {
        ServiceView {
            id: "651d1f0a0000000000000001".to_owned(),
            slug: slug.to_owned(),
            label: label.to_owned(),
            description: String::new(),
            age: String::new(),
            steps: Vec::new(),
            icon: icon.to_owned(),
            pro,
            position: 0,
        }
    }

    /// Two pictos that differ, so an assertion about one cannot pass on the other.
    fn services() -> Vec<ServiceView> {
        vec![
            service("aperos-creatifs", "Apéros créatifs", "Wine", false),
            service("en-institution", "Ateliers en institution", "Building2", true),
        ]
    }

    /// Both menus are built from the same rows, so the filter is the one place
    /// they could disagree about where a workshop belongs.
    #[test]
    fn a_workshop_appears_in_its_section_and_only_there() {
        let at_home = section(&services(), false);
        let for_structures = section(&services(), true);

        assert_eq!(at_home.len(), 1, "{at_home:?}");
        assert_eq!(at_home[0].slug, "aperos-creatifs");
        assert_eq!(for_structures.len(), 1, "{for_structures:?}");
        assert_eq!(for_structures[0].slug, "en-institution");
    }

    /// The bar and the panel had drifted apart once before, one pointing at
    /// `/services/` and the other at `/pro/`. They now build the href the same way,
    /// from the row, so the destination is decided in one place.
    #[test]
    fn both_menus_build_the_same_destination() {
        for service in services() {
            let expected = if service.pro { "/pro/" } else { "/services/" };

            assert_eq!(
                service.page_path(),
                format!("{expected}{}", service.slug),
                "{} lands in the wrong section",
                service.slug
            );
        }
    }

    fn desktop_nav() -> String {
        crate::pages::admin::init_test_executor();

        Owner::new().with(|| {
            // The theme toggle sits in this bar and reads the context `App` provides.
            provide_context(crate::components::hooks::use_theme_mode::ThemeMode::init());

            let services: Services =
                Resource::new(|| (), |()| async move { Ok(super::tests::services()) });

            view! { <DesktopNav services=services/> }.to_html()
        })
    }

    fn mobile_nav() -> String {
        crate::pages::admin::init_test_executor();

        Owner::new().with(|| {
            provide_context(crate::components::hooks::use_theme_mode::ThemeMode::init());

            let services: Services =
                Resource::new(|| (), |()| async move { Ok(super::tests::services()) });

            view! { <MobileNav services=services/> }.to_html()
        })
    }

    /// The logo is taller than the text triggers beside it, so it must not inherit
    /// their fixed height, and it must stay out of baseline alignment: an
    /// `inline-flex` box takes its baseline from the image's bottom edge, which grows
    /// the row unevenly and knocks the logo off centre.
    #[test]
    fn the_logo_link_is_block_level_and_as_tall_as_its_image() {
        let html = desktop_nav();

        let logo = html
            .split("<a ")
            .find(|fragment| fragment.contains("/assets/icon.svg"))
            .expect("the logo link should render");

        let class = logo
            .split_once("class=\"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .expect("the logo link should carry classes")
            .0;

        assert!(class.contains("h-16"), "should match the 64px image: {class}");
        assert!(!class.contains("h-9"), "should not keep the trigger height: {class}");
        assert!(
            class.split_whitespace().any(|name| name == "flex"),
            "should be block-level flex: {class}"
        );
        assert!(
            !class.split_whitespace().any(|name| name == "inline-flex"),
            "inline-flex would reintroduce baseline alignment: {class}"
        );
    }

    /// Everything the logo link renders, bounded by its closing tag so an
    /// assertion cannot pass on markup belonging to a later link.
    fn logo_link() -> String {
        desktop_nav()
            .split("<a ")
            .find(|fragment| fragment.contains("/assets/icon.svg"))
            .and_then(|fragment| fragment.split_once("</a>").map(|(inside, _)| inside.to_owned()))
            .expect("the logo link should render")
    }

    /// The mark is a bubble with no lettering in it, so the word beside it is the
    /// only thing that says where the link leads. A label revealed on hover left
    /// out everyone not holding a pointer.
    #[test]
    fn the_home_link_names_itself_in_visible_text() {
        let logo = logo_link();

        assert!(logo.contains(">Accueil<"), "the word should render: {logo}");
        assert!(
            !logo.contains("opacity-0"),
            "and stay visible rather than wait for a hover: {logo}"
        );
    }

    /// Every home link across the two menus, bounded by its closing tag so an
    /// assertion cannot pass on markup belonging to a later link.
    fn home_links(html: &str) -> Vec<String> {
        html.split("<a ")
            .filter(|fragment| fragment.contains("/assets/icon.svg"))
            .filter_map(|fragment| {
                fragment.split_once("</a>").map(|(inside, _)| inside.to_owned())
            })
            .collect()
    }

    /// The bar, and the two the mobile side carries: its closed header and the
    /// open panel's. Gathered rather than counted over the page, which a later
    /// mention of the word anywhere else would throw off.
    #[test]
    fn every_home_link_names_itself_and_leaves_the_mark_silent() {
        let links = [(desktop_nav(), 1), (mobile_nav(), 2)];

        for (html, expected) in links {
            let found = home_links(&html);
            assert_eq!(found.len(), expected, "wrong number of home links: {html}");

            for link in found {
                assert!(link.contains(">Accueil<"), "unnamed home link: {link}");
                assert!(
                    link.contains(r#"alt="""#),
                    "the mark should be decorative beside the word: {link}"
                );
                assert!(
                    !link.contains(r#"aria-label="#),
                    "an aria-label would override the visible word: {link}"
                );
            }
        }
    }

    /// A page reachable from one menu only is invisible to half the visitors, so
    /// every entry under "Moi et mon atelier" has to sit in both.
    #[test]
    fn both_menus_reach_the_pages_of_the_workshop_section() {
        let bar = desktop_nav();
        let panel = mobile_nav();

        for slug in ["qui-suis-je", "catalogue", "diplomes-et-formations"] {
            let href = format!("href=\"/moi-et-mon-atelier/{slug}\"");

            assert!(bar.contains(&href), "the bar is missing {href}: {bar}");
            assert!(panel.contains(&href), "the panel is missing {href}: {panel}");
        }
    }

    /// The two lists, rendered from a plain `Vec`. Going through either nav would
    /// only ever show the fallback: a `Transition` does not resolve under a
    /// synchronous `to_html`.
    fn service_lists(services: Vec<ServiceView>) -> [String; 2] {
        use leptos_router::components::Router;
        use leptos_router::location::RequestUrl;

        let for_bar = services.clone();
        let bar = Owner::new().with(move || {
            // The links resolve `aria-current` against the location, which
            // server-side is the request being rendered.
            provide_context(RequestUrl::new("/"));

            view! {
                <Router>
                    <DesktopServiceList services=for_bar/>
                </Router>
            }
            .to_html()
        });

        let panel = Owner::new().with(move || {
            provide_context(RequestUrl::new("/"));
            let open = RwSignal::new(true);

            view! {
                <Router>
                    <MobileServiceList services=services open=open/>
                </Router>
            }
            .to_html()
        });

        [bar, panel]
    }

    /// Both menus list the same workshops, each pointing at its own page.
    #[test]
    fn both_lists_name_every_workshop_and_link_to_it() {
        for rendered in service_lists(services()) {
            for service in services() {
                assert!(
                    rendered.contains(&service.label),
                    "{} is not named: {rendered}",
                    service.slug
                );
                assert!(
                    rendered.contains(&format!("href=\"{}\"", service.page_path())),
                    "{} has no link: {rendered}",
                    service.slug
                );
            }
        }
    }

    /// The picto is what the admin chose to stand for a workshop, so it belongs
    /// wherever that workshop is listed -- in both menus, not just on its page.
    #[test]
    fn both_lists_draw_the_picto_beside_the_workshop() {
        for rendered in service_lists(services()) {
            for service in services() {
                // The icon crate stamps the name on the `<svg>` it draws, which is
                // the only way to tell one picto from another in the markup.
                let drawn = format!(r#"data-name="{}""#, service.icon);

                assert!(rendered.contains(&drawn), "missing {drawn}: {rendered}");
            }
        }
    }

    /// A workshop may carry no picto, and the row then has to close up rather than
    /// hold an empty square where the icon would have gone.
    #[test]
    fn a_workshop_without_a_picto_draws_none() {
        let bare = vec![service("aperos-creatifs", "Apéros créatifs", "", false)];

        for rendered in service_lists(bare) {
            assert!(!rendered.contains("<svg"), "drew something: {rendered}");
            assert!(rendered.contains("Apéros créatifs"), "lost the label: {rendered}");
        }
    }

    /// The sections are the menu's own structure, so they render whether or not
    /// the workshops have loaded: a heading that came and went with its contents
    /// would make the bar jump on every page.
    #[test]
    fn the_sections_stand_whatever_the_workshops_do() {
        let panel = mobile_nav();

        for heading in ["Ateliers à domicile", "Autres Ateliers", "Moi et mon atelier"] {
            assert!(panel.contains(heading), "the panel lost {heading}: {panel}");
        }
    }

    /// A rule goes between the sections, not after each one, so their counts stay
    /// one apart however many sections the menu grows to.
    #[test]
    fn a_wave_sits_between_the_mobile_sections_and_not_after_the_last() {
        let html = mobile_nav();

        let sections = html.matches("text-lg font-semibold").count();
        let waves = html.matches("text-heading-soft").count();

        assert!(sections >= 2, "the menu should have sections to divide: {sections}");
        assert_eq!(
            waves,
            sections - 1,
            "{sections} sections want {} rules, found {waves}",
            sections - 1
        );
    }

    /// An icon-sized mark has to be centred, and an `svg` is inline by default,
    /// which leaves `mx-auto` nothing to work with.
    #[test]
    fn the_wave_is_centred_as_a_block() {
        let html = mobile_nav();

        let class = html
            .split("<svg")
            .find(|fragment| fragment.contains("text-heading-soft"))
            .and_then(|fragment| fragment.split_once("class=\""))
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(class, _)| class)
            .expect("the wave should render with classes");

        let names: Vec<&str> = class.split_whitespace().collect();

        assert!(names.contains(&"mx-auto"), "it should be centred: {class}");
        assert!(
            names.contains(&"block"),
            "mx-auto does nothing on an inline box: {class}"
        );
        assert!(
            !names.contains(&"w-full"),
            "it is an icon between the sections, not a rule across them: {class}"
        );
    }
}
