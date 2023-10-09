use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(
    tag = "type",
    content = "data",
    rename_all_fields = "camelCase",
    rename_all = "camelCase"
)]
pub enum WSMessage {
    NewMessage {
        user_id: Uuid,
        room_id: Uuid,
        message: String,
        server_id: Uuid,
    },
    Login {
        jwt_token: String,
    },
    Logout {
        user_id: Option<Uuid>,
    },
}

impl WSMessage {
    pub fn new(user_id: &Option<Uuid>, message: &Option<Message>) -> Self {
        let user_id = user_id.clone();
        let message = match message {
            None => return WSMessage::Logout { user_id },
            Some(t) => t,
        };

        return match message {
            Message::Text(t) => {
                let message: WSMessage = serde_json::from_str(t).expect("Failed to parse message");
                message
            }
            Message::Binary(_) => todo!(),
            Message::Ping(_) => todo!(),
            Message::Pong(_) => todo!(),
            Message::Close(_) => WSMessage::Logout { user_id },
            Message::Frame(_) => todo!(),
        };
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize message")
    }
}
