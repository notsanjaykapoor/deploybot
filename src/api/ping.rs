use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use slog::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    message: String,
}

/// this handler uses json extractor
pub async fn ping(
    logger: web::Data<slog::Logger>,
    _channel: web::Data<crossbeam_channel::Sender<String>>,
) -> HttpResponse {
    info!(logger, "ping_api_request");

    let result = PingResult {
        message: "pong".to_string()
    };

    HttpResponse::Ok().json(result)
}
