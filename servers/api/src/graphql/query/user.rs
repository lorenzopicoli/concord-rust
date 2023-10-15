use async_graphql::{Context, Object, Result};
use entity::{async_graphql, user};
use service::Query;
use uuid::Uuid;

use crate::db::Database;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    async fn get_users(&self, ctx: &Context<'_>) -> Result<Vec<user::Model>> {
        let db = ctx.data::<Database>().unwrap();
        let conn = db.get_connection();

        Ok(Query::get_all_users(conn)
            .await
            .map_err(|e| e.to_string())?)
    }

    async fn get_user_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<user::Model>> {
        let db = ctx.data::<Database>().unwrap();
        let conn = db.get_connection();

        Ok(Query::find_user_by_id(conn, id)
            .await
            .map_err(|e| e.to_string())?)
    }
}
