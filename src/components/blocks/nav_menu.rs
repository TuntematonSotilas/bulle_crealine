use icons::{Menu, X};
use leptos::prelude::*;

use crate::components::ui::{navigation_menu::*, theme_toggle::ThemeToggle};
use crate::models::ServiceType;

/// The pages that are not one of the workshops.
const PAGES: [(&str, &str); 2] = [
    ("/qui-suis-je", "Qui suis-je"),
    ("/newsletter", "Newsletter"),
];

#[component]
pub fn NavMenu() -> impl IntoView {
    view! {
        <DesktopNav/>
        <MobileNav/>
    }
}

/// The hover-driven bar, from `md` up.
#[component]
fn DesktopNav() -> impl IntoView {
    view! {
        <div class="hidden justify-center items-start py-8 md:flex">
            <NavigationMenu>
                <NavigationMenuList>

                    <NavigationMenuItem>
                        <NavigationMenuLink href="/" class=navigation_menu_trigger_style()>
                            <img src="/assets/icon.svg" alt="Logo" class="w-8 h-8"/>
                        </NavigationMenuLink>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                        <NavigationMenuTrigger>"Services"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <ul class="grid gap-3 p-0 md:grid-cols-2 md:w-[500px] lg:w-[600px]">
                                {ServiceType::ALL
                                    .into_iter()
                                    .map(|service| {
                                        view! {
                                            <li>
                                                <a
                                                    href=service.page_path()
                                                    class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                                                >
                                                    <div class="text-sm font-medium leading-none">
                                                        {service.label()}
                                                    </div>
                                                </a>
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </ul>
                        </NavigationMenuContent>
                    </NavigationMenuItem>

                    {PAGES
                        .into_iter()
                        .map(|(href, title)| {
                            view! {
                                <NavigationMenuItem>
                                    <NavigationMenuLink
                                        class=navigation_menu_trigger_style()
                                        href=href
                                    >
                                        {title}
                                    </NavigationMenuLink>
                                </NavigationMenuItem>
                            }
                        })
                        .collect::<Vec<_>>()}

                    <NavigationMenuItem>
                        <ThemeToggle/>
                    </NavigationMenuItem>

                </NavigationMenuList>
            </NavigationMenu>
        </div>
    }
}

/// A bar with a hamburger below `md`, opening the whole viewport.
///
/// Dropdowns are a poor fit for a touch screen: everything is laid out flat here
/// instead, so no link is more than one tap away.
#[component]
fn MobileNav() -> impl IntoView {
    let open = RwSignal::new(false);
    let close = move |_| open.set(false);

    view! {
        // The page behind must not scroll under the open panel; `:has` keeps that
        // rule next to the markup it depends on rather than in a global sheet.
        <style>"body:has([data-mobile-nav='open']) { overflow: hidden; }"</style>

        <div class="flex justify-between items-center px-4 py-4 md:hidden">
            <a href="/" aria-label="Accueil">
                <img src="/assets/icon.svg" alt="Bulle Créaline" class="w-8 h-8"/>
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
                <a href="/" aria-label="Accueil" on:click=close>
                    <img src="/assets/icon.svg" alt="Bulle Créaline" class="w-8 h-8"/>
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
                <p class="px-3 mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                    "Services"
                </p>
                <ul class="mb-8">
                    {ServiceType::ALL
                        .into_iter()
                        .map(|service| {
                            view! {
                                <li>
                                    <a
                                        href=service.page_path()
                                        class="block px-3 py-3 text-lg rounded-md transition-colors hover:bg-accent"
                                        on:click=close
                                    >
                                        {service.label()}
                                    </a>
                                </li>
                            }
                        })
                        .collect::<Vec<_>>()}
                </ul>

                <ul class="pt-6 border-t">
                    {PAGES
                        .into_iter()
                        .map(|(href, title)| {
                            view! {
                                <li>
                                    <a
                                        href=href
                                        class="block px-3 py-3 text-lg rounded-md transition-colors hover:bg-accent"
                                        on:click=close
                                    >
                                        {title}
                                    </a>
                                </li>
                            }
                        })
                        .collect::<Vec<_>>()}
                </ul>
            </nav>
        </div>
    }
}
