use std::collections::HashMap;
use tokio::fs as async_fs;
use std::fs;
use warp::Filter;
use warp::http::Response;
use crate::config::Endpoint;

pub fn routes(endpoints: HashMap<String, Endpoint>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let endpoints = warp::any()
        .map(move || endpoints.clone());
    
    warp::path::full()
        .and(warp::method())
        .and(warp::body::bytes())
        .and(endpoints)
        .and_then(handle_request)
}

pub async fn handle_request(
    path: warp::path::FullPath,
    method: warp::http::Method,
    body: bytes::Bytes,
    endpoints: HashMap<String, Endpoint>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(endpoint) = endpoints.get(path.as_str()) {
        let method_str = method.as_str();

        return if endpoint.method.iter().any(|m| m == method_str) {
            match method_str {
                "GET" => {
                    let file_path = format!("responses/{}", endpoint.file);
                    if let Ok(contents) = fs::read_to_string(file_path) {
                        Ok(Response::builder()
                            .status(200)
                            .header("Content-Type", "application/json")
                            .body(contents)
                            .unwrap())
                    } else {
                        Ok(Response::builder().status(404).body("Not Found\n".into()).unwrap())
                    }
                }
                "POST" | "PUT" => {
                    let file_path = format!("responses/{}", endpoint.file);
                    if let Err(_) = async_fs::write(file_path, body).await {
                        return Ok(Response::builder().status(500).body("Internal Server Error\n".into()).unwrap());
                    }
                    Ok(Response::builder().status(201).body("Created\n".into()).unwrap())
                }
                "DELETE" => {
                    let file_path = format!("responses/{}", endpoint.file);
                    if let Err(_) = async_fs::remove_file(file_path).await {
                        return Ok(Response::builder().status(404).body("File not Found\n".into()).unwrap());
                    }
                    Ok(Response::builder().status(204).body("".into()).unwrap())
                }
                _ => Ok(Response::builder().status(405).body("Method not allowed\n".into()).unwrap())
            }
        } else {
            Ok(Response::builder().status(405).body("Method not allowed\n".into()).unwrap())
        }
    }

    Ok(Response::builder().status(404).body("Not Found\n".into()).unwrap())
}