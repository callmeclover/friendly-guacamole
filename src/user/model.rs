use rustrict::Context;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct User {
    pub context: Context,
}