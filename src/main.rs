use std::sync::Arc;
use std::sync::Mutex;
use std::env;

use actix_web::HttpResponse;
use actix_web::{get, post, web, App, HttpServer, Responder};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

struct AppState {
    model: Arc<Mutex<SentenceEmbeddingsModel>>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello from pro-text-vectorizer")
}

#[post("/predict")]
async fn predict(data: web::Data<AppState>, req_body: String) -> impl Responder {
    let model = data.model.lock().unwrap();
    let embeddings = model.encode(&[req_body]);
    match embeddings {
        Ok(embeddings) => HttpResponse::Ok().json(embeddings[0].clone()),
        Err(_) => HttpResponse::InternalServerError().body("error generating embeddings"),
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

    let app_data = web::Data::new(AppState {
        model: Arc::new(Mutex::new(model)),
    });

    let port = env::var("PORT").unwrap_or("8080".to_string());
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());

    let bind_addr = vec![host, port].join(":");

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(index)
            .service(predict)
    })
    .bind(bind_addr)?
    .run()
    .await
}
