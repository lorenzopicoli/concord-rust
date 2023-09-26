use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use futures::{
    stream::{SplitSink, SplitStream, StreamExt, TryStreamExt},
    SinkExt,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

pub type WSWriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WSReadStream = SplitStream<WebSocketStream<TcpStream>>;
pub type WSRooms = Arc<Mutex<HashMap<String, WSRoom>>>;

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum WSMessageType {
    Message = 0,
    Identification,
    EnterRoom,
    LeaveRoom,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WSMessage {
    #[serde(rename = "type")]
    pub kind: WSMessageType,
    pub data: String,
}

#[derive(Debug)]
pub struct Peer {
    pub user_id: uuid::Uuid,
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

#[derive(Debug)]
pub struct WSRoom {
    pub txs: HashMap<uuid::Uuid, WSWriteStream>,
}

impl WSRoom {
    pub async fn broadcast(&mut self, sender: &uuid::Uuid, msg: Message) {
        for (k, v) in self.txs.iter_mut() {
            if *k == *sender {
                continue;
            }
            let message = WSMessage {
                kind: WSMessageType::Message,
                data: msg.clone().to_string(),
            };

            let message = Message::text(
                serde_json::to_string(&message).expect("Failed to construct message"),
            );

            v.send(message).await.expect("Failed to send message");
        }
    }
}

#[derive(Debug)]
pub struct WSServer {
    listener: TcpListener,
}

pub struct NewConnection {
    pub tx: WSWriteStream,
    pub rx: WSReadStream,
    pub addr: SocketAddr,
}

impl WSServer {
    pub async fn start() -> Self {
        let host = "127.0.0.1";
        let port = "9001";
        let server = TcpListener::bind(format!("{}:{}", host, port))
            .await
            .expect("Failed to start TCP server");
        println!("Server listening on port {}", port);

        WSServer { listener: server }
    }

    pub async fn poll_new_peer(&self) -> Result<(WSWriteStream, WSReadStream), Box<dyn Error>> {
        let (stream, _addr) = self.listener.accept().await?;
        let ws = accept_async(stream).await?;
        let (tx, rx) = ws.split();

        Ok((tx, rx))
    }
}
