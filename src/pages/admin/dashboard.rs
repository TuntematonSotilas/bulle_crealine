use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::Redirect;

use crate::auth::{LOGIN_PATH, Logout, admin_email};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};

/// Home page of the admin area.
#[component]
pub fn AdminPage() -> impl IntoView {
    // On a page load, `admin_guard` has already turned unauthenticated visitors
    // away. This resource covers Leptos's in-app navigation, which does not go
    // back to the server: without it, a token expiring mid-session would still
    // let the page render.
    let session = Resource::new(|| (), |_| async move { admin_email().await });

    view! {
        <Title text="Administration — Bulle Créaline"/>

        <Transition fallback=|| {
            view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
        }>
            {move || Suspend::new(async move {
                match session.await {
                    Ok(Some(email)) => Either::Left(view! { <AdminHome email=email/> }),
                    _ => Either::Right(view! { <Redirect path=LOGIN_PATH/> }),
                }
            })}
        </Transition>
    }
}

/// Admin content, once access has been established.
#[component]
fn AdminHome(email: String) -> impl IntoView {
    let logout = ServerAction::<Logout>::new();

    view! {
        <div class="flex flex-col gap-6 mx-auto max-w-3xl">

            <div class="flex flex-wrap gap-4 justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold">"Administration"</h1>
                    <p class="text-sm text-muted-foreground">{email}</p>
                </div>

                <ActionForm action=logout>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Se déconnecter"
                    </Button>
                </ActionForm>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>"Rien à gérer pour l'instant"</CardTitle>
                    <CardDescription>
                        "Cet espace est prêt à accueillir la gestion du contenu du site."
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <p class="text-sm text-muted-foreground">
                    </p>
                </CardContent>
            </Card>

        </div>
    }
}
