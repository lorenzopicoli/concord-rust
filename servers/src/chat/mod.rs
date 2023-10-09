use futures::stream::TryStreamExt;
use std::collections::HashSet;
use std::{collections::HashMap, error::Error};

use futures::stream::{SplitSink, SplitStream, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
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
enum MpscCommand {
    NewConnection(Uuid, WSWriteStream),
    WSMessage(Uuid, Option<Message>),
}

pub async fn start(listener: TcpListener) {
    let (message_tx, message_rx) = mpsc::channel::<MpscCommand>(32);
    println!("Starting server");
    println!("Waiting for new channel message");
    tokio::spawn(poll_new_peers(listener, message_tx));
    manage_ws_messages(message_rx).await;
}

fn mock_data() -> HashMap<Uuid, Vec<Uuid>> {
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

async fn handleNewMessage() {
    println!("NewMessage received {:#?}", message);
    session.broadcast(message).await;
}

async fn manage_ws_messages(mut channel_rx: mpsc::Receiver<MpscCommand>) {
    let servers_by_users = mock_data();
    let mut user_servers: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
    let mut session = Session {
        connected_users: HashMap::new(),
        unidentified_users: HashMap::new(),
    };
    while let Some(channel_message) = channel_rx.recv().await {
        println!("New channel message {:#?}", channel_message);
        match channel_message {
            MpscCommand::NewConnection(connection_id, tx) => {
                println!("Regestering as unknown");
                session.unknown_connected(&connection_id, tx);
                println!("Unknown list {:#?}", session.unidentified_users);
            }
            MpscCommand::WSMessage(connection_id, message) => {
                let message = WSMessage::new(&message);
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
                        session.identify_user(connection_id, user_id);
                        // peer.user_id = Some(user_id);
                        println!("Identifying user {} as {}", connection_id, user_id);
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
                        session.broadcast(message).await;
                        println!("User servers {:#?}", user_servers);
                    }
                    WSMessage::Logout { user_id: _ } => {
                        let dropped_connection = session.connected_users.remove(&connection_id);
                        // If connection was known and user was identified, drop from
                        // connection list
                        if let Some(dropped_connection) = dropped_connection {
                            if let Some(user_id) = dropped_connection.0 {
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
                            }
                        }
                        println!("Peer disconnected {}", connection_id);
                    }
                }
            }
        }
    }
}

async fn poll_new_peers(listener: TcpListener, channel_tx: mpsc::Sender<MpscCommand>) {
    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to pool new peer {}", e);
                continue;
            }
        };
        let ws = match accept_async(stream).await {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to accept connection {}", e);
                continue;
            }
        };
        let (ws_tx, ws_rx) = ws.split();
        println!("New connection detected");
        let connection_id = Uuid::new_v4();
        if let Err(e) = channel_tx
            .send(MpscCommand::NewConnection(connection_id, ws_tx))
            .await
        {
            println!(
                "Failed to commucate connection through channel. Dropping connection {}",
                e
            );
            continue;
        }

        println!("Starting new thread to listen to user");
        // Wait for new messages from this new peer in a different thread
        let _ = tokio::spawn(listen_peer(connection_id, ws_rx, channel_tx.clone()));
    }
}

async fn listen_peer(
    connection_id: Uuid,
    mut ws_rx: WSReadStream,
    channel_tx: mpsc::Sender<MpscCommand>,
) {
    loop {
        // Wait for new message
        let message = match ws_rx.try_next().await {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to read message {}. Disconnecting user", e);
                break;
            }
        };
        println!("New message received {:#?}", message);

        if let Err(e) = channel_tx
            .send(MpscCommand::WSMessage(connection_id, message.clone()))
            .await
        {
            println!(
                "Failed to commucate message through channel. Message is possibly lost {}",
                e
            );
            continue;
        }
        // Is none then we have disconnected
        if message.is_none() {
            break;
        }
    }
}
