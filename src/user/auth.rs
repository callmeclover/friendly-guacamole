use std::{error::Error, future::Future};
use sea_orm::{*, prelude::*};
use crate::user::model::{Entity as ModelEntity, Column as ModelColumn};

pub struct DatabaseConnectix {
    connection: DatabaseConnection
}

impl Default for DatabaseConnectix {
    fn default() -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let uri = std::env::var("DB_URL").unwrap();
            let db: DatabaseConnection = Database::connect(uri).await.expect("couldn't connect to database!");

            return Self {
                connection: db
            };
        })
        
    }
}

impl DatabaseConnectix {
    pub fn new(uri: &str) -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let db: DatabaseConnection = Database::connect(uri).await.expect("couldn't connect to database!");
        
            return Self {
                connection: db
            };
        })            
    }

    /// Gets a possible user id (if one exists) for a username.
    pub async fn get_user_id(&self, name: &str) -> Result<i32, Box<dyn Error>> {
        if let Some(res) = ModelEntity::find().expr(Expr::col("id").max()).filter(ModelColumn::Name.contains(name)).one(&self.connection).await? {
            if res.id == 9999 { return Err("username is taken".into()); }
            Ok(res.id+1)
        } else {
            Ok(1)
        }
    }
}