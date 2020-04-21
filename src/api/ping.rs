use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    message: String,
}

/// this handler uses json extractor
pub async fn ping(
    _logger: web::Data<slog::Logger>,
    _sender: web::Data<crossbeam::Sender<String>>,
) -> HttpResponse {
    let result = PingResult {
        message: "pong".to_string()
    };

    HttpResponse::Ok().json(result)
}
