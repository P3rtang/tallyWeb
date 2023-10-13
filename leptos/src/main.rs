#![allow(unused_imports)]
use leptos::*;
use tally_web::app::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use actix_web::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use actix_web::http::StatusCode;
        use actix_web::HttpRequest;

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            let conf = get_configuration(None).await.unwrap();
            let addr = conf.leptos_options.site_addr;
            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|| view! { <App/> });

            backend::migrate().await.unwrap();

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                App::new()
                    .wrap(middleware::Compress::default())
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    .service(privacy_policy)
                    // serve JS/WASM/CSS from `pkg`
                    .service(Files::new("/pkg", format!("{site_root}/pkg")))
                    // serve other assets from the `assets` directory
                    .service(Files::new("/assets", site_root))
                    .service(Files::new("/fa", format!("{site_root}/font_awesome")))
                    // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes(
                        leptos_options.to_owned(),
                        routes.to_owned(),
                        || view! { <App/> },
                    )
                    .app_data(web::Data::new(leptos_options.to_owned()))
                    .service(Files::new("/", site_root))
            })
            .bind(&addr)?
            .run()
            .await
        }

        #[actix_web::get("/favicon.svg")]
        async fn favicon(
            leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
        ) -> actix_web::Result<actix_files::NamedFile> {
            let leptos_options = leptos_options.into_inner();
            let site_root = &leptos_options.site_root;
            Ok(actix_files::NamedFile::open_async(format!(
                "{site_root}/tallyGo.svg"
            )).await?)
        }

        #[actix_web::get("/privacy-policy")]
        async fn privacy_policy(
            leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
        ) -> impl Responder {
            let leptos_options = leptos_options.into_inner();
            let site_root = &leptos_options.site_root;
            actix_files::NamedFile::open_async(format!("{site_root}/tallyWeb-privacy-policy.html")).await
        }
    } else {
        fn main() {
        }
    }
}
