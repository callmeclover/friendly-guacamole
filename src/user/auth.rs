use std::error::Error;
use postgres::{Client, NoTls};

use super::model::{Model, User};

pub struct DatabaseConnectix {
    connection: Client
}

impl Default for DatabaseConnectix {
    fn default() -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let uri = std::env::var("DB_URL").unwrap();
            let mut client = Client::connect(&uri, NoTls).expect("can't connect to database!");

            return Self {
                connection: client
            };
        })
        
    }
}

impl DatabaseConnectix {
    pub fn new(uri: &str) -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut client = Client::connect(uri, NoTls).expect("can't connect to database!");

            return Self {
                connection: client
            };
        })            
    }

    /// Gets a possible user id (if one exists) for a username.
    pub fn get_user_id(&self, name: &str) -> Result<i32, Box<dyn Error>> {
        if let Some(res) = self.connection.query_one("select max(id) from users where name=$1", &[&name]) {
            if res.id == 9999 { return Err("username is taken".into()); }
            Ok(res.id+1)
        } else {
            Ok(1)
        }
    }

    pub fn post_user(&self, user: User) {
        let data: Model = user.into();
        let Model { id, name, uuid, moderation_stats } = data;
        self.connection.execute(
            "INSERT INTO users (id, name, uuid, mod) VALUES ($1, $2, $3, $4)",
            &[&id, &name, &uuid, &moderation_stats],
        ).expect("couldn't execute post_user query!");
    }
}