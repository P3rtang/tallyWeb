#![allow(unused_imports)]

use std::io::Write;
use std::process::Command;
use std::thread;

use leptos::*;
use frontend::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use actix_web::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use actix_web::http::StatusCode;
        use actix_web::HttpRequest;

        #[tokio::main]
        async fn main() -> Result<(), AppError> {
            let conf = get_configuration(None).await.unwrap();
            let addr = conf.leptos_options.site_addr;

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|| view! { <app::App/> });

            let pool = backend::create_pool().await.map_err(|err| AppError::DbConnection(err.to_string()))?;
            let _ = backend::migrate(&pool).await.map_err(|err| println!("{err}"));

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                App::new()
                    .wrap(actix_web::middleware::Condition::new(conf.leptos_options.env == leptos_config::Env::PROD, middleware::Compress::default()))
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    .service(privacy_policy)
                    .service(Files::new("/pkg", format!("{site_root}/pkg")))
                    .service(Files::new("/fa", format!("{site_root}/font_awesome")))
                    .service(Files::new("/icons", format!("{site_root}/icons")))
                    // // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes(
                        leptos_options.to_owned(),
                        routes.to_owned(),
                        || view! { <app::App/> },
                    )
                    .app_data(web::Data::new(leptos_options.to_owned()))
                    .app_data(web::Data::new(pool.clone()))
            })

            .bind(&addr)
            .map_err(|err| AppError::ActixError(err.to_string()))?
            .run()
            .await
            .map_err(|err| AppError::ActixError(err.to_string()))?;

            return Ok(())
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
