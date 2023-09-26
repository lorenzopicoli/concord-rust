use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use futures::{
    stream::{SplitSink, SplitStream, StreamExt, TryStreamExt},
    SinkExt,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type WSWriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;
type WSReadStream = SplitStream<WebSocketStream<TcpStream>>;

#[derive(Debug)]
struct Peer {
    addr: SocketAddr,
    rx: WSReadStream,
}

impl Peer {
    async fn listen(&mut self) -> Option<Message> {
        if let Ok(message) = self.rx.try_next().await {
            message
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct WSRoom {
    txs: HashMap<SocketAddr, WSWriteStream>,
}

impl WSRoom {
    async fn broadcast(&mut self, sender: &SocketAddr, msg: Message) {
        for (k, v) in self.txs.iter_mut() {
            if *k == *sender {
                continue;
            }
            v.send(msg.clone()).await.expect("Failed to send message");
        }
    }
}

async fn connect(tcp_stream: TcpStream) -> (WSWriteStream, WSReadStream) {
    let ws = match accept_async(tcp_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            panic!("Failed to accept websocket connection: {}", e);
        }
    };
    ws.split()
}

async fn start_server() -> TcpListener {
    let host = "127.0.0.1";
    let port = "9001";
    let server = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to start TCP server");
    println!("Server listening on port {}", port);

    server
}

struct NewConnection {
    tx: WSWriteStream,
    rx: WSReadStream,
    addr: SocketAddr,
}

async fn poll_new_peer(server: &TcpListener) -> Result<NewConnection, Box<dyn Error>> {
    let (connection, addr) = server.accept().await?;
    let (tx, rx) = connect(connection).await;

    Ok(NewConnection { tx, rx, addr })
}

#[tokio::main]
async fn main() {
    let server = start_server().await;
    let rooms: Arc<Mutex<HashMap<String, WSRoom>>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let new_connection = poll_new_peer(&server).await.expect("BLA");

        let room_id = "room1".to_string();
        {
            let mut rooms = rooms.lock().await;
            match rooms.get_mut(&room_id) {
                Some(v) => {
                    v.txs.insert(new_connection.addr, new_connection.tx);
                }
                None => {
                    let mut room = WSRoom {
                        txs: HashMap::new(),
                    };
                    room.txs.insert(new_connection.addr, new_connection.tx);
                    rooms.insert(room_id.clone(), room);
                }
            }
        };

        let mut peer = Peer {
            rx: new_connection.rx,
            addr: new_connection.addr.clone(),
        };
        let rooms = rooms.clone();

        let _ = tokio::spawn(async move {
            loop {
                let new_message = peer.listen().await;
                let rooms = rooms.clone();
                let room_id = room_id.clone();
                match new_message {
                    Some(m) => {
                        tokio::spawn(async move {
                            let mut rooms = rooms.lock().await;
                            let room = rooms
                                .get_mut(&room_id.clone())
                                .expect("Failed to fetch room in thread");
                            room.broadcast(&peer.addr, m.clone()).await;
                        });
                    }
                    None => {
                        println!("Disconnect");
                    }
                }
            }
        });
    }
}
