use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::pages::admin::AdminShell;

/// Home page of the admin area.
#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Title text="Administration — Bulle Créaline"/>

        <AdminShell title="Administration" current="/admin">
            <div class="grid gap-4 md:grid-cols-2">

                <Card>
                    <CardHeader>
                        <CardTitle>"Séances"</CardTitle>
                        <CardDescription>
                            "Créer, modifier et supprimer les dates proposées à la réservation."
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <a href="/admin/sessions" class="text-sm underline underline-offset-4">
                            "Gérer les séances"
                        </a>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>"Thèmes"</CardTitle>
                        <CardDescription>
                            "Gérer les thèmes, leur nom et leur photo, avant de les associer aux séances."
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <a href="/admin/themes" class="text-sm underline underline-offset-4">
                            "Gérer les thèmes"
                        </a>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>"Réservations"</CardTitle>
                        <CardDescription>
                            "Consulter les inscriptions et y attacher une note interne."
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <a href="/admin/bookings" class="text-sm underline underline-offset-4">
                            "Voir les réservations"
                        </a>
                    </CardContent>
                </Card>

            </div>
        </AdminShell>
    }
}
