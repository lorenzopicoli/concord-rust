use ::entity::{user, user::Entity as User};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_user(db: &DbConn, form_data: user::Model) -> Result<user::Model, DbErr> {
        let active_model = user::ActiveModel {
            name: Set(form_data.name.to_owned()),
            profile_picture: Set(form_data.profile_picture.to_owned()),
            ..Default::default()
        };
        println!("HERERRR");
        let res = match User::insert(active_model).exec(db).await {
            Ok(v) => v,
            Err(e) => panic!("HMMM {:#}", e),
        };

        println!("{:#?}", res);

        Ok(user::Model {
            id: res.last_insert_id,
            ..form_data
        })
    }
}
