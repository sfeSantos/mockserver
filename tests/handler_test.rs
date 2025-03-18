use std::collections::HashMap;
use std::fs;
use tokio::time::Instant;
use warp::test::request;
use mockserver::config::Endpoint;
use mockserver::handler::routes;

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
            delay: None,
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let api = routes(endpoints, String::from("responses"));
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
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));
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
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));
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
            delay: None,
        },
    );

    fs::write("responses/delete.json", "to be deleted").unwrap();

    let api = routes(endpoints, String::from("responses"));
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
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));
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
            delay: None,
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let api = routes(endpoints, String::from("responses"));
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
            authentication: Some(
                serde_yaml::from_str("basic: { user: 'admin', password: 'secret' }").unwrap(),
            ),
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));
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
            authentication: Some(
                serde_yaml::from_str(
                    "
            basic:
                user: 'admin'
                password: 'secret'
        ",
                )
                .unwrap(),
            ),
            delay: None,
        },
    );

    fs::write("responses/protected.json", "{\"message\": \"ok\"}").unwrap();

    let api = routes(endpoints, String::from("responses"));

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
            authentication: Some(
                serde_yaml::from_str(
                    r#"
            bearer:
                token: 'valid_token'
        "#,
                )
                .unwrap(),
            ),
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));

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
            authentication: Some(
                serde_yaml::from_str(
                    r#"
            bearer:
                token: 'valid_token'
        "#,
                )
                .unwrap(),
            ),
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));

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
            authentication: Some(
                serde_yaml::from_str(
                    r#"
            bearer:
                token: 'valid_token'
                claims:
                    role: 'admin'
        "#,
                )
                .unwrap(),
            ),
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));

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
            authentication: Some(
                serde_yaml::from_str(
                    r#"
            bearer:
                token: 'valid_token'
                claims:
                    role: 'admin'
        "#,
                )
                .unwrap(),
            ),
            delay: None,
        },
    );

    let api = routes(endpoints, String::from("responses"));

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

#[tokio::test]
async fn test_response_with_delay() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/test".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "protected.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: Some(500), // 500ms delay
        },
    );

    let start_time = Instant::now();
    let api = routes(endpoints, String::from("responses"));

    let res = request()
        .method("GET")
        .path("/test")
        .reply(&api).await;

    let elapsed = start_time.elapsed();

    assert!(elapsed.as_millis() >= 500, "Expected at least 500ms delay");
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn test_response_with_zero_delay() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/test".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "protected.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: Some(0), // Edge case: 0 delay
        },
    );

    let start_time = Instant::now();

    let api = routes(endpoints, String::from("responses"));

    let res = request()
        .method("GET")
        .path("/test")
        .reply(&api).await;

    let elapsed = start_time.elapsed();
    assert!(elapsed.as_millis() < 50, "Expected minimal delay for 0ms setting");
    assert_eq!(res.status(), 200);
}