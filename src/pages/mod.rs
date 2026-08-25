pub mod admin;
pub mod booking;
pub mod home;
pub mod not_found;
pub mod newsletter;
pub mod services;
pub mod mentions_legales;
pub mod moi_et_mon_atelier;

pub use admin::{
    AdminBookingsPage, AdminLoginPage, AdminPage, AdminSessionsPage, AdminThemesPage,
};
pub use booking::BookingPage;
pub use home::HomePage;
pub use not_found::NotFound;
pub use newsletter::NewsletterPage;
pub use mentions_legales::MentionsLegales;
pub use services::{
    aperos_creatifs::AperosCreatifs,
    apres_midis_creatifs::ApresMidisCreatifs,
    parents_enfants_moins_six::AteliersParentsEnfantsMoinsSix,
    parents_enfants_six_a_douze::AteliersParentsEnfantsSixADouze,
};
pub use moi_et_mon_atelier::{
    diplomes_et_formations::DiplomesEtFormationsPage, qui_suis_je::QuiSuisJePage,
};
