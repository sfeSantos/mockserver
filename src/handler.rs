use crate::config::Endpoint;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::fs as async_fs;
use tokio::time::sleep;
use tracing::info;
use warp::{Filter, Rejection, Reply};
use warp::http::header::AUTHORIZATION;
use warp::http::Response;
use warp::hyper::Body;
use warp::reject::custom;
use crate::authentication::{validate_auth, Unauthorized};
use crate::rate_limit::{check_rate_limit, RateLimitTracker, RateLimited};

pub fn routes(
    endpoints: HashMap<String, Endpoint>,
    responses_folder: String,
    rate_limiter: RateLimitTracker,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let endpoints = warp::any().map(move || endpoints.clone());
    let responses_folder = warp::any().map(move || responses_folder.clone());
    let rate_limiter = warp::any().map(move || rate_limiter.clone());

    //TODO allow cors be passed via configuration file
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allow_headers(vec!["Content-Type", "Authorization", "Accept"])
        .build();

    warp::path::full()
        .and(warp::method())
        .and(warp::header::optional::<String>(AUTHORIZATION.as_str()))
        .and(warp::body::bytes())
        .and(endpoints)
        .and(responses_folder)
        .and(rate_limiter)
        .and_then(process_request)
        .recover(handle_rejection)
        .with(cors)
}

/// Handles the rate limit and processes the request
async fn process_request(
    path: warp::path::FullPath,
    method: warp::http::Method,
    auth_header: Option<String>,
    body: bytes::Bytes,
    endpoints: HashMap<String, Endpoint>,
    responses_folder: String,
    rate_limiter: RateLimitTracker,
) -> Result<impl Reply, Rejection> {
    let path_str = path.as_str().to_string();

    if let Some(endpoint) = endpoints.get(&path_str) {
        check_rate_limit(path_str.clone(), method.as_str(), endpoint.rate_limit.as_ref(), rate_limiter.clone()).await?;
    }

    handle_request(path, method, auth_header, body, endpoints, responses_folder).await
}

pub async fn handle_request(
    path: warp::path::FullPath,
    method: warp::http::Method,
    auth_header: Option<String>,
    body: bytes::Bytes,
    endpoints: HashMap<String, Endpoint>,
    responses_folder: String,
) -> Result<impl Reply, Rejection> {
    info!("Received request: {} {}", method, path.as_str());

    if let Some(endpoint) = endpoints.get(path.as_str()) {
        if let Some(auth) = &endpoint.authentication {
            if !validate_auth(auth, auth_header) {
                info!("❌ Unauthorized access attempt to {}", path.as_str());
                return Err(custom(Unauthorized));
            }
        }

        add_possible_delay(endpoint).await;
        
        let method_str = method.as_str();
        let status_code = default_status_code(endpoint, method_str);

        return if endpoint.method.iter().any(|m| m == method_str) {
            match method_str {
                "GET" => {
                    let file_path = format!("{}/{}", responses_folder, endpoint.file);
                    info!("📂 Fetching file from: {}", file_path);
                    if let Ok(contents) = fs::read_to_string(&file_path) {
                        Ok(Response::builder()
                            .status(status_code)
                            .header("Content-Type", "application/json")
                            .body(contents)
                            .unwrap())
                    } else {
                        info!("🚫 File not found: {}", file_path);
                        Ok(Response::builder()
                            .status(404)
                            .body("Not Found\n".into())
                            .unwrap())
                    }
                }
                "POST" | "PUT" => {
                    let file_path = format!("{}/{}", responses_folder, endpoint.file);
                    info!("📂 Saving file to: {}", file_path);
                    if (async_fs::write(file_path, body).await).is_err() {
                        return Ok(Response::builder()
                            .status(500)
                            .body("Internal Server Error\n".into())
                            .unwrap());
                    }
                    Ok(Response::builder()
                        .status(status_code)
                        .body("Created\n".into())
                        .unwrap())
                }
                "DELETE" => {
                    let file_path = format!("{}/{}", responses_folder, endpoint.file);
                    info!("📂 Deleting file from: {}", file_path);
                    if (async_fs::remove_file(file_path).await).is_err() {
                        return Ok(Response::builder()
                            .status(404)
                            .body("File not Found\n".into())
                            .unwrap());
                    }
                    Ok(Response::builder()
                        .status(status_code)
                        .body("".into())
                        .unwrap())
                }
                _ => Ok(Response::builder()
                    .status(405)
                    .body("Method not allowed\n".into())
                    .unwrap()),
            }
        } else {
            info!("🚫 Method not allowed: {} {}", method, path.as_str());
            Ok(Response::builder()
                .status(405)
                .body("Method not allowed\n".into())
                .unwrap())
        };
    }

    Ok(Response::builder()
        .status(404)
        .body("Not Found\n".into())
        .unwrap())
}

async fn add_possible_delay(endpoint: &Endpoint) {
    if let Some(delay) = endpoint.delay {
        info!("⏳ Applying delay of {} ms", delay);
        sleep(Duration::from_millis(delay)).await;
    }
}

fn default_status_code(endpoint: &Endpoint, method_str: &str) -> u16 {
    endpoint.status_code.unwrap_or(match method_str {
        "GET" => 200,
        "POST" | "PUT" => 201,
        "DELETE" => 204,
        _ => 405,
    })
}

/// Custom rejection handler for returning proper error responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.find::<Unauthorized>().is_some() {
        let response: Response<Body> = Response::builder()
            .status(401)
            .body(Body::from("Unauthorized\n"))
            .unwrap();
        return Ok(response);
    } else if err.find::<RateLimited>().is_some() {
        let response: Response<Body> = Response::builder()
            .status(429)
            .body(Body::from("Rate limit exceeded\n"))
            .unwrap();
        return Ok(response);
    }
    
    Err(err)
}
