use anyhow::Result;
use sqlx::{query_as, query, postgres::{PgPoolOptions, PgPool}, types::Json};
use super::model::{Model, User, GlassModeration};
use uuid::Uuid;

pub struct DatabaseConnectix {
    connection: PgPool
}

impl Default for DatabaseConnectix {
    fn default() -> Result<Self> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let uri = std::env::var("DB_URL")?;
            let client = PgPoolOptions::new()
                .max_connections(5)
                .connect(&uri).await?;

            Ok(Self {
                connection: client
            })
        })
        
    }
}

impl DatabaseConnectix {
    pub fn new(uri: &str) -> Result<Self> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(uri).await?;

            Ok(Self {
                connection: pool
            })
        })            
    }

    /// Gets a possible user id (if one exists) for a username.
    pub async fn get_user_id(&mut self, username: &str) -> Result<i32> {
        let user: Option<User> = query_as(
            "select max(id) from users where username=$1 limit 1;"
        )
            .bind(username)
            .fetch_optional(&mut self.connection)
            .await?;

        if user.is_none() {
            Ok(1)
        } else {
            if user.id == 9999 { return Err("username is taken"); }
            Ok(user.id+1)
        }
    }

    pub async fn post_user(&mut self, username: String, password: String) -> Result<()> {
        let data: Model = Model {
            id: self.get_user_id(username).await?,
            uuid: Uuid::new_v4(),
            username,
            password,
            moderation_stats: Json(GlassModeration::default())
        };
        data.validate()?;
        
        let _ = sqlx::query("insert into users (id, uuid, username, password, mod) values ($1, $2, $3, $4, $5)")
            .bind(data.id).bind(data.uuid).bind(data.username).bind(data.password).bind(data.moderation_stats)
            .exectute(&mut self.connection)
            .await?;
        Ok(())
    }

    pub async fn update_user(&mut self, username: &str, prev_username: &str, prev_id: i32) -> Result<()> {
        let id = self.get_user_id(username).await?;
        
        let _ = query("update users set username=$1, id=$2 where username=$3 and id=$4")
            .bind(username).bind(id).bind(prev_username).bind(prev_id)
            .exectute(&mut self.connection)
            .await?;
        Ok(())
    }
}