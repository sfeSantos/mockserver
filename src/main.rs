extern crate core;

use clap::Parser;

mod config;
mod handler;
mod server;
mod authentication;

#[derive(Parser, Debug)]
#[command(version, about="Mockserver")]
struct Args {
    #[arg(short, long, default_value = "responses/")]
    responses_folder: String,
    #[arg(short, long, default_value = "config.yaml")]
    file: String,
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let responses_folder = args.responses_folder;
    let config_file = args.file;
    let port= args.port;

    println!("Starting server on http://localhost:{}", port);
    server::start_server(responses_folder, config_file.as_str(), port).await;
}
