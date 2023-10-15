use async_graphql::{EmptySubscription, Schema};
use entity::async_graphql;

use crate::{db::Database, graphql::mutation::Mutation, graphql::query::Query};

pub type ApiSchema = Schema<Query, Mutation, EmptySubscription>;

/// Builds the GraphQL Schema, attaching the Database to the context
pub async fn build_schema() -> ApiSchema {
    let db = Database::new().await;

    // Migrator::up(db.get_connection(), None).await.unwrap();

    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .finish()
}
