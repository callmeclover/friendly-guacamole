use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    pub user: String,
    #[serde(skip_serializing)]
    pub time: DateTime<Utc>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetrieveMessages {
    pub msgs: Vec<MessageSent>
}

// User related messages
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserJoin {
    pub userjoin: String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLeft {
    pub userleft: String
}