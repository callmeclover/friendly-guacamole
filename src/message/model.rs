use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageTypes {
  MessageSent(MessageSent),
  RetrieveMessages(RetrieveMessages)
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageSent {
    pub msg: String,
    user: String
}
#[derive(Serialize, Deserialize, Debug)]
struct RetrieveMessages {
    pub msgs: Vec<MessageTypes::MessageSent>
}
