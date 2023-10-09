use std::{collections::HashMap, error::Error, sync::Arc};

use futures::stream::{SplitSink, SplitStream, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use self::message::WSMessage;

pub type WSWriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WSReadStream = SplitStream<WebSocketStream<TcpStream>>;
pub type WSRooms = Arc<Mutex<HashMap<Uuid, room::WSRoom>>>;

mod message;
mod peer;
mod room;

#[derive(Debug)]
pub struct WSServer {
    pub listener: TcpListener,
}

impl WSServer {
    pub async fn new() -> Self {
        let host = "127.0.0.1";
        let port = "9001";
        let server = TcpListener::bind(format!("{}:{}", host, port))
            .await
            .expect("Failed to start TCP server");
        println!("Server listening on port {}", port);

        WSServer { listener: server }
    }

    pub async fn start(&self) {
        let rooms: WSRooms = Arc::new(Mutex::new(HashMap::new()));

        loop {
            let new_connection = match self.poll_new_peer().await {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to pool new peer {}", e);
                    continue;
                }
            };
            let user_id = uuid::Uuid::new_v4();

            // Locks the room mutex and inserts the new connection in the room or creates a room
            // with the connection if necessary
            // {
            //     let mut rooms = rooms.lock().await;
            //     match rooms.get_mut(&room_id) {
            //         Some(v) => {
            //             v.txs.insert(user_id, new_connection.0);
            //         }
            //         None => {
            //             let mut room = room::WSRoom {
            //                 txs: HashMap::new(),
            //             };
            //             room.txs.insert(user_id, new_connection.0);
            //             rooms.insert(room_id.clone(), room);
            //         }
            //     }
            // };

            let mut peer = peer::Peer {
                rx: new_connection.1,
                user_id: user_id.clone(),
            };
            let rooms = rooms.clone();
            // Wait for new messages from this new peer in a different thread
            let _ = tokio::spawn(async move {
                loop {
                    // Wait for new message
                    let new_message = peer.listen().await;
                    let rooms = rooms.clone();
                    let message = WSMessage::new(&user_id, &new_message);
                    match message {
                        WSMessage::NewMessage {
                            user_id: _,
                            room_id,
                            message: _,
                            server_id: _,
                        } => {
                            // If peer sent a new message spawn a new thread locking rooms and
                            // broadcasting the message to all other peers
                            tokio::spawn(async move {
                                let mut rooms = rooms.lock().await;
                                let room = rooms
                                    .get_mut(&room_id)
                                    .expect("Failed to fetch room in thread");
                                room.broadcast(message).await;
                            });
                        }
                        WSMessage::Login { user_id: _ } => todo!(),
                        WSMessage::Logout { user_id: id } => {
                            let mut rooms = rooms.lock().await;
                            for (_key, room) in rooms.iter_mut() {
                                room.txs.remove(&id);
                            }
                            println!("Peer disconnected");
                            break;
                        }
                    }
                }
            });
        }
    }

    async fn poll_new_peer(&self) -> Result<(WSWriteStream, WSReadStream), Box<dyn Error>> {
        let (stream, _addr) = self.listener.accept().await?;
        let ws = accept_async(stream).await?;
        let (tx, rx) = ws.split();

        Ok((tx, rx))
    }
}
