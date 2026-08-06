use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Redirect;

use crate::auth::{LOGIN_PATH, Logout, admin_email};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};

/// The admin pages, and the label shown in the shell's navigation.
const PAGES: [(&str, &str); 3] = [
    ("/admin", "Accueil"),
    ("/admin/sessions", "Séances"),
    ("/admin/bookings", "Réservations"),
];

/// Frame shared by every admin page: navigation, the logged-in address, and the
/// logout button.
///
/// It also re-checks the session. On a page load `admin_guard` has already done
/// that, but Leptos's in-app navigation never reaches the server, so without this
/// a token expiring mid-visit would leave the frame on screen. It is a convenience
/// rather than a barrier: the data itself is guarded by `require_admin` inside each
/// server function.
#[component]
pub fn AdminShell(
    /// Heading of the page being framed.
    #[prop(into)]
    title: String,
    /// Path of that page, to mark the matching navigation link.
    #[prop(into)]
    current: String,
    children: ChildrenFn,
) -> impl IntoView {
    let session = Resource::new(|| (), |_| async move { admin_email().await });

    view! {
        <Transition fallback=|| {
            view! { <p class="text-sm text-muted-foreground">"Chargement…"</p> }
        }>
            {move || {
                let title = title.clone();
                let current = current.clone();
                let children = children.clone();

                Suspend::new(async move {
                    match session.await {
                        Ok(Some(email)) => {
                            Either::Left(
                                view! {
                                    <Frame title=title current=current email=email>
                                        {children()}
                                    </Frame>
                                },
                            )
                        }
                        _ => Either::Right(view! { <Redirect path=LOGIN_PATH/> }),
                    }
                })
            }}
        </Transition>
    }
}

#[component]
fn Frame(
    title: String,
    current: String,
    email: String,
    children: ChildrenFn,
) -> impl IntoView {
    let logout = ServerAction::<Logout>::new();

    let links = PAGES
        .into_iter()
        .map(|(path, label)| {
            let active = path == current;

            view! {
                <a
                    href=path
                    aria-current=if active { "page" } else { "false" }
                    class=if active {
                        "px-3 py-1.5 text-sm font-medium rounded-md bg-accent text-accent-foreground"
                    } else {
                        "px-3 py-1.5 text-sm font-medium rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                    }
                >
                    {label}
                </a>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="flex flex-col gap-6 mx-auto max-w-5xl">

            <div class="flex flex-wrap gap-4 justify-between items-center">
                <div>
                    <h1 class="text-2xl font-semibold">{title}</h1>
                    <p class="text-sm text-muted-foreground">{email}</p>
                </div>

                <ActionForm action=logout>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                        "Se déconnecter"
                    </Button>
                </ActionForm>
            </div>

            <nav class="flex flex-wrap gap-1 pb-2 border-b">{links}</nav>

            {children()}

        </div>
    }
}
