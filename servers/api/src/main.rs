use std::net::SocketAddr;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::*;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
struct Query;

type ServiceSchema = Schema<Query, EmptyMutation, EmptySubscription>;
#[Object]
impl Query {
    async fn howdy(&self) -> &'static str {
        "partner"
    }
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/playground", get(graphql_playground))
        .layer(Extension(schema));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/ws"),
    ))
}

async fn graphql_handler(schema: Extension<ServiceSchema>, req: GraphQLRequest) -> GraphQLResponse {
    let response = schema.execute(req.into_inner()).await;

    response.into()
}
