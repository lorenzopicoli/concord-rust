use futures::stream::TryStreamExt;

use futures::stream::{SplitSink, SplitStream, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use self::manager::WSManager;
use self::message::WSMessage;

pub type WSWriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WSReadStream = SplitStream<WebSocketStream<TcpStream>>;

mod manager;
mod message;

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

async fn manage_ws_messages(mut channel_rx: mpsc::Receiver<MpscCommand>) {
    let mut session = WSManager::new();

    while let Some(channel_message) = channel_rx.recv().await {
        println!("New channel message");
        match channel_message {
            MpscCommand::NewConnection(connection_id, tx) => {
                println!("Regestering as unknown");
                session.unknown_connected(&connection_id, tx);
                println!("Connected users {:#?}", session.connected_users);
            }
            MpscCommand::WSMessage(connection_id, message) => {
                let message = WSMessage::new(&message);
                match message {
                    WSMessage::NewMessage(message) => {
                        session.handle_new_message(message).await;
                    }
                    WSMessage::Login(message) => {
                        session.handle_login_message(&connection_id, message).await;
                    }
                    WSMessage::Logout(message) => {
                        session.handle_logout_message(&connection_id, message).await;
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
        println!("New message received from peer");

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
