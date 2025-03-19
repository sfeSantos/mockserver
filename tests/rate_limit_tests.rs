use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use warp::test::request;
use mockserver::config::Endpoint;
use mockserver::handler::routes;
use mockserver::rate_limit::{new_rate_limit, RateLimit};

#[tokio::test]
async fn test_request_under_limit() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/test".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "test_response.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: None,
            rate_limit: Some(RateLimit {
                requests: 2,
                window_ms: 1000,
            }),
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let rate_limiter = new_rate_limit();
    let api = routes(endpoints, "responses".to_string(), rate_limiter);

    // First request should succeed
    let resp = request()
        .method("GET")
        .path("/test")
        .reply(&api)
        .await;

    assert_eq!(resp.status(), 200);
    
    let resp = request()
        .method("GET")
        .path("/test")
        .reply(&api)
        .await;

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_exceeding_rate_limit() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/rate_limited".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "test_response.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: None,
            rate_limit: Some(RateLimit {
                requests: 2,
                window_ms: 1000,
            }),
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let rate_limiter = new_rate_limit();
    let api = routes(endpoints, "responses".to_string(), rate_limiter);

    // First request should succeed
    let resp = request()
        .method("GET")
        .path("/rate_limited")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);

    // Second request should succeed
    let resp = request()
        .method("GET")
        .path("/rate_limited")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);

    // Third request should be blocked due to rate limiting
    let resp = request()
        .method("GET")
        .path("/rate_limited")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 429);
}

#[tokio::test]
async fn test_different_endpoints_have_separate_limits() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/endpoint1".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "test_response.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: None,
            rate_limit: Some(RateLimit {
                requests: 1,
                window_ms: 1000,
            }),
        },
    );
    endpoints.insert(
        "/endpoint2".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "test_response.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: None,
            rate_limit: Some(RateLimit {
                requests: 1,
                window_ms: 1000,
            }),
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let rate_limiter = new_rate_limit();
    let api = routes(endpoints, "responses".to_string(), rate_limiter);

    // First request to /endpoint1 should succeed
    let resp = request()
        .method("GET")
        .path("/endpoint1")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);

    // First request to /endpoint2 should also succeed (separate limit)
    let resp = request()
        .method("GET")
        .path("/endpoint2")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_rate_limit_resets_after_window() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/reset_test".to_string(),
        Endpoint {
            method: vec!["GET".to_string()],
            file: "test_response.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: None,
            rate_limit: Some(RateLimit {
                requests: 1,
                window_ms: 500,
            }),
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let rate_limiter = new_rate_limit();
    let api = routes(endpoints, "responses".to_string(), rate_limiter);

    // First request should succeed
    let resp = request()
        .method("GET")
        .path("/reset_test")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);

    // Second request should be blocked
    let resp = request()
        .method("GET")
        .path("/reset_test")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 429);

    // Wait for rate limit window to expire
    sleep(Duration::from_millis(600)).await;

    // Third request should succeed after reset
    let resp = request()
        .method("GET")
        .path("/reset_test")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_different_methods_tracked_separately() {
    let mut endpoints = HashMap::new();
    endpoints.insert(
        "/method_test".to_string(),
        Endpoint {
            method: vec!["GET".to_string(), "POST".to_string()],
            file: "test_response.json".to_string(),
            status_code: Some(200),
            authentication: None,
            delay: None,
            rate_limit: Some(RateLimit {
                requests: 1,
                window_ms: 1000,
            }),
        },
    );

    fs::write("responses/test_response.json", "{\"message\": \"ok\"}").unwrap();

    let rate_limiter = new_rate_limit();
    let api = routes(endpoints, "responses".to_string(), rate_limiter);

    // First GET request should succeed
    let resp = request()
        .method("GET")
        .path("/method_test")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);

    // Second GET request should be blocked
    let resp = request()
        .method("GET")
        .path("/method_test")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 429);

    // POST request should succeed since it has a separate counter
    let resp = request()
        .method("POST")
        .path("/method_test")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 200);
}