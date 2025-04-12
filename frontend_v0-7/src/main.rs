#![allow(unused_imports)]
// TODO: prevent login, edit... as a username

use dev::{Service, ServiceRequest};
use dotenvy::var;
use std::io::Write;
use std::process::Command;
use std::thread;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use tallyweb_frontend_v0_7::{app, AppError, middleware as mw};
        use actix_web::*;
        use leptos::{prelude::*, config};
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use leptos_meta::MetaTags;
        use actix_web::http::StatusCode;
        use actix_web::HttpRequest;

        #[tokio::main]
        async fn main() -> std::result::Result<(), AppError> {
            let mut conf = config::get_configuration(Some("./Cargo.toml")).unwrap();
            let addr = conf.leptos_options.site_addr;

            if let Ok(env) = var("APP_ENVIRONMENT") {
                conf.leptos_options.env = env.as_str().into();
            }

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|| view! { <app::App/> });

            let pool = backend::create_pool().await.map_err(|err| AppError::DbConnection(err.to_string()))?;
            let _ = sqlx::migrate!("../migrations").run(&pool).await.map_err(|err| println!("{err}"));

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                App::new()
                    .wrap(actix_web::middleware::Condition::new(conf.leptos_options.env == config::Env::PROD, middleware::Compress::default()))
                    .service(
                        web::scope("/api")
                            .service(web::scope("/session").wrap(mw::CheckSession).route("/{tail:.*}", leptos_actix::handle_server_fns()))
                            .service(web::scope("/session_v2").wrap(mw::CheckSessionV2).route("/{tail:.*}", leptos_actix::handle_server_fns()))
                            .route("/{tail:.*}", leptos_actix::handle_server_fns())
                    )
                    .service(
                        web::scope("/login")
                            .wrap_fn(move |req, srv| {
                                let session_cookie = req.cookie("session");
                                let fut = srv.call(req);
                                async move {
                                    let mut r = fut.await.unwrap();
                                    if let Some(c) = session_cookie {
                                        r.response_mut().add_removal_cookie(&c).unwrap();
                                    };
                                    Ok(r)
                                }
                            })
                            .route("", {
                                let leptos_options = leptos_options.clone();
                                leptos_actix::render_app_async_with_context(
                                    || (),
                                    move || app::shell(leptos_options.clone()),
                                    Default::default(),
                                )
                            })
                    )
                    .service(privacy_policy)
                    .service(Files::new("/pkg", format!("{site_root}/pkg")))
                    .service(Files::new("/fa", format!("{site_root}/font_awesome")))
                    .service(Files::new("/icons", format!("{site_root}/icons")))
                    // // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes(
                        routes.to_owned(), {
                            let leptos_options = leptos_options.clone();
                            move || app::shell(leptos_options.clone())
                        },
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
            leptos_options: actix_web::web::Data<config::LeptosOptions>,
        ) -> actix_web::Result<actix_files::NamedFile> {
            let leptos_options = leptos_options.into_inner();
            let site_root = &leptos_options.site_root;
            Ok(actix_files::NamedFile::open_async(format!(
                "{site_root}/tallyGo.svg"
            )).await?)
        }

        #[actix_web::get("/privacy-policy.html")]
        async fn privacy_policy(
            leptos_options: actix_web::web::Data<config::LeptosOptions>,
        ) -> impl Responder {
            let leptos_options = leptos_options.into_inner();
            let site_root = &leptos_options.site_root;
            actix_files::NamedFile::open_async(format!("{site_root}/privacy-policy.html")).await
        }
    } else {
        fn main() {
        }
    }
}
