use actix_web::{dev::Payload, error::PayloadError, web::Bytes};
use futures::StreamExt;

use super::*;

#[derive(Debug, Clone, serde::Deserialize)]
struct QueryParams {
    session: UserSession,
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct CheckSessionV2;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for CheckSessionV2
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Middleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware {
            service: std::rc::Rc::new(service),
        }))
    }
}

pub struct Middleware<S> {
    service: std::rc::Rc<S>,
}

impl<S, B> Middleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
}

impl<S, B> Service<ServiceRequest> for Middleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let pool = req
            .app_data::<actix_web::web::Data<backend::PgPool>>()
            .ok_or(actix_web::error::ErrorInternalServerError(
                "Missing DB pool",
            ))
            .map(|data| data.get_ref().clone());

        let service = std::rc::Rc::clone(&self.service);

        Box::pin(async move {
            let mut tx = pool?
                .begin()
                .await
                .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

            // Extract the request body without consuming the original
            let (request, payload) = req.into_parts();

            // Read and clone the payload bytes
            let body_bytes = actix_web::web::BytesMut::new();
            let mut body = actix_web::dev::Payload::from(payload);
            let mut body_bytes = actix_web::web::BytesMut::from(&body_bytes[..]);

            while let Some(chunk) = body.next().await {
                let chunk = chunk?;
                body_bytes.extend_from_slice(&chunk);
            }

            // Clone the bytes for logging/processing
            let cloned_bytes = body_bytes.clone().freeze();

            let query_params: QueryParams = serde_qs::from_bytes(&cloned_bytes).map_err(|_| {
                actix_web::error::ErrorUnauthorized("Unable to parse session token")
            })?;

            // Reconstruct the request with the original payload
            let payload: Payload = actix_web::dev::Payload::Stream {
                payload: Box::pin(futures::stream::once(async move {
                    Ok::<Bytes, PayloadError>(body_bytes.freeze())
                })),
            };

            let service_req = ServiceRequest::from_parts(request, payload);
            let fut = service.call(service_req);

            let ret = match backend::auth::check_user(
                &mut tx,
                &query_params.session.username,
                query_params.session.token,
            )
            .await
            {
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
            };

            tx.commit()
                .await
                .map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))?;

            ret
        })
    }
}
