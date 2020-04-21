use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::lib::deploy::DeployStage;
use crate::lib::pki::PkiCheck;
use crate::lib::fs::{FsRemove, FsTouch};

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
    sender: web::Data<crossbeam::Sender<String>>,
    item: web::Json<DeployStruct>,
) -> HttpResponse {
    // println!("model: {:?}", &item);

    let id = Ulid::new().to_string();

    let result = DeployResult {
        id: id.clone(),
    };

    match PkiCheck::new(&id).call(&item.plain_msg, &item.crypto_sign, &logger.get_ref()) {
        Err(_) => {
            return HttpResponse::Unauthorized().json(result)
        },
        Ok(_) => {}
    }

    tokio::spawn(async move {
        FsTouch::call(&id);

        let mut deploy_stage = DeployStage::new(
            id.clone(),
            item.repo.clone(),
            item.tag.clone(),
            item.path.clone(),
            logger.get_ref().clone(),
            sender.get_ref().clone(),
        );

        let _code = match deploy_stage.call().await {
            Some(code) => {
                code
            },
            None => {
                500
            }
        };

        FsRemove::call(&id);
    });

    HttpResponse::Accepted().json(result)
}
