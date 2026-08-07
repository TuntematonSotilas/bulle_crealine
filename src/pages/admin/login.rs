use icons::{Eye, EyeOff};
use leptos::html;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::auth::{Login, user_message};
use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;

/// Login page of the admin area.
#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let login = ServerAction::<Login>::new();
    let pending = login.pending();

    // `ActionForm` submits natively when the WASM bundle has not loaded: logging
    // in still works, the server answering with a plain HTTP redirect.
    let error = move || {
        login
            .value()
            .get()
            .and_then(|result| result.err())
            .map(|error| user_message(&error))
    };

    let password = NodeRef::<html::Input>::new();
    let revealed = RwSignal::new(false);

    // The type is flipped on the element itself rather than through a prop:
    // `Input` takes its type as a plain value, so there is nothing reactive to
    // drive. Without the WASM bundle the button does nothing and the field simply
    // stays masked.
    let toggle_password = move |_| {
        let next = !revealed.get_untracked();
        revealed.set(next);

        if let Some(field) = password.get_untracked() {
            field.set_type(if next { "text" } else { "password" });
        }
    };

    view! {
        <Title text="Connexion — Bulle Créaline"/>

        <div class="flex justify-center items-center py-10 w-full">
            <div class="w-full max-w-sm">
                <Card>
                    <CardHeader>
                        <CardTitle>"Connexion"</CardTitle>
                        <CardDescription>
                            "Accès réservé à l'administration du site."
                        </CardDescription>
                    </CardHeader>

                    <CardContent>
                        <ActionForm action=login>
                            <div class="flex flex-col gap-6">

                                <div class="grid gap-3">
                                    <Label r#for="email">"Adresse e-mail"</Label>
                                    <Input
                                        r#type=InputType::Email
                                        id="email"
                                        name="email"
                                        placeholder="vous@exemple.fr"
                                        autocomplete="username"
                                        required=true
                                        autofocus=true
                                    />
                                </div>

                                <div class="grid gap-3">
                                    <Label r#for="password">"Mot de passe"</Label>
                                    <div class="relative">
                                        <Input
                                            r#type=InputType::Password
                                            id="password"
                                            name="password"
                                            autocomplete="current-password"
                                            required=true
                                            node_ref=password
                                            class="pr-10"
                                        />
                                        <button
                                            type="button"
                                            on:click=toggle_password
                                            aria-controls="password"
                                            aria-pressed=move || {
                                                if revealed.get() { "true" } else { "false" }
                                            }
                                            aria-label=move || {
                                                if revealed.get() {
                                                    "Masquer le mot de passe"
                                                } else {
                                                    "Afficher le mot de passe"
                                                }
                                            }
                                            class="flex absolute inset-y-0 right-0 items-center px-3 rounded-r-md text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
                                        >
                                            <Show
                                                when=move || revealed.get()
                                                fallback=|| view! { <Eye class="size-4"/> }
                                            >
                                                <EyeOff class="size-4"/>
                                            </Show>
                                        </button>
                                    </div>
                                </div>

                                {move || {
                                    error()
                                        .map(|message| {
                                            view! {
                                                <p
                                                    role="alert"
                                                    class="text-sm font-medium text-destructive"
                                                >
                                                    {message}
                                                </p>
                                            }
                                        })
                                }}

                                <Button class="w-full">
                                    {move || {
                                        if pending.get() { "Connexion…" } else { "Se connecter" }
                                    }}
                                </Button>

                            </div>
                        </ActionForm>
                    </CardContent>
                </Card>
            </div>
        </div>
    }
}
