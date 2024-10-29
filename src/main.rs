#[macro_use]
extern crate juniper;
extern crate serde_json;
extern crate signal_hook;

use actix_web::{middleware, web, App, HttpServer};
use crossbeam_channel::{bounded, unbounded};
use dotenv;
use slog::*;
use std::process;
use std::sync::Mutex;
use std::thread;

use crate::api::deploys::deploys_create;
use crate::api::ping::ping;
use crate::handlers::register;
use crate::lib::deploy::DeployThread;
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
    let _signal = unsafe { signal_hook::low_level::register(signal_hook::consts::SIGINT, || process::abort()) }?;

    let listen_address = dotenv::var("LISTEN_ADDRESS").unwrap();  // e.g. 0.0.0.0:80

    // create logger
    let logger = Logger::root(
        Mutex::new(slog_json::Json::default(std::io::stdout())).map(slog::Fuse),
        o!(),
    );

    // create channels for sending and receiving messages
    let (deploy_sender, deploy_receiver) = bounded::<String>(1);
    let (slack_sender, slack_receiver) = unbounded::<String>();

    // create app data objects
    let app_logger = web::Data::new(logger.clone());
    let app_channel = web::Data::new(deploy_sender.clone());

    // create deploy thread
    thread::spawn({
        let deploy_channel = deploy_receiver.clone();
        let slack_channel = slack_sender.clone();
        let logger = logger.clone();

        move || {
            DeployThread::new(
                deploy_channel,
                slack_channel,
                logger,
            ).call();
        }
    });

    // create slack thread
    thread::spawn({
        let slack_channel = slack_receiver.clone();
        let logger = logger.clone();

        move || {
            SlackThread::new(
                slack_channel,
                logger,
            ).call();
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_logger.clone())
            .app_data(app_channel.clone())
            .app_data(web::JsonConfig::default().limit(4096))
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
