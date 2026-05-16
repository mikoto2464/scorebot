use serde::{Deserialize, Serialize};

pub const SOCKET_NAME: &str = "scorebot.socket";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub channel: Channel,
    pub command: Command
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Channel {
    OfficialPrivateMessage { openid: String, event_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Quit,
    Help
}