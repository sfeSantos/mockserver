use base64::{Engine as _, engine::{general_purpose}};
use serde_yaml::Value;
use warp::reject::Reject;

#[derive(Debug)]
pub struct Unauthorized;

impl Reject for Unauthorized {}

pub fn validate_auth(auth: &Value, auth_header: Option<String>) -> bool {
    if let Some(header) = auth_header {
        if let Some(basic) = auth.get("basic") {
            if let (Some(user), Some(password)) = (basic.get("user"), basic.get("password")) {
                if let Ok(decoded) = general_purpose::STANDARD
                    .decode(header.replace("Basic ", "")) {
                    let creds = String::from_utf8_lossy(&decoded);
                    let expected = format!("{}:{}", user.as_str().unwrap(), password.as_str().unwrap());
                    return creds == expected;
                }
            }
        }

        if let Some(bearer) = auth.get("bearer") {
            if let Some(token) = bearer.get("token") {
                let provided_token = header.replace("Bearer ", "");
                if provided_token == token.as_str().unwrap() {
                    return validate_claims(bearer, &provided_token);
                }
            }
        }
    }

    false
}

fn validate_claims(expected_claims: &Value, token: &str) -> bool {
    let decoded_claims: Value = serde_json::from_str(token).unwrap_or_default();

    if let Some(expected_map) = expected_claims.as_mapping() {
        for (key, expected_value) in expected_map.iter() {
            if let Some(key_str) = key.as_str() {
                if key_str == "token" {
                    continue;
                }
                if let Some(actual_value) = decoded_claims.get(key_str) {
                    if actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false; // Expected claim is missing
                }
            }
        }
    }
    
    true
}