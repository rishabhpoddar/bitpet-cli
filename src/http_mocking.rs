use crate::constants::{LOGIN_PATH, LOGOUT_PATH};
use http::Extensions;
use reqwest::{Body, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use serde::Deserialize;
use serde_json::json;

pub struct MockingMiddleware;

#[derive(Deserialize, Debug)]
struct LoginRequest {
    user_code: String,
}

const MOCK_TOKEN: &str = "mock-token";
const MOCK_EMAIL: &str = "mock@bitpet.dev";
const MOCK_USERNAME: &str = "mock-username";
const MOCK_OTP: &str = "-9999";

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
            let token = req
                .headers()
                .get("Authorization")
                .unwrap()
                .to_str()
                .unwrap();
            if token == "Bearer ".to_owned() + MOCK_TOKEN {
                return Ok(http::Response::builder()
                    .status(200)
                    .body(Body::from("Logged out successfully!"))
                    .unwrap()
                    .into());
            }
        }
        next.run(req, extensions).await
    }
}
