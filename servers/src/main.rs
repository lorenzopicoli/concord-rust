mod chat;

#[tokio::main]
async fn main() {
    let server = chat::WSServer::new().await;
    server.start().await;
}
