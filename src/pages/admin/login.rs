use leptos::prelude::*;
use leptos_meta::Title;

use crate::auth::{Login, user_message};
use crate::components::ui::button::Button;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;

/// Page de connexion à l'espace d'administration.
#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let login = ServerAction::<Login>::new();
    let pending = login.pending();

    // `ActionForm` soumet le formulaire nativement quand le WASM n'est pas
    // chargé : la connexion reste possible, le serveur répondant alors par une
    // redirection HTTP classique.
    let error = move || {
        login
            .value()
            .get()
            .and_then(|result| result.err())
            .map(|error| user_message(&error))
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
                                    <Input
                                        r#type=InputType::Password
                                        id="password"
                                        name="password"
                                        autocomplete="current-password"
                                        required=true
                                    />
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
