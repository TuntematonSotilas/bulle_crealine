use leptos::prelude::*;
use crate::components::blocks::service_block::ServiceBlock;
use crate::models::ServiceType;

/// Renders the "Ateliers créatifs pour tous" page.
#[component]
pub fn AteliersCreatifsPourTous() -> impl IntoView {

    let title = "Ateliers créatifs pour tous";
    let desc = "Découvrez nos ateliers créatifs conçus pour tous les âges et tous les niveaux.";
    let schedule = "Tous les dimanches, de 14h à 17h";
    let place = "Bulle Créaline, 5 Rue Marc Seguin, 42110 Feurs";
    let age = "À partir de 13 ans";
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
    let pics = vec![
        "/assets/fake1.png".to_string(),
        "/assets/fake2.png".to_string(),
    ];
    view! {
        <ServiceBlock title=title 
            description=desc
            pictures=pics
            schedule=schedule
            place=place
            age=age
            place_link=place_link
            steps=steps
            service=ServiceType::CreatifsPourTous/>
    }
}
