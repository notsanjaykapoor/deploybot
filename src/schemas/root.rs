use juniper::{FieldError, FieldResult, RootNode};
// use slog::*;
// use std::sync::Mutex;
// use std::thread;
use ulid::Ulid;

// use crate::lib::deploy::DeployStage;
use crate::schemas::deploy::DeployResult;
use crate::schemas::ping::Ping;

pub struct Context {
    pub name: String,  // store anything in context for now
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
    #[graphql(description = "deploy stage", name = "deploy_stage")]
    fn deploy_stage(context: &Context, repo: String, tag: String, path: String) -> FieldResult<DeployResult> {
        let id = Ulid::new().to_string();

        // let logger = Logger::root(
        //     Mutex::new(slog_json::Json::default(std::io::stdout())).map(slog::Fuse),
        //     o!(),
        // );
        //
        // thread::spawn({
        //     let id = id.clone();
        //     let logger = logger.clone();
        //     let sender = None;
        //
        //     move || {
        //         let code = match DeployStage::new(id, repo, tag, path, logger, sender).call() {
        //             Some(code) => {
        //                 code
        //             },
        //             None => {
        //                 500
        //             }
        //         };
        //     }
        // });

        Ok(DeployResult {
            code: 202,
            id: id,
        })
    }


    #[graphql(description = "gql error", name = "error")]
    fn error(context: &Context) -> FieldResult<Ping> {
        Err(FieldError::new(
            "gql error",
            graphql_value!({ "not_found": "gql error" })
        ))
    }

    #[graphql(description = "ping")]
    fn ping(context: &Context, message: String) -> FieldResult<Ping> {
        Ok(Ping{ message: message })
    }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot)
}
