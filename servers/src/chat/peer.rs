use super::WSReadStream;
use futures::stream::TryStreamExt;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct Peer {
    pub user_id: Option<uuid::Uuid>,
    pub rx: WSReadStream,
}

impl Peer {
    pub async fn listen(&mut self) -> Option<Message> {
        if let Ok(message) = self.rx.try_next().await {
            message
        } else {
            None
        }
    }
}
