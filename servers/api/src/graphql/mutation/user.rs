use async_graphql::{Context, Object, Result};
use chrono::DateTime;
use entity::async_graphql::{self, InputObject};
use entity::user;
use service::Mutation;
use uuid::Uuid;

use crate::db::Database;

#[derive(InputObject)]
pub struct CreateUserInput {
    pub name: String,
    pub profile_picture: String,
}

impl CreateUserInput {
    fn into_model_with_arbitrary_id(self) -> user::Model {
        user::Model {
            id: Uuid::new_v4(),
            name: self.name,
            profile_picture: self.profile_picture,
            created_at: DateTime::default(),
            updated_at: None,
        }
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    pub async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserInput,
    ) -> Result<user::Model> {
        let db = ctx.data::<Database>().unwrap();
        let conn = db.get_connection();

        Ok(Mutation::create_user(conn, input.into_model_with_arbitrary_id()).await?)
    }
}
