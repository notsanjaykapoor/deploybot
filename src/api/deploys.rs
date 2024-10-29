use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::lib::deploy::DeployMessage;
use crate::lib::pki::PkiCheck;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployStruct {
    repo: String,
    tag: String,
    path: String,
    plain_msg: String,
    crypto_sign: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployResult {
    id: String,
}

/// this handler uses json extractor
pub async fn deploys_create(
    logger: web::Data<slog::Logger>,
    channel: web::Data<crossbeam_channel::Sender<String>>,
    item: web::Json<DeployStruct>,
) -> HttpResponse {

    let result = DeployResult {
        id: Ulid::new().to_string(),
    };

    match PkiCheck::new(&result.id).call(&item.plain_msg, &item.crypto_sign, &logger.get_ref()) {
        Err(_) => {
            return HttpResponse::Unauthorized().json(result)
        },
        Ok(_) => {}
    }

    if channel.is_full() {
        return HttpResponse::TooManyRequests().json(result)
    }

    // create deploy message and send to thread using channel

    let deploy_message = DeployMessage {
        id: result.id.clone(),
        repo: item.repo.clone(),
        tag: item.tag.clone(),
        path: item.path.clone(),
    };

    channel.send(serde_json::to_string(&deploy_message).unwrap()).unwrap();

    HttpResponse::Accepted().json(result)
}
