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
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let endpoints = warp::any().map(move || endpoints.clone());

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
                    let file_path = format!("responses/{}", endpoint.file);
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
                    let file_path = format!("responses/{}", endpoint.file);
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
                    let file_path = format!("responses/{}", endpoint.file);
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

#[cfg(test)]
mod test {
    use super::*;
    use warp::test::request;

    #[tokio::test]
    async fn test_get_existing_file() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/test".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "test_response.json".to_string(),
                status_code: None,
                authentication: None,
            },
        );

        fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

        let api = routes(endpoints);
        let res = request().method("GET").path("/test").reply(&api).await;

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
                status_code: None,
                authentication: None,
            },
        );

        let api = routes(endpoints);
        let res = request().method("GET").path("/missing").reply(&api).await;

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
                status_code: Some(201),
                authentication: None,
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
                status_code: Some(205),
                authentication: None,
            },
        );

        fs::write("responses/delete.json", "to be deleted").unwrap();

        let api = routes(endpoints);
        let res = request().method("DELETE").path("/delete").reply(&api).await;

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
                status_code: None,
                authentication: None,
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
                authentication: None,
            },
        );

        fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

        let api = routes(endpoints);
        let res = request().method("GET").path("/test").reply(&api).await;

        assert_eq!(res.status(), 201);
        assert_eq!(res.body(), "{\"message\": \"ok\"}");
    }

    #[tokio::test]
    async fn test_unauthorized_access_basic() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/protected".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "protected.json".to_string(),
                status_code: Some(200),
                authentication: Some(serde_yaml::from_str("basic: { user: 'admin', password: 'secret' }").unwrap()),
            },
        );

        let api = routes(endpoints);
        let res = request().method("GET").path("/protected").reply(&api).await;
        assert_eq!(res.status(), 401);
    }

    #[tokio::test]
    async fn test_valid_basic_auth() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/protected".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "protected.json".to_string(),
                status_code: Some(200),
                authentication: Some(serde_yaml::from_str("
                    basic:
                        user: 'admin'
                        password: 'secret'
                ").unwrap()),
            },
        );

        fs::write("responses/protected.json", "{\"message\": \"ok\"}").unwrap();
        
        let api = routes(endpoints);

        let auth_header = "Basic YWRtaW46c2VjcmV0"; // base64 of "admin:secret"
        let res = request()
            .method("GET")
            .path("/protected")
            .header("Authorization", auth_header)
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200);
    }

    #[tokio::test]
    async fn test_valid_bearer_token() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/protected".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "protected.json".to_string(),
                status_code: Some(200),
                authentication: Some(serde_yaml::from_str(r#"
                    bearer:
                        token: 'valid_token'
                "#).unwrap()),
            },
        );

        let api = routes(endpoints);

        // Valid Bearer Token (should return status 200)
        let auth_header = "Bearer valid_token";
        let res = request()
            .method("GET")
            .path("/protected")
            .header("Authorization", auth_header)
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200);
    }

    #[tokio::test]
    async fn test_invalid_bearer_token() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/protected".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "protected.json".to_string(),
                status_code: Some(200),
                authentication: Some(serde_yaml::from_str(r#"
                    bearer:
                        token: 'valid_token'
                "#).unwrap()),
            },
        );

        let api = routes(endpoints);

        // Invalid Bearer Token (should return status 401)
        let auth_header = "Bearer invalid_token";
        let res = request()
            .method("GET")
            .path("/protected")
            .header("Authorization", auth_header)
            .reply(&api)
            .await;
        assert_eq!(res.status(), 401);
    }

    #[tokio::test]
    async fn test_edge_case_missing_claims_in_bearer_token() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/protected".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "protected.json".to_string(),
                status_code: Some(200),
                authentication: Some(serde_yaml::from_str(r#"
                    bearer:
                        token: 'valid_token'
                        claims:
                            role: 'admin'
                "#).unwrap()),
            },
        );

        let api = routes(endpoints);

        // Missing claim in the bearer token (should return 401)
        let auth_header = "Bearer valid_token"; // token without role claim
        let res = request()
            .method("GET")
            .path("/protected")
            .header("Authorization", auth_header)
            .reply(&api)
            .await;
        assert_eq!(res.status(), 401);
    }

    #[tokio::test]
    async fn test_edge_case_invalid_claims_in_bearer_token() {
        let mut endpoints = HashMap::new();
        endpoints.insert(
            "/protected".to_string(),
            Endpoint {
                method: vec!["GET".to_string()],
                file: "protected.json".to_string(),
                status_code: Some(200),
                authentication: Some(serde_yaml::from_str(r#"
                    bearer:
                        token: 'valid_token'
                        claims:
                            role: 'admin'
                "#).unwrap()),
            },
        );

        let api = routes(endpoints);

        // Invalid claim in the bearer token (should return 401)
        let auth_header = "Bearer invalid_token_with_claim"; // token with incorrect claim
        let res = request()
            .method("GET")
            .path("/protected")
            .header("Authorization", auth_header)
            .reply(&api)
            .await;
        assert_eq!(res.status(), 401);
    }
}
