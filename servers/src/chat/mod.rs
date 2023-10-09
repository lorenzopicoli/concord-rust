use std::collections::HashSet;
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

    fn mock_data(&self) -> HashMap<Uuid, Vec<Uuid>> {
        let mut servers_by_users: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let server1 = Uuid::parse_str("7f6dcb56-2c71-4921-96c1-92d9a891f626").unwrap();
        let server2 = Uuid::parse_str("737bfbcb-0a47-417c-8b2e-75a05f7942f6").unwrap();
        let server3 = Uuid::parse_str("cf7f6696-613a-4c76-96e9-68e08e757380").unwrap();

        let user1 = Uuid::parse_str("6d53e385-dba4-4a34-933f-6883e1e76cd5").unwrap();
        let user2 = Uuid::parse_str("72b8d01a-2c39-45b2-8df0-8110aeb6ca03").unwrap();
        let user3 = Uuid::parse_str("4e96e730-7b37-4233-bbd0-ee7489ebc6f0").unwrap();
        let user4 = Uuid::parse_str("2511ace5-0068-4a33-8d03-357a9a79430d").unwrap();

        servers_by_users.insert(user1, vec![server1, server2, server3]);

        servers_by_users.insert(user2, vec![server1]);

        servers_by_users.insert(user3, vec![server3, server2]);

        servers_by_users.insert(user4, vec![server1, server2]);

        servers_by_users
    }

    pub async fn start(&self) {
        let mut user_servers: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        let mut session = Session {
            connected_users: HashMap::new(),
            unidentified_users: HashMap::new(),
        };

        let servers_by_users = self.mock_data();

        println!("Starting server");
        loop {
            let (message_tx, mut message_rx) = mpsc::channel(32);
            let new_connection = match self.poll_new_peer().await {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to pool new peer {}", e);
                    continue;
                }
            };

            println!("New connection detected");

            let mut peer = peer::Peer {
                rx: new_connection.1,
                // Not identified yet
                user_id: None,
            };

            let message_tx = message_tx.clone();
            let temp_id = Uuid::new_v4();
            // Wait for new messages from this new peer in a different thread
            let _ = tokio::spawn(async move {
                println!("Starting new thread to listen to user {}", temp_id.clone());
                loop {
                    // Wait for new message
                    let new_message = peer.listen().await;
                    println!(
                        "New message received for user {:#?} - {:#?}",
                        peer.user_id, new_message
                    );
                    let message = WSMessage::new(&peer.user_id, &new_message);
                    match message {
                        WSMessage::Logout { user_id: _ } => {
                            // If message is logout, communicate it and stop
                            // listening to this peer
                            message_tx.send(message).await;
                            break;
                        }
                        _ => {
                            message_tx.send(message).await;
                        }
                    }
                }
            });

            println!("Regestering as unknown");
            session.unknown_connected(&temp_id, new_connection.0);
            println!("Unknown list {:#?}", session.unidentified_users);
            println!("Waiting for new channel message");
            'channel: while let Some(message) = message_rx.recv().await {
                println!("New channel message {:#?}", message);
                match message {
                    WSMessage::NewMessage {
                        user_id: _,
                        room_id: _,
                        message: _,
                        server_id: _,
                    } => {
                        println!("NewMessage received {:#?}", message);
                        session.broadcast(message).await;
                    }
                    WSMessage::Login { user_id } => {
                        println!("Login received {:#?}", user_id);
                        // Get from token
                        session.identify_user(temp_id, user_id);
                        // peer.user_id = Some(user_id);
                        println!("Identifying user {} as {}", temp_id, user_id);
                        let server_list = servers_by_users
                            .get(&user_id)
                            .expect("Server doens't exist");
                        for server_id in server_list {
                            match user_servers.get_mut(server_id) {
                                Some(v) => {
                                    v.insert(user_id);
                                }
                                None => {
                                    let mut set = HashSet::new();
                                    set.insert(user_id);
                                    user_servers.insert(server_id.clone(), set);
                                }
                            }
                        }
                        // Make sure to not leak jwt here in the future
                        session.broadcast(message);
                        println!("User servers {:#?}", user_servers);
                    }
                    WSMessage::Logout { user_id } => match user_id {
                        Some(user_id) => {
                            session.connected_users.remove(&user_id);
                            let mut servers_to_clear: Vec<Uuid> = Vec::new();
                            for (key, users) in user_servers.iter_mut() {
                                users.remove(&user_id);
                                if users.len() == 0 {
                                    servers_to_clear.push(key.clone());
                                }
                            }
                            for id in servers_to_clear {
                                user_servers.remove(&id);
                            }
                            println!("Peer disconnected {}", user_id);
                            break 'channel;
                        }
                        None => {
                            println!("Warning, logout without userId. Possibly hanging entry in servers map");
                            break 'channel;
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
