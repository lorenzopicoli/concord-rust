use ::entity::{user, user::Entity as User};
use sea_orm::*;
use uuid::Uuid;

pub struct Query;

impl Query {
    pub async fn find_user_by_id(db: &DbConn, id: Uuid) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    pub async fn get_all_users(db: &DbConn) -> Result<Vec<user::Model>, DbErr> {
        User::find().all(db).await
    }

    /// If ok, returns (user models, num pages).
    pub async fn find_users_in_page(
        db: &DbConn,
        page: u64,
        users_per_page: u64,
    ) -> Result<(Vec<user::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = User::find()
            .order_by_asc(user::Column::Id)
            .paginate(db, users_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated users
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
