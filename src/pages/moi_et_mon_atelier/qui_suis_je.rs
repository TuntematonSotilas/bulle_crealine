use leptos::prelude::*;

/// Renders the "About Me" page.
#[component]
pub fn QuiSuisJePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl space-y-12 py-6">
            <div class="grid gap-10 lg:grid-cols-[minmax(0,1fr)_420px] items-start">
                <div class="space-y-6">
                    <div class="space-y-4">
                        <h1 class="text-4xl font-semibold tracking-tight">"Qui suis-je ?"</h1>
                        <p class="text-lg leading-8 text-muted-foreground">
                            "Née en 1992, j’ai longtemps cherché ma place. Créative depuis toujours, j’ai appris au fil du temps à transformer mes différences en force et à avancer à mon rythme."
                        </p>
                    </div>

                    <div class="space-y-4 text-base leading-8 text-foreground">
                        <p>
                            "Mes expériences de vie, mes formations et mes déclics personnels m’ont peu à peu menée vers ce qui me ressemble profondément : créer, transmettre et accompagner."
                        </p>
                        <p>
                            "Aujourd’hui, j’ai à cœur de proposer des ateliers créatifs, doux et bienveillants, où chacun peut explorer, essayer et s’exprimer sans pression. Je crois beaucoup à la pair-aidance, au partage, et à la créativité comme outil de reconnexion à soi."
                        </p>
                        <p>
                            "Avec La Bulle Créaline, je souhaite offrir un lieu chaleureux, vivant et rassurant, où l’on vient créer, souffler et repartir avec un peu plus de fierté et de joie."
                        </p>
                    </div>
                </div>

                <div class="rounded-[2rem] overflow-hidden border border-border bg-background shadow-lg">
                    <img src="/assets/coco.png" alt="Portrait" class="block h-full w-full object-cover" />
                </div>
            </div>

            <div class="grid gap-8 lg:grid-cols-[1fr_320px]">
                <div class="rounded-[2rem] border border-border bg-card p-8 shadow-sm">
                    <h2 class="text-2xl font-semibold">"Ce qui m’anime"</h2>
                    <ul class="mt-6 space-y-3 text-base leading-7 text-foreground">
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
                            "l’envie d’apporter un petit quelque chose de beau autour de moi"
                        </li>
                    </ul>
                </div>

                <div class="rounded-[2rem] border border-border bg-surface p-6 shadow-sm">
                    <p class="text-base leading-8 text-foreground">"Créative depuis l’enfance, j’ai longtemps cherché ma place avant de comprendre que mes différences faisaient aussi ma force. Aujourd’hui, j’ai choisi de les mettre au service d’un projet qui me ressemble : proposer des ateliers créatifs, doux et accessibles, dans un esprit de partage, de bienveillance et de reconnexion à soi."</p>
                </div>
            </div>
        </div>
    }
}
