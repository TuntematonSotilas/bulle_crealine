use icons::{Menu, X};
use leptos::prelude::*;

use crate::components::ui::{navigation_menu::*, theme_toggle::ThemeToggle};


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
                        <NavigationMenuTrigger>"Ateliers à domicile"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <div class="w-[320px] p-3">
                                <ul class="space-y-2">
                                    <li class="p-1">
                                        <div class="block text-sm font-medium leading-none text-foreground">
                                            "Ateliers parents-enfants"
                                        </div>
                                        <ul class="mt-2 ml-4 space-y-1 text-sm text-muted-foreground">
                                            <li>
                                                <a href="/services/parents-enfants-moins-6-ans" class="block py-1 transition-colors hover:text-primary">
                                                    "Moins de 6 ans"
                                                </a>
                                            </li>
                                            <li>
                                                <a href="/services/parents-enfants-6-12-ans" class="block py-1 transition-colors hover:text-primary">
                                                    "6 à 12 ans"
                                                </a>
                                            </li>
                                        </ul>
                                    </li>
                                    <li class="p-1">
                                        <div class="block text-sm font-medium leading-none text-foreground">
                                            "Ateliers adultes"
                                        </div>
                                        <ul class="mt-2 ml-4 space-y-1 text-sm text-muted-foreground">
                                            <li class="p-1">
                                                <a href="/services/aperos-creatifs" class="block text-sm font-medium leading-none transition-colors hover:text-primary">
                                                    "Apéros créatifs"
                                                </a>
                                            </li>
                                            <li class="p-1">
                                                <a href="/services/apres-midis-creatifs" class="block text-sm font-medium leading-none transition-colors hover:text-primary">
                                                    "Après-midis créatifs"
                                                </a>
                                            </li>
                                        </ul>
                                    </li>
                                </ul>
                            </div>
                        </NavigationMenuContent>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                         <NavigationMenuTrigger>"Autres Ateliers"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <div class="w-[320px] p-3">
                                <ul class="space-y-2">
                                    <li class="p-1">
                                        <a href="/services/en-institution" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Ateliers en institution"</div>
                                        </a>
                                    </li>
                                    <li class="p-1">
                                        <a href="/services/hors-les-murs" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Ateliers hors les murs"</div>
                                        </a>
                                    </li>
                                    <li class="p-1">
                                        <a href="/services/individuels" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Ateliers individuels"</div>
                                        </a>
                                    </li>
                                </ul>
                            </div>
                        </NavigationMenuContent>
                    </NavigationMenuItem>

                    <NavigationMenuItem>
                         <NavigationMenuTrigger>"Moi et mon atelier"</NavigationMenuTrigger>
                        <NavigationMenuContent>
                            <div class="w-[320px] p-3">
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
                                        <a href="/moi-et-mon-atelier/formations" class="block p-3 space-y-1 leading-none no-underline rounded-md transition-colors outline-none select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                            <div class="text-sm font-medium leading-none">"Formations"</div>
                                        </a>
                                    </li>
                                </ul>
                            </div>
                        </NavigationMenuContent>
                    </NavigationMenuItem>
                    
                    

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
                <ul class="space-y-2">
                    <li>
                        <div class="px-3 py-2 text-lg font-semibold">"Ateliers à domicile"</div>
                        <ul class="ml-4 mt-2 space-y-1">
                            <li>
                                <div class="block px-3 py-2 rounded-md transition-colors hover:bg-accent">
                                    "Ateliers parents-enfants"
                                </div>
                                <ul class="ml-4 mt-1 space-y-1 text-sm text-muted-foreground">
                                    <li>
                                        <a href="/services/parents-enfants-moins-6-ans" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                            "Moins de 6 ans"
                                        </a>
                                    </li>
                                    <li>
                                        <a href="/services/parents-enfants-6-12-ans" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                            "6 à 12 ans"
                                        </a>
                                    </li>
                                </ul>
                            </li>
                            <li>
                                <a href="/services/aperos-creatifs" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Apéros créatifs"
                                </a>
                            </li>
                            <li>
                                <a href="/services/apres-midis-creatifs" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Après-midis créatifs"
                                </a>
                            </li>
                        </ul>
                    </li>

                    <li>
                        <div class="px-3 py-2 text-lg font-semibold">"Autres Ateliers"</div>
                        <ul class="ml-4 mt-2 space-y-1">
                            <li>
                                <a href="/pro/en-institution" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Ateliers en institution"
                                </a>
                            </li>
                            <li>
                                <a href="/pro/hors-les-murs" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Ateliers hors les murs"
                                </a>
                            </li>
                            <li>
                                <a href="/pro/individuels" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Ateliers individuels"
                                </a>
                            </li>
                        </ul>
                    </li>

                    <li>
                        <div class="px-3 py-2 text-lg font-semibold">"Moi et mon atelier"</div>
                        <ul class="ml-4 mt-2 space-y-1">
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
                                <a href="/moi-et-mon-atelier/formations" class="block px-3 py-2 rounded-md transition-colors hover:bg-accent" on:click=close>
                                    "Formations"
                                </a>
                            </li>
                        </ul>
                    </li>
                </ul>
            </nav>
        </div>
    }
}
