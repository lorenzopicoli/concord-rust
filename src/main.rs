use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

mod chat;

#[tokio::main]
async fn main() {
    let server = chat::WSServer::start().await;
    let rooms: chat::WSRooms = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let new_connection = match server.poll_new_peer().await {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to pool new peer {}", e);
                continue;
            }
        };
        let user_id = uuid::Uuid::new_v4();
        let room_id = "room1".to_string();

        // Locks the room mutex and inserts the new connection in the room or creates a room
        // with the connection if necessary
        {
            let mut rooms = rooms.lock().await;
            match rooms.get_mut(&room_id) {
                Some(v) => {
                    v.txs.insert(user_id, new_connection.0);
                }
                None => {
                    let mut room = chat::WSRoom {
                        txs: HashMap::new(),
                    };
                    room.txs.insert(user_id, new_connection.0);
                    rooms.insert(room_id.clone(), room);
                }
            }
        };

        let mut peer = chat::Peer {
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
                let room_id = room_id.clone();
                match new_message {
                    Some(m) => {
                        // If peer sent a new message spawn a new thread locking rooms and
                        // broadcasting the message to all other peers
                        tokio::spawn(async move {
                            let mut rooms = rooms.lock().await;
                            let room = rooms
                                .get_mut(&room_id.clone())
                                .expect("Failed to fetch room in thread");
                            room.broadcast(&peer.user_id, m.clone()).await;
                        });
                    }
                    // User disconnected
                    None => {
                        let mut rooms = rooms.lock().await;
                        let room = rooms
                            .get_mut(&room_id.clone())
                            .expect("Failed to fetch room in thread");
                        room.txs.remove(&peer.user_id);
                        println!("Peer disconnected");
                        break;
                    }
                }
            }
        });
    }
}
