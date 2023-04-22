use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Serialize, Deserialize};

use super::db::get_conn;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub password: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Save user infomation to database
pub async fn create(
    username: &str,
    password: &str
) -> anyhow::Result<()> {
    let conn = get_conn().await;
    let model = ActiveModel {
        username: Set(username.into()),
        password: Set(password.into()),
        ..Default::default()
    };
    model.insert(conn).await?;

    Ok(())
}

pub async fn find(username: &str) -> anyhow::Result<Option<Model>> {
    let conn = get_conn().await;
    Entity::find()
        .filter(Column::Username.eq(username))
        .one(conn)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
