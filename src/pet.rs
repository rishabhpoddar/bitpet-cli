use serde::{Deserialize, Serialize};

use crate::CommandResult;
use crate::config::{Config, UserInfo};
use crate::constants::STATUS_PATH;
use crate::error::CustomErrorTrait;
use crate::http_mocking::MockingMiddleware;
use crate::ui::get_pet_display;
use async_trait::async_trait;

use crate::auth::{AuthenticatedCommand, execute_authenticated_command};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Pet {
    pub user_id: String,
    pub id: String,
    pub name: String,
    pub level: f64,
    pub hunger: f64,
    pub coding_energy: f64,
    pub boredom: f64,
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
    async fn execute(self, pet: Pet, user: UserInfo, config: &mut Config) -> CommandResult;
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
            let pet = get_pet_status(user.token.as_str(), config).await?;
            if pet.is_none() {
                return Err(format!(
                    "You do not yet have a pet! Please use the 'pet new-pet' command to create one."
                )
                .into());
            } else {
                self.command.execute(pet.unwrap(), user, config).await
            }
        }
    }
    execute_authenticated_command(config, AuthCommandIfPetExists { command }).await
}

async fn get_pet_status(
    token: &str,
    config: &mut Config,
) -> Result<Option<Pet>, Box<dyn CustomErrorTrait>> {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .get("https://api.bitpet.dev".to_owned() + STATUS_PATH)
        .bearer_auth(token)
        .send()
        .await?;

    if response.status().is_success() {
        let pet: Pet = response.json().await?;
        Ok(Some(pet))
    } else if response.status().as_u16() == 404 {
        Ok(None)
    } else if response.status().as_u16() == 401 {
        config.user = None;
        config.save()?;
        Err(format!("Oops! Please login again!").into())
    } else {
        let error_text = response.text().await?;
        Err(format!("Failed to get pet status: {}", error_text).into())
    }
}
