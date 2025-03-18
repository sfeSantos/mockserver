extern crate core;

use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use mockserver::server;

#[derive(Parser, Debug)]
#[command(version, about="Mockserver")]
struct Args {
    #[arg(short, long, default_value = "responses")]
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

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    server::start_server(responses_folder, config_file.as_str(), port).await;
}
