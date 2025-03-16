extern crate core;

mod config;
mod handler;
mod server;
mod authentication;

#[tokio::main]
async fn main() {
    println!("Starting server on http://localhost:8080");
    server::start_server().await;
}
