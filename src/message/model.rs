use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageTypes {
  MessageSent(MessageSent),
  RetrieveMessages(RetrieveMessages),
  UserJoin(UserJoin),
  UserLeft(UserLeft)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageSent {
    pub msg: String,
    user: String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetrieveMessages {
    pub msgs: Vec<MessageTypes>
}

// User related messages
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserJoin {
    pub user: String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLeft {
    pub user: String
}