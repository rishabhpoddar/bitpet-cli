use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::CommandResult;
use crate::config::{Config, UserInfo};
use crate::constants::{DOES_PET_EXIST_PATH, FEED_PATH, STATUS_PATH};
use crate::error::CustomErrorTrait;
use crate::git;
use crate::http_mocking::MockingMiddleware;
use crate::ui::Animation;
use crate::ui::get_pet_display;
use async_trait::async_trait;
use serde_json::json;

use crate::auth::{AuthenticatedCommand, execute_authenticated_command};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Pet {
    pub user_id: String,
    pub id: String,
    pub name: String,
    pub level: f64,
    pub hunger: f64,
    pub happiness: f64,
    pub created_at: u64,
    pub streak: u64,
}

impl std::fmt::Display for Pet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_pet_display(self))
    }
}

#[async_trait]
pub trait CommandIfPetExists {
    async fn execute(self, user: UserInfo, config: &mut Config) -> CommandResult;
}

pub async fn execute_command_if_pet_exists(
    config: &mut Config,
    command: impl CommandIfPetExists + Send,
) -> CommandResult {
    struct AuthCommandIfPetExists<C> {
        command: C,
    }
    #[async_trait]
    impl<C: CommandIfPetExists + Send> AuthenticatedCommand for AuthCommandIfPetExists<C> {
        async fn execute(self, user: UserInfo, config: &mut Config) -> CommandResult {
            let does_exist = does_pet_exist(user.token.as_str(), config).await?;
            if !does_exist {
                return Err(format!(
                    "You do not yet have a pet! Please use the 'pet new-pet' command to create one."
                )
                .into());
            } else {
                self.command.execute(user, config).await
            }
        }
    }
    execute_authenticated_command(config, AuthCommandIfPetExists { command }).await
}

async fn does_pet_exist(
    token: &str,
    config: &mut Config,
) -> Result<bool, Box<dyn CustomErrorTrait>> {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .get("https://api.bitpet.dev".to_owned() + DOES_PET_EXIST_PATH)
        .bearer_auth(token)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(true)
    } else if response.status().as_u16() == 404 {
        Ok(false)
    } else if response.status().as_u16() == 401 {
        config.user = None;
        config.save()?;
        Err(format!("Oops! Please login again!").into())
    } else {
        let error_text = response.text().await?;
        Err(format!("Failed to get pet status: {}", error_text).into())
    }
}

#[derive(Serialize, Deserialize)]
pub struct StatusAPIResult {
    pub animation: Animation,
    pub pet: Pet,
}

pub async fn get_pet_status(
    token: &str,
    config: &mut Config,
) -> Result<(Pet, Animation), Box<dyn CustomErrorTrait>> {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .get("https://api.bitpet.dev".to_owned() + STATUS_PATH)
        .bearer_auth(token)
        .send()
        .await?;

    if response.status().is_success() {
        let pet: StatusAPIResult = response.json().await?;
        Ok((pet.pet, pet.animation))
    } else if response.status().as_u16() == 401 {
        config.user = None;
        config.save()?;
        Err(format!("Oops! Please login again!").into())
    } else {
        let error_text = response.text().await?;
        Err(format!("Failed to get pet status: {}", error_text).into())
    }
}

#[derive(Serialize, Deserialize)]
pub enum FeedStatus {
    FeedSuccess,
    TooMuchFood,
    AskForChallenge,
}

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct FeedAPIResult {
    pub animation: Option<Animation>,
    pub status: FeedStatus,
    pub challenge: Option<Challenge>,
    pub pet: Option<Pet>,
    pub text_before_animation: Option<String>,
}

pub async fn feed_pet(
    token: &str,
    config: &mut Config,
    commits: HashMap<String, Vec<git::Commit>>,
) -> Result<FeedAPIResult, Box<dyn CustomErrorTrait>> {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .post("https://api.bitpet.dev".to_owned() + FEED_PATH)
        .bearer_auth(token)
        .body(serde_json::to_string(&json!({
            "commits": commits
        }))?)
        .send()
        .await?;

    if response.status().is_success() {
        let api_result: FeedAPIResult = response.json().await?;
        Ok(api_result)
    } else if response.status().as_u16() == 401 {
        config.user = None;
        config.save()?;
        Err(format!("Oops! Please login again!").into())
    } else {
        let error_text = response.text().await?;
        Err(format!("Failed to get pet status: {}", error_text).into())
    }
}
