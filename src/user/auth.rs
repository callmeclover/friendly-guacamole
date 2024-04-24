use lazy_static::lazy_static;
use std::error::Error;
use sea_orm::*;
use crate::user::model::{Model, Entity as ModelEntity, Column as ModelColumn};

pub struct DatabaseConnectix {
    connection: DatabaseConnection
}

impl Default for DatabaseConnectix {
    pub async fn default() -> Self {
        let uri = std::env::var("DB_URL").unwrap();
        let db: DatabaseConnection = Database::connect(uri).await?;

        return Self {
            connection: db
        };
    }
}

impl DatabaseConnectix {
    pub async fn new(uri: &str) -> Self {
            let db: DatabaseConnection = Database::connect(uri).await?;

            return Self {
                connection: db
            };
    }

    /// Gets a possible user id (if one exists) for a username.
    pub async fn get_user_id(&self, name: &str) -> Result<i32, Box<dyn Error>> {
        if let Some(res) = ModelEntity::find().expr(Expr::col("id").max()).filter(ModelColumn::Name.eq(name)).one(self.connection).await? {
            if res.id == 9999 { return Box::new(Err("username is taken")); }
            Ok(res.id+1)
        } else {
            Ok(0001)
        }
    }
}