use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;

use super::UserSession;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct CheckSession;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for CheckSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckSessionMW<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckSessionMW { service }))
    }
}

pub struct CheckSessionMW<S> {
    service: S,
}

impl<S, B> CheckSessionMW<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
}

impl<S, B> Service<ServiceRequest> for CheckSessionMW<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let get_session = |req: &ServiceRequest| {
            let cookie = req
                .cookie("session")
                .ok_or(actix_web::error::ErrorUnauthorized(
                    "Missing `session` cookie",
                ))?
                .value()
                .to_string();

            let pool = req
                .app_data::<actix_web::web::Data<backend::PgPool>>()
                .ok_or(actix_web::error::ErrorInternalServerError(
                    "Missing DB pool",
                ))?;

            let session: UserSession = serde_json::from_str(&cookie)
                .map_err(|err| actix_web::error::ErrorBadRequest(err))?;

            Ok((session.clone(), pool.clone().into_inner().clone()))
        };

        let (session, pool) = match get_session(&req) {
            Ok(res) => res,
            Err(err) => return Box::pin(async { Err(err) }),
        };

        let fut = self.service.call(req);

        Box::pin(async move {
            match backend::auth::check_user(&pool, &session.username, session.token).await {
                Ok(backend::auth::SessionState::Valid) => fut.await,
                Ok(backend::auth::SessionState::Expired) => {
                    let (req, resp) = fut.await?.into_parts();
                    let resp = HttpResponse::Ok()
                        .insert_header(("serverfnredirect", "/login"))
                        .insert_header((header::LOCATION, "/login"))
                        .message_body(resp.into_body())?;
                    Ok(ServiceResponse::new(req, resp))
                }
                Err(err) => Err(actix_web::error::ErrorUnauthorized(err)),
            }
        })
    }
}
