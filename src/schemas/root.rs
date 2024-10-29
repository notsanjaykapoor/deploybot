use juniper::{EmptySubscription, FieldError, FieldResult};
use ulid::Ulid;

use crate::schemas::deploy::DeployResult;
use crate::schemas::ping::Ping;

pub struct QueryRoot;

#[juniper::graphql_object]
impl QueryRoot {
    #[graphql(description = "deploy stage", name = "deploy_stage")]
    fn deploy_stage(_repo: String, _tag: String, _path: String) -> FieldResult<DeployResult> {
        let id = Ulid::new().to_string();

        Ok(DeployResult {
            code: 202,
            id: id,
        })
    }

    #[graphql(description = "gql error", name = "error")]
    fn error() -> FieldResult<Ping> {
        Err(FieldError::new(
            "gql error",
            graphql_value!({ "not_found": "gql error" })
        ))
    }

    #[graphql(description = "ping")]
    fn ping(message: String) -> FieldResult<Ping> {
        Ok(Ping{ message: message })
    }
}

pub struct MutationRoot;

#[juniper::graphql_object]
impl MutationRoot {
    fn ping() -> FieldResult<DeployResult> {
        let id = Ulid::new().to_string();

        Ok(DeployResult {
            code: 202,
            id: id,
        })
    }
}

pub type Schema = juniper::RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
