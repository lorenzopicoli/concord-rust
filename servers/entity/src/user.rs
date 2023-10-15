use async_graphql::*;
use chrono::{DateTime, FixedOffset, Local};
use sea_orm::{entity::prelude::*, DeleteMany, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(concrete(name = "User", params()))]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub name: String,
    pub profile_picture: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let dt = Local::now();
        let naive_utc = dt.naive_utc();
        let offset = dt.offset().clone();
        let dt_new = DateTime::<FixedOffset>::from_naive_utc_and_offset(naive_utc, offset);

        println!("New called");
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(dt_new),
            ..ActiveModelTrait::default()
        }
    }
}

impl Entity {
    pub fn find_by_id(id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(id))
    }

    pub fn find_by_name(name: &str) -> Select<Entity> {
        Self::find().filter(Column::Name.eq(name))
    }

    pub fn delete_by_id(id: i32) -> DeleteMany<Entity> {
        Self::delete_many().filter(Column::Id.eq(id))
    }
}
