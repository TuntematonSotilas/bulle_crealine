use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};

/// Renders the "About Me" page.
#[component]
pub fn QuiSuisJePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl space-y-12 py-6">
            <div class="grid gap-10 lg:grid-cols-[minmax(0,1fr)_420px] items-start">
                <div class="space-y-6">
                    <div class="space-y-4">
                        <h1 class="text-4xl font-semibold tracking-tight text-heading">
                            "Qui suis-je ?"
                        </h1>
                        <span class="block h-1 w-10 rounded-full bg-heading-soft"></span>
                        <p class="text-lg leading-8 text-muted-foreground">
                            "Créative depuis toujours, j'ai appris au fil du temps à transformer mes différences en force et à avancer à mon rythme."
                        </p>
                    </div>

                    <div class="space-y-4 text-base leading-8 text-card-foreground p-6 rounded-[2rem] border border-border bg-card shadow-lg">
                        <h2 class="text-2xl font-semibold tracking-tight text-heading">
                            "Mon parcours"
                        </h2>
                        <p>
                            "Née en 1992, j'ai longtemps cherché ma place. Mes expériences de vie, mes formations et mes déclics personnels m'ont peu à peu menée vers ce qui me ressemble profondément : créer, transmettre et accompagner."
                        </p>
                        <p>
                            "Aujourd'hui, j'ai à cœur de proposer des ateliers créatifs, doux et bienveillants, où chacun peut explorer, essayer et s'exprimer sans pression. Je crois beaucoup à la pair-aidance, au partage, et à la créativité comme outil de reconnexion à soi."
                        </p>

                    </div>
                </div>

                <div class="rounded-[2rem] overflow-hidden border border-border bg-card shadow-lg">
                    <img src="/assets/coco.png" alt="Portrait" class="block h-full w-full object-cover" />
                </div>
            </div>

            <div class="grid gap-8 lg:grid-cols-[1fr_320px]">
                <div class="rounded-[2rem] border border-border bg-card p-8 shadow-sm">
                    <h2 class="text-2xl font-semibold text-heading-soft">"Ce qui m'anime"</h2>
                    <ul class="mt-6 space-y-3 text-base leading-7 text-card-foreground">
                        <li class="flex items-start gap-3">
                            <span class="inline-flex h-2.5 w-2.5 rounded-full bg-primary mt-1.5"></span>
                            "la créativité comme espace de liberté"
                        </li>
                        <li class="flex items-start gap-3">
                            <span class="inline-flex h-2.5 w-2.5 rounded-full bg-primary mt-1.5"></span>
                            "le partage et la bienveillance"
                        </li>
                        <li class="flex items-start gap-3">
                            <span class="inline-flex h-2.5 w-2.5 rounded-full bg-primary mt-1.5"></span>
                            "l'envie d'apporter un petit quelque chose de beau autour de moi"
                        </li>
                    </ul>
                </div>

                <div class="space-y-6">
                    <div class="flex justify-center">
                        <span class="block h-1 w-10 rounded-full bg-primary"></span>
                    </div>

                    <div class="space-y-4 rounded-[2rem] border border-border bg-surface text-surface-foreground p-6 shadow-sm">
                        <p class="text-base leading-8">
                            "Avec "
                            <span class="font-semibold text-heading">"La Bulle Créaline"</span>
                            ", je souhaite offrir un lieu chaleureux, vivant et rassurant, où l'on vient créer, souffler et repartir avec un peu plus de fierté et de joie."
                        </p>
                        /* Sized at 24px so it clears WCAG as large text: the accent
                           colour only reaches 4.02:1 on this panel, under the 4.5
                           demanded at body size but over the 3.0 for large text. */
                        <p class="text-2xl italic leading-snug text-heading">
                            "« Créer, c'est se retrouver. »"
                        </p>
                    </div>
                </div>
            </div>

            <div class="flex flex-wrap justify-center gap-6">
                <Button size=ButtonSize::Pill href="/moi-et-mon-atelier/diplomes-et-formations">
                    "Diplômes et formations"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Pill
                    href="/moi-et-mon-atelier/mon-atelier"
                >
                    "Mon atelier"
                </Button>
                <Button
                    variant=ButtonVariant::Tertiary
                    size=ButtonSize::Pill
                    href="/moi-et-mon-atelier/photos"
                >
                    "Album de mes ateliers"
                </Button>
            </div>
        </div>
    }
}
