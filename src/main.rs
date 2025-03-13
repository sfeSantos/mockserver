mod config;
mod handler;
mod server;

#[tokio::main]
async fn main() {
    println!("Starting server on http://localhost:8080");
    server::start_server().await;
}
