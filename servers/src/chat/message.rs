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
        user_id: Uuid,
    },
    Logout {
        user_id: Uuid,
    },
}

impl WSMessage {
    pub fn new(user_id: &Uuid, message: &Option<Message>) -> Self {
        let user_id = user_id.clone();
        let message = match message {
            None => return WSMessage::Logout { user_id },
            Some(t) => t,
        };

        return match message {
            Message::Text(t) => {
                let s = serde_json::to_string(&WSMessage::NewMessage {
                    user_id: Uuid::new_v4(),
                    room_id: Uuid::new_v4(),
                    message: "dsfsfsd".to_string(),
                    server_id: Uuid::new_v4(),
                })
                .expect("adasd");
                println!("{}", s);
                let s = serde_json::to_string(&WSMessage::Login {
                    user_id: Uuid::new_v4(),
                })
                .expect("adasd");
                println!("{}", s);
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
