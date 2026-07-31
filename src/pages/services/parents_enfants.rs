use leptos::prelude::*;

use crate::components::blocks::service_block::{DetPerAge, ServiceBlock, Session};

/// Renders the "Ateliers parents-enfants" page.
#[component]
pub fn AteliersParentsEnfants() -> impl IntoView {

    let title = "Ateliers parents-enfants";
    let desc = "Nos ateliers parents-enfants offrent un espace de création commune, favorisant un temps de partage loin des impératifs quotidiens.";
    let dets_per_age = vec![
        DetPerAge {
            prefix: "Pour les enfants de ".to_string(),
            age: "moins de 3 ans".to_string(),
            description: ", pour un moment de decouverte de la créativité adapté et sans salir toute la maison.".to_string(),
        },
        DetPerAge {
            prefix: "Pour les enfants de ".to_string(),
            age: "3 à 10 ans".to_string(),
            description: ", pour découvrir le plaisir de créer et la magie de créer pour soi ou pour les autres.".to_string(),
        },
        DetPerAge {
            prefix: "Pour les enfants de ".to_string(),
            age: "10 à 13 ans".to_string(),
            description: ", pour découvrir l'autonomie et apprendre à innover.".to_string(),
        },
    ];
    let schedule = "Chaque premier mercredi du mois, de 10h à 12h";
    let place = "Bulle Créaline, 5 Rue Marc Seguin, 42110 Feurs";
    let age = "De 0 à 13 ans";
    let place_link = "https://maps.app.goo.gl/Fgmpg9RF8HiPGrkf7";
    let steps = vec![
        "Accueil et présentation de l'atelier",
        "Petit exercice créatif simple",
        "Explication du thème et du matériel",
        "Découverte des materiaux et des techniques par les participants",
        "Choix du projet par le participant (accompagnement possible)",
        "Réalisation du projet",
        "Temps de partage et d'échange autour des créations",
        "Clôture de la séance et prise de retours"
    ].into_iter().map(String::from).collect::<Vec<String>>();
    let sessions = vec![
        Session {
            date: "Mercredi 3 juillet 2026".to_string(),
            theme: "Animaux / Scuplture".to_string(),
            price: "65".to_string(),
        },
        Session {
            date: "Mercredi 7 août 2026".to_string(),
            theme: "Nature / Peinture".to_string(),
            price: "65".to_string(),
        },
    ];
    let pics = vec![
        "/assets/fake1.png".to_string(),
        "/assets/fake2.png".to_string(),
    ];
    view! {
        <ServiceBlock title=title 
            description=desc
            details_per_age=dets_per_age
            pictures=pics
            is_register=true
            schedule=schedule
            place=place
            age=age
            place_link=place_link
            steps=steps
            sessions=sessions/>
    }
}
