pub mod bookings;
pub mod dashboard;
pub mod login;
pub mod sessions;
pub mod shell;
pub mod themes;

pub use bookings::AdminBookingsPage;
pub use dashboard::AdminPage;
pub use login::AdminLoginPage;
pub use sessions::AdminSessionsPage;
pub use shell::AdminShell;
pub use themes::AdminThemesPage;

/// Starts an executor so that rendering a component owning a `Resource` works.
///
/// Test-only: outside tests the browser or the server provides one. Failing means
/// another test got there first, which is exactly what we want.
#[cfg(all(test, feature = "ssr"))]
pub(crate) fn init_test_executor() {
    let _ = any_spawner::Executor::init_futures_executor();
}
