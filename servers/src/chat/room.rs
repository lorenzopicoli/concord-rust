use std::collections::HashMap;

use tokio_tungstenite::tungstenite::Message;

use super::{message::WSMessage, WSWriteStream};
use futures::SinkExt;

#[derive(Debug)]
pub struct Session {
    pub connected_users: HashMap<uuid::Uuid, WSWriteStream>,
    pub unidentified_users: HashMap<uuid::Uuid, WSWriteStream>,
}

impl Session {
    pub fn identify_user(&mut self, temp_id: uuid::Uuid, user_id: uuid::Uuid) {
        let tx = match self.unidentified_users.remove(&temp_id) {
            Some(tx) => tx,
            None => panic!("Trying to identify wrong user"),
        };
        self.connected_users.insert(user_id, tx);
    }
    pub fn unknown_connected(&mut self, temp_id: &uuid::Uuid, tx: WSWriteStream) {
        self.unidentified_users.insert(temp_id.clone(), tx);
    }
    pub async fn broadcast(&mut self, message: WSMessage) {
        for (_k, v) in self.connected_users.iter_mut() {
            v.send(Message::Text(message.serialize().clone()))
                .await
                .expect("Failed to send message");
        }
    }
}
