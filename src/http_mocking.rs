use std::collections::HashMap;
use std::sync::LazyLock;

use crate::constants::{LOGIN_PATH, LOGOUT_PATH};
use crate::utils;
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
struct Commit {
    hash: String,
    time_since_epoch_ms: u64,
}

#[derive(Clone)]
struct Pet {
    user_id: String,
    id: String,
    name: String,
    level: u64,
    hunger: u64,
    energy: u64,
    happiness: u64,
    created_at: u64,
    last_time_slept: u64,
    last_fed_commits: HashMap<String, Commit>,
    streak_count: u64,
    last_streak_day: u64,
    last_interaction_time: u64,
    timezone: String,
}

static PET: LazyLock<Pet> = LazyLock::new(|| Pet {
    user_id: "mock-user-id".to_string(),
    id: "mock-pet-id".to_string(),
    name: "mock-name".to_string(),
    level: 0,
    hunger: 40,
    energy: 80,
    happiness: 60,
    created_at: utils::get_ms_time_since_epoch(),
    last_time_slept: utils::get_ms_time_since_epoch(),
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

fn handle_feed(
    pet: &mut Pet,
    repo_commits: &HashMap<String, Vec<Commit>>, // repo → new commits
) -> std::result::Result<(), &'static str> {
    let elapsed_hours =
        (utils::get_ms_time_since_epoch() - pet.last_interaction_time) as f32 / 3600000.0;
    tick(pet, elapsed_hours, false);
    pet.last_interaction_time = utils::get_ms_time_since_epoch();

    let mut processed_any = false;
    let mut too_full = "";

    for (repo, commits) in repo_commits {
        // Get last seen commit for this repo (if any)
        let last_seen = pet.last_fed_commits.get(repo).cloned();

        // Process commits in chronological order (assume input sorted oldest → newest)
        for commit in commits {
            let is_new = match &last_seen {
                Some(prev) => {
                    commit.hash != prev.hash
                        && commit.time_since_epoch_ms > prev.time_since_epoch_ms
                }
                None => true,
            };

            if is_new {
                // Each commit = one feed
                match feed(pet, pet.streak_count) {
                    Ok(()) => {
                        processed_any = true;
                    }
                    Err(msg) => {
                        too_full = msg;
                    }
                }

                // Update last seen commit for this repo
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

fn handle_play(pet: &mut Pet) -> std::result::Result<(), &'static str> {
    let elapsed_hours =
        (utils::get_ms_time_since_epoch() - pet.last_interaction_time) as f32 / 3600000.0;
    tick(pet, elapsed_hours, utils::is_weekend_local_timezone());
    pet.last_interaction_time = utils::get_ms_time_since_epoch();
    play(pet)?;
    Ok(())
}

fn handle_sleep(pet: &mut Pet) -> std::result::Result<(), &'static str> {
    let elapsed_hours =
        (utils::get_ms_time_since_epoch() - pet.last_interaction_time) as f32 / 3600000.0;
    tick(pet, elapsed_hours, utils::is_weekend_local_timezone());
    pet.last_interaction_time = utils::get_ms_time_since_epoch();
    sleep(pet)?;
    Ok(())
}

fn tick(pet: &mut Pet, elapsed_hours: f32, is_weekend: bool) {
    // Hunger ↑ gradually
    let mut hunger_rate = 1.6; // baseline per hour (~+38/day)
    if is_weekend {
        hunger_rate *= 0.4; // weekend forgiveness
    }
    if pet.happiness >= 70 {
        hunger_rate *= 0.6; // happy pet: slower hunger
    }
    if pet.happiness <= 30 {
        hunger_rate *= 1.25; // sad pet: faster hunger
    }
    pet.hunger = (pet.hunger as f32 + hunger_rate * elapsed_hours).min(100.0) as u64;

    // Energy ↑ passively (resting recovers energy)
    pet.energy = (pet.energy as f32 + 8.0 * elapsed_hours).min(100.0) as u64;

    // Happiness drifts ↓ if ignored
    let mut happiness_change = -2.0 * elapsed_hours;
    if is_weekend {
        happiness_change *= 0.5; // less unhappiness on weekends
    }
    if pet.hunger >= 80 {
        happiness_change -= 1.0 * elapsed_hours;
    }
    if pet.energy <= 20 {
        happiness_change -= 1.0 * elapsed_hours;
    }
    if pet.hunger <= 30 && pet.energy >= 50 {
        happiness_change += 0.5 * elapsed_hours;
    }
    pet.happiness = (pet.happiness as f32 + happiness_change).clamp(0.0, 100.0) as u64;

    // Level decays if starving
    if pet.hunger == 100 {
        pet.level = pet.level.saturating_sub((5.0 * elapsed_hours) as u64);
    }
}

fn feed(pet: &mut Pet, streak_days: u64) -> std::result::Result<(), &'static str> {
    if pet.hunger <= 10 {
        return Err("I'm full...");
    }

    // XP gain grows with streak
    let streak_bonus = 1.0 + (0.15 * streak_days as f32).min(1.5); // up to ×2.5
    let base_gain = 10.0;
    let level_gain = (base_gain * streak_bonus).floor() as u64;

    // Apply effects
    pet.hunger = pet.hunger.saturating_sub(15);
    pet.energy = pet.energy.saturating_sub(5);
    pet.happiness = (pet.happiness + 5).min(100);
    pet.level += level_gain;

    Ok(()) // return how much "level" went up
}

fn sleep(pet: &mut Pet) -> std::result::Result<(), &'static str> {
    if pet.energy >= 70 {
        return Err("I'm not sleepy yet!");
    }

    let level_gain = 5;

    pet.energy = (pet.energy + 50).min(100);
    pet.hunger = (pet.hunger + 10).min(100);
    pet.happiness = (pet.happiness + 2).min(100);
    pet.level += level_gain;
    pet.last_time_slept = utils::get_ms_time_since_epoch();

    Ok(())
}

fn play(pet: &mut Pet) -> std::result::Result<(), &'static str> {
    if pet.energy <= 20 {
        return Err("Too tired to play—let me rest!");
    }
    if pet.hunger >= 80 {
        return Err("I'm starving—feed me first!");
    }

    let level_gain = 5;

    pet.happiness = (pet.happiness + 15).min(100);
    pet.energy = pet.energy.saturating_sub(10);
    pet.level += level_gain;

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
