pub mod admin;
pub mod booking;
pub mod home;
pub mod not_found;
pub mod newsletter;
pub mod services;
pub mod mentions_legales;
pub mod moi_et_mon_atelier;

pub use admin::{AdminBookingsPage, AdminLoginPage, AdminPage, AdminSessionsPage};
pub use booking::BookingPage;
pub use home::HomePage;
pub use not_found::NotFound;
pub use newsletter::NewsletterPage;
pub use mentions_legales::MentionsLegales;
pub use services::{
    aperos_creatifs::AperosCreatifs,
    apres_midis_creatifs::ApresMidisCreatifs,
    parents_enfants_moins_6::AteliersParentsEnfantsMoinsDe6Ans,
    parents_enfants_6_12::AteliersParentsEnfants6A12Ans,
};
pub use moi_et_mon_atelier::qui_suis_je::QuiSuisJePage;