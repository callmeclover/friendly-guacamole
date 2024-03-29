use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MessageType {
    MessageSent {},
    RetrieveMessages,
    RoomJoin,
    RoomLeave,
    UserJoin,
    UserLeave,
    ChangeUserData // Implement this later!
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum MessageTypes {
  MessageSent(MessageSent),
  RetrieveMessages(RetrieveMessages)
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageSent {
    msg: String,
    user: String
}
#[derive(Serialize, Deserialize, Debug)]
struct RetrieveMessages {
    msgs: VecString
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageModel {
    // Message typing so the client and server know what they should do with results
    #[serde(rename="type")]
    pub msgtype: MessageType,

    // Make params with aliases for each message type
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias="msg")]
    pub param1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias="user")]
    pub param2: Option<String>,
}