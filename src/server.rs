use crate::{config, handler};

pub async fn start_server(responses_folder: String, config_file: &str, port: u16) {
    let endpoints = config::load_config(config_file).expect("Failed to load config");
    let routes = handler::routes(endpoints, responses_folder);
    
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}