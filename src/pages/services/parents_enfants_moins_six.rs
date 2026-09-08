use leptos::prelude::*;

use crate::components::blocks::service_block::ServiceBlock;
use crate::models::ServiceType;

#[component]
pub fn AteliersParentsEnfantsMoinsSix() -> impl IntoView {

    let title = "Ateliers parents-enfants (moins de 6 ans)";
    let desc = ServiceType::ParentsEnfantsMoinsSix.description();

    let schedule = "Chaque premier mercredi du mois, de 10h à 12h";
    let place = "Bulle Créaline, 5 Rue Marc Seguin, 42110 Feurs";
    let age = "De 0 à 6 ans";
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
    view! {
        <ServiceBlock title=title 
            description=desc
            schedule=schedule
            place=place
            age=age
            place_link=place_link
            steps=steps
            service=ServiceType::ParentsEnfantsMoinsSix/>
    }
}
