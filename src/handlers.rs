use actix_web;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use crate::schemas::root::{create_schema, Schema};

#[actix_web::route("/graphql", method = "GET", method = "POST")]
pub async fn graphql(
    schema: actix_web::web::Data<Schema>,
    data: actix_web::web::Json<GraphQLRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let res = data.execute(&schema, &()).await;

    Ok(actix_web::HttpResponse::Ok().json(res))
}

/// GraphiQL UI
#[actix_web::get("/graphiql")]
async fn graphql_playground() -> impl actix_web::Responder {
    actix_web::web::Html::new(graphiql_source("/graphql", None))
}

pub fn register(config: &mut actix_web::web::ServiceConfig) {
    config
        .app_data(actix_web::web::Data::new(create_schema()))
        .service(graphql)
        .service(graphql_playground);
}
