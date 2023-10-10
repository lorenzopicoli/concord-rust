use std::collections::{HashMap, HashSet};

use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use super::{
    message::{LoginMessage, LogoutMessage, NewMessage, WSMessage},
    WSWriteStream,
};
use futures::SinkExt;
pub fn mock_data() -> HashMap<Uuid, Vec<Uuid>> {
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

#[derive(Debug)]
pub struct WSManager {
    /// Map keys are connection ids, map to (user_id, tx)
    pub connected_users: HashMap<uuid::Uuid, (Option<uuid::Uuid>, WSWriteStream)>,
    /// Map keys are server ids mapping to connected users. Use HashSet in case we receive
    /// two login requets from same user
    pub user_servers: HashMap<Uuid, HashSet<Uuid>>,
}

impl WSManager {
    pub fn new() -> Self {
        let session = WSManager {
            connected_users: HashMap::new(),
            user_servers: HashMap::new(),
        };
        session
    }
    pub fn identify_user(&mut self, connection_id: &uuid::Uuid, user_id: uuid::Uuid) {
        self.connected_users.get_mut(&connection_id).unwrap().0 = Some(user_id);
    }
    pub fn unknown_connected(&mut self, connection_id: &uuid::Uuid, tx: WSWriteStream) {
        self.connected_users
            .insert(connection_id.clone(), (None, tx));
    }

    pub async fn handle_new_message(&mut self, message: NewMessage) {
        println!("NewMessage received {:#?}", message);
        self.broadcast(WSMessage::NewMessage(message)).await;
    }

    pub async fn handle_login_message(
        &mut self,
        connection_id: &uuid::Uuid,
        message: LoginMessage,
    ) {
        let servers_by_users = mock_data();
        println!("Login received {:#?}", message.user_id);
        // Get from token
        println!("Identifying user {} as {}", connection_id, message.user_id);
        self.identify_user(connection_id, message.user_id);

        let server_list = servers_by_users
            .get(&message.user_id)
            .expect("Server doens't exist");
        for server_id in server_list {
            match self.user_servers.get_mut(server_id) {
                Some(v) => {
                    v.insert(message.user_id);
                }
                None => {
                    let mut set = HashSet::new();
                    set.insert(message.user_id);
                    self.user_servers.insert(server_id.clone(), set);
                }
            }
        }
        // Make sure to not leak jwt here in the future
        self.broadcast(WSMessage::Login(message)).await;
        println!("User servers {:#?}", self.user_servers);
    }

    pub async fn handle_logout_message(
        &mut self,
        connection_id: &uuid::Uuid,
        // Shouldn't mut
        mut message: LogoutMessage,
    ) {
        let dropped_connection = self.connected_users.remove(&connection_id);
        // If connection was known and user was identified, drop from
        // connection list
        if let Some(dropped_connection) = dropped_connection {
            if let Some(user_id) = dropped_connection.0 {
                message.user_id = Some(user_id.clone());
                let mut servers_to_clear: Vec<Uuid> = Vec::new();
                for (key, users) in self.user_servers.iter_mut() {
                    users.remove(&user_id);
                    if users.len() == 0 {
                        servers_to_clear.push(key.clone());
                    }
                }
                for id in servers_to_clear {
                    self.user_servers.remove(&id);
                }
            }
        }
        println!("Peer disconnected {}", connection_id);
        self.broadcast(WSMessage::Logout(message)).await;
    }

    async fn broadcast(&mut self, message: WSMessage) {
        for (_k, v) in self.connected_users.iter_mut() {
            // Don't broadcast to unindentified connections
            if v.0.is_none() {
                continue;
            }
            println!("Sending {:#?} to {:#?}", message, v.0);
            v.1.send(Message::Text(message.serialize().clone()))
                .await
                .expect("Failed to send message");
        }
    }
}
