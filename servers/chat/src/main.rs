mod core;

#[tokio::main]
async fn main() {
    let host = "127.0.0.1";
    let port = "9001";
    let server = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to start TCP server");
    println!("Server listening on port {}", port);
    core::start(server).await;
}
