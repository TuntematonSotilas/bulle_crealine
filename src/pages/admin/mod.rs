pub mod bookings;
pub mod dashboard;
pub mod login;
pub mod qrcode;
pub mod services;
pub mod sessions;
pub mod shell;
pub mod themes;

pub use bookings::AdminBookingsPage;
pub use dashboard::AdminPage;
pub use login::AdminLoginPage;
pub use qrcode::AdminQrCodePage;
pub use services::AdminServicesPage;
pub use sessions::AdminSessionsPage;
pub use shell::AdminShell;
pub use themes::AdminThemesPage;

/// Identity of a row in an admin table: its whole content.
///
/// Every admin table is a keyed `<For>` over this rather than a plain `Vec` of
/// views. A `Vec` is diffed by position, so when a filter or a reload shifted the
/// rows, each kept its old place's button handlers -- the "Modifier" or "Supprimer"
/// of the third row then acted on whichever row used to be third. Keyed, a row is
/// either kept exactly as built or built afresh.
///
/// The whole content rather than the id, so that an edited row is rebuilt too: it
/// would otherwise keep showing, and handing to its buttons, what it held before.
pub(crate) fn row_key<T: std::fmt::Debug>(row: &T) -> String {
    format!("{row:?}")
}

/// Starts an executor so that rendering a component owning a `Resource` works.
///
/// Test-only: outside tests the browser or the server provides one. Failing means
/// another test got there first, which is exactly what we want.
#[cfg(all(test, feature = "ssr"))]
pub(crate) fn init_test_executor() {
    let _ = any_spawner::Executor::init_futures_executor();
}

#[cfg(test)]
mod tests {
    use super::row_key;

    // Rows stand in as `(id, name)` tuples: all `row_key` reads is `Debug`.
    #[test]
    fn rows_with_the_same_content_share_a_key() {
        assert_eq!(row_key(&("1", "A")), row_key(&("1", "A")));
    }

    #[test]
    fn another_row_or_an_edited_one_gets_a_key_of_its_own() {
        let row = row_key(&("1", "A"));

        assert_ne!(row, row_key(&("2", "A")), "another row");
        assert_ne!(row, row_key(&("1", "B")), "the same row, edited");
    }
}
