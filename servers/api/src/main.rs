use std::net::SocketAddr;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::*;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use graphql::schema::ApiSchema;
// struct Query;
mod db;
mod graphql;

#[tokio::main]
async fn main() {
    let schema = graphql::schema::build_schema().await;
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/playground", get(graphql_playground))
        .with_state(schema);
    // .layer(Extension(schema));
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

async fn graphql_handler(schema: State<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
    let response = schema.execute(req.into_inner()).await;

    response.into()
}
