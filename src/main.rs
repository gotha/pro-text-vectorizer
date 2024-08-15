use std::env;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::{get, post, web, App, HttpServer, Responder};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use serde::{Deserialize, Serialize};

mod auth;
mod logging;
mod state;

const APP_NAME: &str = "text-vectorizer";

#[derive(Serialize, Deserialize)]
struct EmbeddingsRequest {
    text: String,
}
impl Default for EmbeddingsRequest {
    fn default() -> EmbeddingsRequest {
        EmbeddingsRequest {
            text: "".to_string(),
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello from ".to_string() + &APP_NAME.to_string())
}

#[post("/predict")]
async fn predict(
    data: web::Data<state::AppState>,
    req_body: String,
    req: HttpRequest,
) -> impl Responder {
    let content_type = req
        .headers()
        .get("Content-Type")
        .and_then(|val| val.to_str().ok())
        .unwrap_or("text/plain")
        .to_string();

    let text: String = match content_type.as_str() {
        "application/json" => {
            let data: EmbeddingsRequest =
                serde_json::from_str(&req_body.to_string()).unwrap_or(EmbeddingsRequest::default());

            data.text.to_string()
        }
        _ => req_body.to_string(),
    };
    if text == "" {
        return HttpResponse::InternalServerError().body("empty or malformed request body");
    }

    let res = web::block(move || {
        let model = data.model.lock().unwrap();
        model.encode(&[text])
    })
    .await;

    if res.is_err() {
        return HttpResponse::InternalServerError().body("error generating embeddings");
    }

    match res.unwrap() {
        Ok(prediction) => HttpResponse::Ok().json(prediction[0].clone()),
        Err(_) => HttpResponse::InternalServerError().body("error getting prediction"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let model = tokio::task::spawn_blocking(|| {
        SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
            .create_model()
            .expect("Failed to load model")
    })
    .await
    .expect("task failed to complete");

    let port = env::var("PORT").unwrap_or("8080".to_string());
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let system_code = env::var("SYSTEM_CODE").unwrap_or(APP_NAME.to_string());
    let allowed_api_key = env::var("API_KEY").unwrap_or("".to_string());

    let app_data = web::Data::new(state::AppState {
        model: Arc::new(Mutex::new(model)),
        system_code,
        allowed_api_key,
    });

    let bind_addr = vec![host, port].join(":");

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(logging::Logger)
            .wrap(auth::Auth)
            .service(index)
            .service(predict)
    })
    .bind(bind_addr)?
    .run()
    .await
}
