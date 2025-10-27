use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::CommandResult;
use crate::config::{Config, UserInfo};
use crate::constants::{
    CHALLENGE_ANS_PATH, DOES_PET_EXIST_PATH, FEED_PATH, PLAY_PATH, STATUS_PATH,
};
use crate::error::CustomErrorTrait;
use crate::git;
use crate::http_mocking::MockingMiddleware;
use crate::ui::Animation;
use crate::ui::get_pet_display;
use crate::utils;
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
        .get(utils::get_api_base_url() + DOES_PET_EXIST_PATH)
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
        .get(utils::get_api_base_url() + STATUS_PATH)
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
    NoFood,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Challenge {
    pub id: String,
    pub description: String,
    pub answer_type: ChallengeAnswerType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChallengeAnswerType {
    Text,
    File,
}

impl Default for ChallengeAnswerType {
    fn default() -> Self {
        Self::Text
    }
}

impl std::fmt::Display for Challenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "==========\nChallenge ID: {}\n\n\x1b[34m{}\x1b[0m\n==========",
            self.id, self.description
        )
    }
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
        .post(utils::get_api_base_url() + FEED_PATH)
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

#[derive(Serialize, Deserialize)]
pub struct ChallengeAnswerAPIResult {
    pub status: ChallengeAnswerStatus,
    pub feed_result: Option<FeedAPIResult>,
}

#[derive(Serialize, Deserialize)]
pub enum ChallengeAnswerStatus {
    Correct,
    Incorrect,
}

pub async fn submit_challenge_answer(
    token: &str,
    config: &mut Config,
    challenge_id: String,
    answer: String,
) -> Result<ChallengeAnswerAPIResult, Box<dyn CustomErrorTrait>> {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .post(utils::get_api_base_url() + CHALLENGE_ANS_PATH)
        .bearer_auth(token)
        .body(serde_json::to_string(&json!({
            "challenge_id": challenge_id,
            "answer": answer
        }))?)
        .send()
        .await?;

    if response.status().is_success() {
        let api_result: ChallengeAnswerAPIResult = response.json().await?;
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

#[derive(Serialize, Deserialize)]
pub enum PlayStatus {
    PlaySuccess,
    TooMuchPlay,
}

#[derive(Serialize, Deserialize)]
pub struct PlayAPIResult {
    pub animation: Option<Animation>,
    pub status: PlayStatus,
    pub pet: Option<Pet>,
    pub text_before_animation: Option<String>,
}

pub async fn play_with_pet(
    token: &str,
    config: &mut Config,
) -> Result<PlayAPIResult, Box<dyn CustomErrorTrait>> {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .post(utils::get_api_base_url() + PLAY_PATH)
        .bearer_auth(token)
        .send()
        .await?;

    if response.status().is_success() {
        let api_result: PlayAPIResult = response.json().await?;
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
