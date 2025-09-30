use crate::constants::{LOGIN_PATH, LOGOUT_PATH, STATUS_PATH};
use crate::pet::Pet;
use http::Extensions;
use reqwest::{Body, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use serde::Deserialize;
use serde_json::json;
use std::sync::LazyLock;

pub struct MockingMiddleware;

#[derive(Deserialize, Debug)]
struct LoginRequest {
    user_code: String,
}

const MOCK_TOKEN: &str = "mock-token";
const MOCK_EMAIL: &str = "mock@bitpet.dev";
const MOCK_USERNAME: &str = "mock-username";
const MOCK_OTP: &str = "-9999";

pub static PET: LazyLock<Pet> = LazyLock::new(|| Pet {
    user_id: "mock-user-id".to_string(),
    id: "mock-pet-id".to_string(),
    name: "mock-name".to_string(),
    level: 0.0,
    hunger: 40.0,
    coding_energy: 80.0,
    boredom: 60.0,
    created_at: 0,
});

#[async_trait::async_trait]
impl Middleware for MockingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let path = req.url().path();
        if path == LOGIN_PATH {
            let body = req.body().unwrap().as_bytes().unwrap();
            let login_request: LoginRequest = serde_json::from_slice(body).unwrap();
            if login_request.user_code == MOCK_OTP {
                return Ok(http::Response::builder()
                    .status(200)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "username": MOCK_USERNAME,
                            "email": MOCK_EMAIL,
                            "token": MOCK_TOKEN
                        }))
                        .unwrap(),
                    ))
                    .unwrap()
                    .into());
            }
        } else if path == LOGOUT_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    return Ok(http::Response::builder()
                        .status(200)
                        .body(Body::from("Logged out successfully!"))
                        .unwrap()
                        .into());
                }
            }
        } else if path == STATUS_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    return Ok(http::Response::builder()
                        .status(200)
                        .body(Body::from(serde_json::to_string(&PET.clone()).unwrap()))
                        .unwrap()
                        .into());
                }
            }
        }
        next.run(req, extensions).await
    }
}
