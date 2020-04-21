#[macro_use]
extern crate juniper;
extern crate serde_json;
extern crate signal_hook;
extern crate tokio;

use actix_web::{middleware, web, App, HttpServer};
use crossbeam_channel::unbounded;
use dotenv;
use slog::*;
use std::process;
use std::sync::Mutex;

use crate::api::deploys::deploys_create;
use crate::api::ping::ping;
use crate::handlers::register;
use crate::lib::slack::SlackThread;

mod api;
mod handlers;
mod lib;
mod schemas;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info,info");
    env_logger::init();

    // register sigint handler
    let _signal = unsafe { signal_hook::register(signal_hook::SIGINT, || process::abort()) }?;

    let listen_address = dotenv::var("LISTEN_ADDRESS").unwrap();  // e.g. 0.0.0.0:80

    // app data logger
    let logger = Logger::root(
        Mutex::new(slog_json::Json::default(std::io::stdout())).map(slog::Fuse),
        o!(),
    );

    // create channels for sending and receiving messages
    let (sender, receiver) = unbounded::<String>();

    // app data objects
    let app_logger = web::Data::new(logger.clone());
    let app_sender = web::Data::new(sender.clone());

    // create slack thread
    tokio::spawn(async move {
        SlackThread::new(receiver.clone(), logger.clone()).call().await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_logger.clone())
            .app_data(app_sender.clone())
            .data(web::JsonConfig::default().limit(4096))
            .configure(register)
            .wrap(middleware::Logger::default())
            // register handlers
            .service(web::resource("/api/v1/deploys").route(web::post().to(deploys_create)))
            .service(web::resource("/ping").route(web::get().to(ping)))
            .default_service(web::to(|| async { "404" }))
    })
    .bind(listen_address)?
    .run()
    .await
}
