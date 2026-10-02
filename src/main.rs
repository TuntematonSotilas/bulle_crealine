#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::prelude::*;
    use leptos::config::get_configuration;
    use leptos_meta::{HashedStylesheet, MetaTags};
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use bulle_crealine::app::*;
    use bulle_crealine::auth::{config::AdminConfig, middleware::admin_guard};
    use bulle_crealine::cache::{cache_headers, set_bundle_is_stamped};
    use bulle_crealine::media;

    let mut conf = get_configuration(None).unwrap();

    // The image renames the bundle after a digest of its contents, so the name the
    // configuration carries is only the one the build started from. Reading the real
    // one off the disk -- before anything is served -- is what keeps the two from
    // ever disagreeing. An empty name is a build that did not go through
    // cargo-leptos, where nothing was stamped and a year of caching would be a
    // year of stale WASM.
    let shipped = stamped_bundle(&conf.leptos_options);
    let built_as = conf.leptos_options.output_name.clone();
    set_bundle_is_stamped(!built_as.is_empty() && *shipped != *built_as);
    conf.leptos_options.output_name = shipped;

    let addr = conf.leptos_options.site_addr;

    // The admin area is optional: without a usable configuration, the public site
    // starts as usual and `/admin` simply stays out of reach.
    match AdminConfig::init() {
        Ok(config) => println!("admin area enabled for {}", config.email),
        Err(error) => eprintln!(
            "admin area disabled: {error}\n  \
             set ADMIN_EMAIL, ADMIN_PASSWORD_HASH and ADMIN_SESSION_SECRET to enable it"
        ),
    }

    // Storage is optional too: the pages that need it report as much rather than
    // keeping the whole site from booting. Connecting here surfaces a bad URI at
    // startup instead of on a visitor's first booking, and creates the indexes.
    match bulle_crealine::db::init().await {
        Ok(_) => println!("connected to mongodb"),
        Err(error) => eprintln!(
            "bookings disabled: {error}\n  \
             set MONGODB_URI to enable them"
        ),
    }

    HttpServer::new(move || {
        // Generate the list of routes in your Leptos App
        let routes = generate_route_list(App);
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone().to_string();

        println!("listening on http://{}", &addr);

        App::new()
            // turn away unauthenticated /admin pages before any rendering
            .wrap(middleware::from_fn(admin_guard))
            // say out loud how long each kind of response may be reused; outermost
            // of the two, so the guard's redirect is covered as well
            .wrap(middleware::from_fn(cache_headers))
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", &site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            // serve theme photos, which live in Mongo rather than on disk
            .route("/media/theme/{id}", web::get().to(media::theme_photo))
            // the same, for the photos of a workshop run for a structure
            .route("/media/service-photo/{id}", web::get().to(media::service_photo))
            // photo uploads do not fit under actix's 256 kB default body limit
            .app_data(web::PayloadConfig::new(media::MAX_BODY_BYTES))
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    view! {
                        <!DOCTYPE html>
                        <html lang="fr">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone() />
                                <HydrationScripts options=leptos_options.clone()/>
                                // Named from the same `output_name` as the scripts
                                // above, so the stylesheet follows the bundle when
                                // the image stamps it. A hard-coded href would go on
                                // asking for a file that no longer exists.
                                <HashedStylesheet options=leptos_options.clone() id="leptos"/>
                                <MetaTags/>
                                <script>
                                    // Set the initial theme mode before the app loads to prevent flashes
                                    (function() {
                                        const stored = localStorage.getItem("darkmode");
                                        const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
                                        const darkMode = stored !== null ? JSON.parse(stored) : prefersDark;
                                        if (darkMode) {
                                            document.documentElement.classList.add("dark");
                                        }
                                    })();
                                </script>
                            </head>
                            <body>
                                <App/>
                            </body>
                        </html>
                    }
                }
            })
            .app_data(web::Data::new(leptos_options.to_owned()))
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

/// Finds the name the bundle actually ships under.
///
/// The image renames `bulle_crealine.js` to `bulle_crealine.<digest>.js`, so that a
/// deploy changes the URL and no browser can go on running yesterday's WASM. Reading
/// the name off the disk rather than out of the environment means the two can never
/// disagree -- and an image where the rename did not happen boots just the same,
/// serving the unstamped name.
///
/// Panicking here is deliberate, and happens before the port is bound: a bad image is
/// then a failed deploy, and the host keeps the previous one serving. The alternative
/// is a site answering every request with an unstyled page that never hydrates.
#[cfg(feature = "ssr")]
fn stamped_bundle(options: &leptos::config::LeptosOptions) -> std::sync::Arc<str> {
    use std::path::PathBuf;

    let pkg = PathBuf::from(options.site_root.to_string())
        .join(options.site_pkg_dir.to_string());

    let mut names: Vec<String> = std::fs::read_dir(&pkg)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", pkg.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_suffix(".js"))
                .map(str::to_owned)
        })
        .collect();
    names.sort();

    let [name] = names.as_slice() else {
        panic!(
            "{} should hold exactly one .js file, the bundle; it holds {}: {names:?}",
            pkg.display(),
            names.len()
        )
    };

    // The three travel together, or the page breaks in a way no visitor can report
    // usefully: scripts that answer 404, or a page with no styling at all.
    for extension in ["wasm", "css"] {
        let sibling = pkg.join(format!("{name}.{extension}"));
        assert!(
            sibling.exists(),
            "the bundle is incomplete: {} is missing beside {name}.js",
            sibling.display()
        );
    }

    println!("serving the bundle as {name}");

    name.as_str().into()
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::config::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use bulle_crealine::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
