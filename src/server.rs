use crate::{config, handler};
use crate::rate_limit::RateLimitTracker;

pub async fn start_server(responses_folder: String, config_file: &str, port: u16, rate_limiter: RateLimitTracker) {
    let endpoints = config::load_config(config_file).expect("Failed to load config");
    let routes = handler::routes(endpoints, responses_folder, rate_limiter);
    
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}