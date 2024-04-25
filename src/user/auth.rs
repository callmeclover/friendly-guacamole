use std::error::Error;
use postgres::{Client, NoTls};

use super::model::Model;

pub struct DatabaseConnectix {
    connection: Client
}

impl Default for DatabaseConnectix {
    fn default() -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let uri = std::env::var("DB_URL").unwrap();
            let mut client = Client::connect(&uri, NoTls)?;

            return Self {
                connection: client
            };
        })
        
    }
}

impl DatabaseConnectix {
    pub fn new(uri: &str) -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut client = Client::connect(uri, NoTls)?;

            return Self {
                connection: client
            };
        })            
    }

    /// Gets a possible user id (if one exists) for a username.
    pub async fn get_user_id(&self, name: &str) -> Result<i32, Box<dyn Error>> {
        todo!("am eepy must finish tomorrow");
        /*if let Some(res) = ModelEntity::find().expr(Expr::col("id").max()).filter(ModelColumn::Name.contains(name)).one(&self.connection).await? {
            if res.id == 9999 { return Err("username is taken".into()); }
            Ok(res.id+1)
        } else {
            Ok(1)
        }*/
    }
}