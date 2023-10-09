use std::collections::HashMap;

use tokio_tungstenite::tungstenite::Message;

use super::{message::WSMessage, WSWriteStream};
use futures::SinkExt;
//     let message = WSMessage::new(&message);
//     match message {
//         WSMessage::NewMessage {
//             user_id: _,
//             room_id: _,
//             message: _,
//             server_id: _,
//         } => {
//             println!("NewMessage received {:#?}", message);
//             session.broadcast(message).await;
//         }
//         WSMessage::Login { user_id } => {
//             println!("Login received {:#?}", user_id);
//             // Get from token
//             session.identify_user(connection_id, user_id);
//             // peer.user_id = Some(user_id);
//             println!("Identifying user {} as {}", connection_id, user_id);
//             let server_list = servers_by_users
//                 .get(&user_id)
//                 .expect("Server doens't exist");
//             for server_id in server_list {
//                 match user_servers.get_mut(server_id) {
//                     Some(v) => {
//                         v.insert(user_id);
//                     }
//                     None => {
//                         let mut set = HashSet::new();
//                         set.insert(user_id);
//                         user_servers.insert(server_id.clone(), set);
//                     }
//                 }
//             }
//             // Make sure to not leak jwt here in the future
//             session.broadcast(message).await;
//             println!("User servers {:#?}", user_servers);
//         }
//         WSMessage::Logout { user_id: _ } => {
//             let dropped_connection = session.connected_users.remove(&connection_id);
//             // If connection was known and user was identified, drop from
//             // connection list
//             if let Some(dropped_connection) = dropped_connection {
//                 if let Some(user_id) = dropped_connection.0 {
//                     let mut servers_to_clear: Vec<Uuid> = Vec::new();
//                     for (key, users) in user_servers.iter_mut() {
//                         users.remove(&user_id);
//                         if users.len() == 0 {
//                             servers_to_clear.push(key.clone());
//                         }
//                     }
//                     for id in servers_to_clear {
//                         user_servers.remove(&id);
//                     }
//                 }
//             }
//             println!("Peer disconnected {}", connection_id);
//         }
//     }
// }

#[derive(Debug)]
pub struct Session {
    pub connected_users: HashMap<uuid::Uuid, (Option<uuid::Uuid>, WSWriteStream)>,
    pub unidentified_users: HashMap<uuid::Uuid, WSWriteStream>,
}

impl Session {
    pub fn identify_user(&mut self, connection_id: uuid::Uuid, user_id: uuid::Uuid) {
        self.connected_users.get_mut(&connection_id).unwrap().0 = Some(user_id);
    }
    pub fn unknown_connected(&mut self, connection_id: &uuid::Uuid, tx: WSWriteStream) {
        self.connected_users
            .insert(connection_id.clone(), (None, tx));
    }

    pub async fn handle_new_message(&self, message: WSMessage::NewMessage) {
        println!("NewMessage received {:#?}", message);
        session.broadcast(message).await;
    }

    pub async fn broadcast(&mut self, message: WSMessage) {
        for (_k, v) in self.connected_users.iter_mut() {
            // Don't broadcast to unindentified connections
            if v.0.is_none() {
                continue;
            }
            v.1.send(Message::Text(message.serialize().clone()))
                .await
                .expect("Failed to send message");
        }
    }
}
