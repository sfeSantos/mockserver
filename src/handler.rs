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
        let status_code = default_status_code(endpoint, method_str);

        return if endpoint.method.iter().any(|m| m == method_str) {
            match method_str {
                "GET" => {
                    let file_path = format!("responses/{}", endpoint.file);
                    if let Ok(contents) = fs::read_to_string(file_path) {
                        Ok(Response::builder()
                            .status(status_code)
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
                    Ok(Response::builder().status(status_code).body("Created\n".into()).unwrap())
                }
                "DELETE" => {
                    let file_path = format!("responses/{}", endpoint.file);
                    if let Err(_) = async_fs::remove_file(file_path).await {
                        return Ok(Response::builder().status(404).body("File not Found\n".into()).unwrap());
                    }
                    Ok(Response::builder().status(status_code).body("".into()).unwrap())
                }
                _ => Ok(Response::builder().status(405).body("Method not allowed\n".into()).unwrap())
            }
        } else {
            Ok(Response::builder().status(405).body("Method not allowed\n".into()).unwrap())
        }
    }

    Ok(Response::builder().status(404).body("Not Found\n".into()).unwrap())
}

fn default_status_code(endpoint: &Endpoint, method_str: &str) -> u16 {
    endpoint.status_code.unwrap_or_else(|| match method_str {
        "GET" => 200,
        "POST" | "PUT" => 201,
        "DELETE" => 204,
        _ => 405,
    })
}

#[cfg(test)]
mod test {
    use warp::test::request;
    use super::*;

    #[tokio::test]
    async fn test_get_existing_file() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/test".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "test_response.json".to_string(),
                status_code: None
            },
        );

        fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

        let api = routes(endpoints);
        let res = request()
            .method("GET")
            .path("/test")
            .reply(&api)
            .await;

        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "{\"message\": \"ok\"}");
    }

    #[tokio::test]
    async fn test_get_non_existent_file() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/missing".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "missing.json".to_string(),
                status_code: None
            },
        );

        let api = routes(endpoints);
        let res = request()
            .method("GET")
            .path("/missing")
            .reply(&api)
            .await;

        assert_eq!(res.status(), 404);
    }

    #[tokio::test]
    async fn test_post_create_file() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/create".to_string(),
            Endpoint {
                method: vec!["POST".to_string()],
                file: "create.json".to_string(),
                status_code: Some(201)
            },
        );

        let api = routes(endpoints);
        let res = request()
            .method("POST")
            .path("/create")
            .body("{\"data\": \"test\"}")
            .reply(&api)
            .await;
        let contents = fs::read_to_string("responses/create.json").unwrap();

        assert_eq!(res.status(), 201);
        assert_eq!(contents, "{\"data\": \"test\"}");
    }

    #[tokio::test]
    async fn test_delete_existing_file() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/delete".to_string(),
            Endpoint {
                method: vec!["DELETE".to_string()],
                file: "delete.json".to_string(),
                status_code: Some(205)
            },
        );

        fs::write("responses/delete.json", "to be deleted").unwrap();

        let api = routes(endpoints);
        let res = request()
            .method("DELETE")
            .path("/delete")
            .reply(&api)
            .await;

        assert_eq!(res.status(), 205);
        assert!(fs::metadata("responses/delete.json").is_err());
    }

    #[tokio::test]
    async fn test_method_not_allowed() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/forbidden".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "forbidden.json".to_string(),
                status_code: None
            },
        );

        let api = routes(endpoints);
        let res = request()
            .method("POST")
            .path("/forbidden")
            .reply(&api)
            .await;

        assert_eq!(res.status(), 405);
    }

    #[tokio::test]
    async fn test_get_existing_file_with_custom_status() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/test".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "test_response.json".to_string(),
                status_code: Some(201),
            },
        );

        fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

        let api = routes(endpoints);
        let res = request()
            .method("GET")
            .path("/test")
            .reply(&api)
            .await;

        assert_eq!(res.status(), 201);
        assert_eq!(res.body(), "{\"message\": \"ok\"}");
    }
}