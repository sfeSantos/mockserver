use crate::config::Endpoint;
use std::collections::HashMap;
use std::fs;
use tokio::fs as async_fs;
use warp::Filter;
use warp::http::header::AUTHORIZATION;
use warp::http::Response;
use warp::hyper::Body;
use warp::reject::custom;
use crate::authentication::{validate_auth, Unauthorized};

pub fn routes(
    endpoints: HashMap<String, Endpoint>,
    responses_folder: String
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let endpoints = warp::any().map(move || endpoints.clone());
    let responses_folder = warp::any().map(move || responses_folder.clone());

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
        .and_then(handle_request)
        .recover(handle_rejection)
        .with(cors)
}

pub async fn handle_request(
    path: warp::path::FullPath,
    method: warp::http::Method,
    auth_header: Option<String>,
    body: bytes::Bytes,
    endpoints: HashMap<String, Endpoint>,
    responses_folder: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(endpoint) = endpoints.get(path.as_str()) {
        if let Some(auth) = &endpoint.authentication {
            if !validate_auth(auth, auth_header) {
                return Err(custom(Unauthorized));
            }
        }
        
        let method_str = method.as_str();
        let status_code = default_status_code(endpoint, method_str);

        return if endpoint.method.iter().any(|m| m == method_str) {
            match method_str {
                "GET" => {
                    let file_path = format!("{}/{}", responses_folder, endpoint.file);
                    if let Ok(contents) = fs::read_to_string(file_path) {
                        Ok(Response::builder()
                            .status(status_code)
                            .header("Content-Type", "application/json")
                            .body(contents)
                            .unwrap())
                    } else {
                        Ok(Response::builder()
                            .status(404)
                            .body("Not Found\n".into())
                            .unwrap())
                    }
                }
                "POST" | "PUT" => {
                    let file_path = format!("{}/{}", responses_folder, endpoint.file);
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

fn default_status_code(endpoint: &Endpoint, method_str: &str) -> u16 {
    endpoint.status_code.unwrap_or(match method_str {
        "GET" => 200,
        "POST" | "PUT" => 201,
        "DELETE" => 204,
        _ => 405,
    })
}

/// Custom rejection handler for returning proper error responses
async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if err.find::<Unauthorized>().is_some() {
        let response: Response<Body> = Response::builder()
            .status(401)
            .body(Body::from("Unauthorized\n"))
            .unwrap();
        return Ok(response);
    }
    Err(err)
}
