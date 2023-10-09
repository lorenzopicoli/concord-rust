use tokio::net::TcpListener;

mod chat;

#[tokio::main]
async fn main() {
    let host = "127.0.0.1";
    let port = "9001";
    let server = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to start TCP server");
    println!("Server listening on port {}", port);
    chat::start(server).await;
}
