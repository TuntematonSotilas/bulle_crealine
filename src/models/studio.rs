/// Who answers, where, and how to reach her.
///
/// One value rather than constants scattered across modules: the legal notice is
/// obliged to carry all of it, the contact page shows all of it, and two copies of a
/// phone number drift the first time one of them changes.
pub struct Studio {
    /// The name the legal notice names as editor and as publication director.
    pub owner: &'static str,
    /// Where the workshops open to booking are held, and the registered address.
    ///
    /// A constant rather than a field on a workshop: every bookable one runs here,
    /// and one run for a structure takes place at the structure's. A second venue
    /// would make it a field again.
    pub address: &'static str,
    /// The same address on a map, for whoever would rather be guided than read.
    pub map_url: &'static str,
    /// Spaced as it is dialled, and as it is written on a card. See
    /// [`Studio::phone_link`] for the form a browser needs.
    pub phone: &'static str,
    pub email: &'static str,
    /// Not registered yet, and the page says so rather than leaving the line out: a
    /// mention marked as pending is a reminder, where a missing one is merely
    /// missing. Filling this in is the whole of what it takes to finish the page.
    pub siret: &'static str,
    /// Undecided. Either a VAT number, or the words saying there is none -- "TVA non
    /// applicable, article 293 B du CGI" -- but never nothing.
    pub vat: &'static str,
}

impl Studio {
    /// The phone number as an `href`.
    ///
    /// Derived rather than stored beside the spaced form: two spellings of one number
    /// are two things to keep in step, and the day they disagree the one nobody reads
    /// is the one that dials.
    ///
    /// International form, `+33` replacing the trunk zero: a French browser dials
    /// either, a roaming one only this.
    pub fn phone_link(&self) -> String {
        let digits: String = self.phone.chars().filter(char::is_ascii_digit).collect();

        match digits.strip_prefix('0') {
            Some(national) => format!("tel:+33{national}"),
            // Anything not starting with a trunk zero is passed through: it is
            // already written for somewhere else, and guessing a country would be
            // worse than saying nothing.
            None => format!("tel:{digits}"),
        }
    }

    /// The email address as an `href`.
    pub fn email_link(&self) -> String {
        format!("mailto:{}", self.email)
    }
}

/// Where this site answers, with no trailing slash.
///
/// A constant rather than an environment variable, because [`crate::app`] is compiled
/// twice -- once for the server and once for WASM -- and `std::env::var` on
/// `wasm32-unknown-unknown` compiles happily and then always answers `Err`. The two
/// builds would disagree about every canonical URL, and hydration would patch a
/// `<head>` the server had just got right. One value both builds read is the only
/// shape that cannot drift.
///
/// The day a domain of its own is attached, this line is what changes -- the
/// canonicals, the `og:url` and the sitemap all read it through [`site_url`].
pub const SITE_ORIGIN: &str = "https://bulle-crealine.onrender.com";

/// An absolute URL for a path rooted at the site.
///
/// `path` starts with `/`, so `site_url("/")` gives the origin with its trailing
/// slash, which is what the home page's canonical has to be.
pub fn site_url(path: &str) -> String {
    format!("{SITE_ORIGIN}{path}")
}

/// The one studio this site is about.
pub const OWNER: Studio = Studio {
    owner: "Coraline Batault",
    address: "Bulle Créaline (E.I), 5 Rue Marc Seguin, 42110 Feurs",
    map_url: "https://maps.app.goo.gl/Fgmpg9RF8HiPGrkf7",
    phone: "06 61 50 25 96",
    email: "bulle.crealine@gmail.com",
    siret: "[À COMPLÉTER]",
    vat: "[À COMPLÉTER]",
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The spaced form is what a visitor reads; this is what the browser dials. A
    /// second stored spelling would be a second thing to keep in step.
    #[test]
    fn the_phone_number_dials_in_international_form() {
        assert_eq!(OWNER.phone_link(), "tel:+33661502596");
    }

    /// Spacing is the reader's business, not the dialler's.
    #[test]
    fn however_the_number_is_spaced_it_dials_the_same() {
        for spelling in ["06 61 50 25 96", "06.61.50.25.96", "0661502596"] {
            let studio = Studio { phone: spelling, ..OWNER };

            assert_eq!(studio.phone_link(), "tel:+33661502596", "{spelling}");
        }
    }

    /// A number written for somewhere else is passed through rather than given a
    /// French country code it never asked for.
    #[test]
    fn a_number_without_a_trunk_zero_keeps_its_own_country() {
        let studio = Studio { phone: "+32 2 123 45 67", ..OWNER };

        assert_eq!(studio.phone_link(), "tel:3221234567");
    }

    #[test]
    fn the_email_opens_a_message() {
        assert_eq!(OWNER.email_link(), "mailto:bulle.crealine@gmail.com");
    }

    /// Every caller joins the origin to a path that already starts with a slash. An
    /// origin carrying one of its own would double it, and two spellings of one URL
    /// is the whole thing a canonical exists to prevent.
    #[test]
    fn an_address_is_joined_with_exactly_one_slash() {
        assert!(!SITE_ORIGIN.ends_with('/'), "{SITE_ORIGIN} should not end in a slash");
        assert_eq!(site_url("/"), format!("{SITE_ORIGIN}/"));
        assert_eq!(site_url("/contact"), format!("{SITE_ORIGIN}/contact"));
    }
}
