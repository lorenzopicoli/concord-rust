use std::{collections::HashMap, net::SocketAddr, sync::Arc};

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
    async fn listen(&mut self) -> Message {
        let message = self
            .rx
            .try_next()
            .await
            .expect("Failed to read message")
            .expect("Failed to open message");
        return message;
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
            dbg!(v)
                .send(msg.clone())
                .await
                .expect("Failed to send message");
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

#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("Failed to start TCP server");

    let rooms: Arc<Mutex<HashMap<String, WSRoom>>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (connection, addr) = match server.accept().await {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to connect client: {}", e);
                continue;
            }
        };
        let room_id = "room1".to_string();
        let (tx, rx) = connect(connection).await;

        {
            let mut rooms = rooms.lock().await;
            match rooms.get_mut(&room_id) {
                Some(v) => {
                    v.txs.insert(addr, tx);
                }
                None => {
                    let mut room = WSRoom {
                        txs: HashMap::new(),
                    };
                    room.txs.insert(addr, tx);
                    rooms.insert(room_id.clone(), room);
                }
            }
        };

        let mut peer = Peer {
            rx,
            addr: addr.clone(),
        };
        let rooms = rooms.clone();
        let _ = tokio::spawn(async move {
            loop {
                let new_message = peer.listen().await;
                {
                    let mut rooms = rooms.lock().await;
                    let room = rooms
                        .get_mut(&room_id.clone())
                        .expect("Failed to fetch room in thread");
                    room.broadcast(&peer.addr, new_message.clone()).await;
                    println!("Done broadcasting {}", new_message);
                }
            }
        });
    }
}
