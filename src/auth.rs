use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::{future::LocalBoxFuture, FutureExt as _, TryFutureExt as _};

use crate::state::AppState;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let app_data = req.app_data::<actix_web::web::Data<AppState>>().clone();
        let allowed_api_key = app_data.unwrap().allowed_api_key.clone();

        let api_key = match req
            .headers()
            .get("Authorization")
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.split(":").last().map(|v| v.trim()))
            .and_then(|val| val.split(" ").last().map(|v| v.trim()))
        {
            Some(str) => str.to_string(),
            None => "".to_string(),
        };
        let path = req.path().to_string();
        if api_key != allowed_api_key && path != "/" {
            return Box::pin(async {
                Ok(req.into_response(HttpResponse::Forbidden().finish().map_into_right_body()))
            });
        }

        self.service
            .call(req)
            .map_ok(ServiceResponse::map_into_left_body)
            .boxed_local()
    }
}
