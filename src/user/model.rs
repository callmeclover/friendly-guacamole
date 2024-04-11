use rustrict::Context;

#[derive(Clone)]
pub struct User {
    pub context: Context,
    pub mut name: String,
    pub id: i32
}