use std::collections::HashMap;
use std::sync::LazyLock;

use crate::constants::{LOGIN_PATH, LOGOUT_PATH};
use crate::pet_sim::{Action, apply_model_transition};
use crate::utils::{self, is_weekend_local_timezone};
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

#[derive(Clone)]
pub struct Commit {
    pub hash: String,
    pub time_since_epoch_ms: u64,
}

#[derive(Clone)]
pub struct Pet {
    pub user_id: String,
    pub id: String,
    pub name: String,
    pub level: f64,
    pub hunger: f64,
    pub energy: f64,
    pub happiness: f64,
    pub created_at: u64,
    pub last_fed_commits: HashMap<String, Commit>,
    pub streak_count: u64,
    pub last_streak_day: u64,
    pub last_interaction_time: u64,
    pub timezone: String,
}

pub static PET: LazyLock<Pet> = LazyLock::new(|| Pet {
    user_id: "mock-user-id".to_string(),
    id: "mock-pet-id".to_string(),
    name: "mock-name".to_string(),
    level: 0.0,
    hunger: 40.0,
    energy: 80.0,
    happiness: 60.0,
    created_at: utils::get_ms_time_since_epoch(),
    last_fed_commits: HashMap::new(),
    streak_count: 0,
    last_streak_day: utils::get_ms_time_since_epoch(),
    last_interaction_time: utils::get_ms_time_since_epoch(),
    timezone: "Asia/Kolkata".to_string(),
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

pub fn handle_feed(
    pet: &mut Pet,
    repo_commits: &HashMap<String, Vec<Commit>>,
) -> std::result::Result<(), &'static str> {
    // First, advance time since last interaction via Tick
    let elapsed_hours =
        (utils::get_ms_time_since_epoch() - pet.last_interaction_time) as f64 / 3600000.0;
    if elapsed_hours > 0.0 {
        apply_model_transition(pet, Action::Tick, elapsed_hours);
    }

    let mut processed_any = false;
    let mut too_full = "";

    for (repo, commits) in repo_commits {
        let last_seen = pet.last_fed_commits.get(repo).cloned();

        for commit in commits {
            let is_new = match &last_seen {
                Some(prev) => {
                    commit.hash != prev.hash
                        && commit.time_since_epoch_ms > prev.time_since_epoch_ms
                }
                None => true,
            };

            if is_new {
                // Apply the learned Feed transition; no extra time has passed since Tick
                apply_model_transition(pet, Action::Feed, 0.0);
                processed_any = true;

                pet.last_fed_commits.insert(repo.clone(), commit.clone());
            }
        }
    }

    if processed_any {
        update_streak(pet);
        Ok(())
    } else if too_full != "" {
        Err(too_full)
    } else {
        Err("No new commits to feed on!")
    }
}

pub fn handle_play(pet: &mut Pet) -> std::result::Result<(), &'static str> {
    // Advance time via Tick
    let elapsed_hours =
        (utils::get_ms_time_since_epoch() - pet.last_interaction_time) as f64 / 3600000.0;
    if elapsed_hours > 0.0 {
        apply_model_transition(pet, Action::Tick, elapsed_hours);
    }
    // Apply Play with zero additional time
    apply_model_transition(pet, Action::Play, 0.0);
    Ok(())
}

pub fn handle_sleep(pet: &mut Pet) -> std::result::Result<(), &'static str> {
    // Advance time via Tick
    let elapsed_hours =
        (utils::get_ms_time_since_epoch() - pet.last_interaction_time) as f64 / 3600000.0;
    if elapsed_hours > 0.0 {
        apply_model_transition(pet, Action::Tick, elapsed_hours);
    }
    // Apply Sleep with zero additional time
    apply_model_transition(pet, Action::Sleep, 0.0);
    Ok(())
}

fn update_streak(pet: &mut Pet) {
    let today = utils::current_day_local_timezone();

    if today == pet.last_streak_day {
        // Already fed today → no change
        return;
    }

    if today == pet.last_streak_day + 1 {
        // Consecutive day → streak++
        pet.streak_count += 1;
    } else {
        // Missed at least one day → reset streak
        pet.streak_count = 1;
    }

    pet.last_streak_day = today;
}
