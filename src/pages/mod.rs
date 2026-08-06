pub mod admin;
pub mod booking;
pub mod home;
pub mod not_found;
pub mod qui_suis_je;
pub mod newsletter;
pub mod services;
pub mod mentions_legales;

pub use admin::{AdminBookingsPage, AdminLoginPage, AdminPage, AdminSessionsPage};
pub use booking::BookingPage;
pub use home::HomePage;
pub use not_found::NotFound;
pub use qui_suis_je::QuiSuisJePage;
pub use newsletter::NewsletterPage;
pub use mentions_legales::MentionsLegales;
pub use services::{
    hors_les_murs::AteliersHorsLesMurs,
    en_institution::AteliersEnInstitution,
    aperos_creatifs::AperosCreatifs,
    ateliers_pour_tous::AteliersCreatifsPourTous,
    parents_enfants::AteliersParentsEnfants,
    individuels::AteliersIndividuels,
};