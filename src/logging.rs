use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use serde::Serialize;
use std::time::Instant;

use crate::state::AppState;

#[derive(Serialize)]
struct LogEntry {
    method: String,
    path: String,
    response_time: u128,
    system_code: String,
}

pub struct Logger;

impl<S, B> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddleware { service }))
    }
}

pub struct LoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
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
        //println!("Hi from start. You requested: {}", req.path());

        let method = req.method().to_string();
        let path = req.path().to_string();
        let start = Instant::now();
        let app_data = req.app_data::<actix_web::web::Data<AppState>>().clone();
        let system_code = app_data.unwrap().system_code.clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            let response_time = start.elapsed().as_millis();

            let log_entry = LogEntry {
                method,
                path,
                response_time,
                system_code,
            };
            println!("{}", serde_json::to_string(&log_entry).unwrap());

            Ok(res)
        })
    }
}
