pub mod admin;
pub mod booking;
pub mod home;
pub mod not_found;
pub mod newsletter;
pub mod service;
pub mod contact;
pub mod mentions_legales;
pub mod moi_et_mon_atelier;

pub use admin::{
    AdminBookingsPage, AdminLoginPage, AdminPage, AdminServicesPage, AdminSessionsPage,
    AdminThemesPage,
};
pub use booking::BookingPage;
pub use home::HomePage;
pub use not_found::NotFound;
pub use newsletter::NewsletterPage;
pub use contact::ContactPage;
pub use mentions_legales::MentionsLegales;
pub use service::ServicePage;
pub use moi_et_mon_atelier::{
    catalogue::CataloguePage, diplomes_et_formations::DiplomesEtFormationsPage,
    qui_suis_je::QuiSuisJePage,
};
