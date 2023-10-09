use std::{collections::HashMap, error::Error};

use futures::stream::{SplitSink, SplitStream, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use self::message::WSMessage;
use self::room::Session;

pub type WSWriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WSReadStream = SplitStream<WebSocketStream<TcpStream>>;

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
        let mut user_servers: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut session = Session {
            connected_users: HashMap::new(),
            unidentified_users: HashMap::new(),
        };

        let mut servers_by_users: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        // User 1
        servers_by_users.insert(
            Uuid::new_v4(),
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        );

        // User 2
        servers_by_users.insert(
            Uuid::new_v4(),
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        );

        // User 3
        servers_by_users.insert(
            Uuid::new_v4(),
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        );

        let (message_tx, mut message_rx) = mpsc::channel(32);
        loop {
            let new_connection = match self.poll_new_peer().await {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to pool new peer {}", e);
                    continue;
                }
            };

            let mut peer = peer::Peer {
                rx: new_connection.1,
                // Not identified yet
                user_id: None,
            };

            let message_tx = message_tx.clone();
            // Wait for new messages from this new peer in a different thread
            let _ = tokio::spawn(async move {
                loop {
                    // Wait for new message
                    let new_message = peer.listen().await;
                    let message = WSMessage::new(&peer.user_id, &new_message);
                    message_tx.send(message).await;
                }
            });
            let temp_id = Uuid::new_v4();
            session.unknown_connected(&temp_id, new_connection.0);

            while let Some(message) = message_rx.recv().await {
                match message {
                    WSMessage::NewMessage {
                        user_id: _,
                        room_id: _,
                        message: _,
                        server_id,
                    } => {
                        // If peer sent a new message spawn a new thread locking rooms and
                        // broadcasting the message to all other peers
                        let room = user_servers
                            .get_mut(&server_id)
                            .expect("Failed to fetch server");
                        // tokio::spawn(async move {
                        session.broadcast(message).await;
                        // });
                    }
                    WSMessage::Login { jwt_token } => {
                        // Get from token
                        let user_id = Uuid::new_v4();
                        session.identify_user(temp_id, user_id);
                        let server_list = servers_by_users
                            .get(&user_id)
                            .expect("Server doens't exist");
                        for server_id in server_list {
                            match user_servers.get_mut(server_id) {
                                Some(v) => {
                                    v.push(user_id);
                                }
                                None => {
                                    user_servers.insert(server_id.clone(), vec![user_id]);
                                }
                            }
                        }
                    }
                    WSMessage::Logout { user_id } => match user_id {
                        Some(user_id) => {
                            session.connected_users.remove(&user_id);
                            let mut servers_to_clear: Vec<Uuid> = Vec::new();
                            for (key, users) in user_servers.iter_mut() {
                                if let Some(index) =
                                    users.iter().position(|value| *value == user_id).clone()
                                {
                                    users.remove(index);
                                    if users.len() == 0 {
                                        servers_to_clear.push(key.clone());
                                    }
                                }
                            }
                            for id in servers_to_clear {
                                user_servers.remove(&id);
                            }
                            println!("Peer disconnected");
                            break;
                        }
                        None => {
                            println!("Warning, logout without userId. Possibly hanging entry in servers map");
                            break;
                        }
                    },
                }
            }
        }
    }

    async fn poll_new_peer(&self) -> Result<(WSWriteStream, WSReadStream), Box<dyn Error>> {
        let (stream, _addr) = self.listener.accept().await?;
        let ws = accept_async(stream).await?;
        let (tx, rx) = ws.split();

        Ok((tx, rx))
    }
}
