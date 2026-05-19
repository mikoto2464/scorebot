use rkyv::{Archive, Deserialize, Serialize};

pub const SOCKET_NAME: &str = "scorebot.socket";

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub channel: Channel,
    pub command: Command
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub enum Channel {
    OfficialPrivateMessage { openid: String, event_id: String },
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Quit,
    Help
}