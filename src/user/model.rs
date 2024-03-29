use rustrict::Context;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct User {
    pub context: Context,
}