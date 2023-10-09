use std::collections::HashMap;

use tokio_tungstenite::tungstenite::Message;

use super::{message::WSMessage, WSWriteStream};
use futures::SinkExt;

#[derive(Debug)]
pub struct WSRoom {
    pub txs: HashMap<uuid::Uuid, WSWriteStream>,
}

impl WSRoom {
    pub async fn broadcast(&mut self, message: WSMessage) {
        let message = Message::text(
            "asdasd", // serde_json::to_string(&message).expect("Failed to construct message"),
        );

        for (_k, v) in self.txs.iter_mut() {
            v.send(message.clone())
                .await
                .expect("Failed to send message");
        }
    }
}
