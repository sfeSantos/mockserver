use crate::{config, handler};

pub async fn start_server() {
    let endpoints = config::load_config().expect("Failed to load config");
    let routes = handler::routes(endpoints);
    
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}