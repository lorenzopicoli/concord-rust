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
    NewMessage(NewMessage),
    Login(LoginMessage),
    Logout(LogoutMessage),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewMessage {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub message: String,
    pub server_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginMessage {
    pub user_id: Uuid,
    //jwt_token
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogoutMessage {
    pub user_id: Option<Uuid>,
}

impl WSMessage {
    pub fn new(message: &Option<Message>) -> Self {
        if let Some(m) = message {
            println!("New message recieved:  {}", m);
        }
        let message = match message {
            None => return WSMessage::Logout(LogoutMessage { user_id: None }),
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
            Message::Close(_) => WSMessage::Logout(LogoutMessage { user_id: None }),
            Message::Frame(_) => todo!(),
        };
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize message")
    }
}
