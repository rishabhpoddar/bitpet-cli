use crate::constants::{
    CHALLENGE_ANS_PATH, DOES_PET_EXIST_PATH, FEED_PATH, LOGIN_PATH, LOGOUT_PATH, PLAY_PATH,
    STATUS_PATH,
};
use crate::pet::StatusAPIResult;
use crate::pet::{
    Challenge, ChallengeAnswerAPIResult, ChallengeAnswerStatus, ChallengeAnswerType, FeedAPIResult,
    FeedStatus, Pet, PlayAPIResult, PlayStatus,
};
use http::Extensions;
use reqwest::{Body, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use serde::Deserialize;
use serde_json::json;
use std::sync::LazyLock;

use crate::ui::{Animation, AnimationWindow};
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
    happiness: 40.0,
    created_at: 0,
    streak: 0,
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
        } else if path == DOES_PET_EXIST_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    return Ok(http::Response::builder()
                        .status(200)
                        .body("")
                        .unwrap()
                        .into());
                }
            }
        } else if path == STATUS_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    let result = StatusAPIResult {
                        animation: generate_pet_status_animation(),
                        pet: PET.clone(),
                    };
                    return Ok(http::Response::builder()
                        .status(200)
                        .body(Body::from(serde_json::to_string(&result).unwrap()))
                        .unwrap()
                        .into());
                }
            }
        } else if path == FEED_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    return Ok(http::Response::builder()
                        .status(200)
                        .body(Body::from(
                            serde_json::to_string(&FeedAPIResult {
                                animation: None,
                                text_before_animation: None,
                                status: FeedStatus::AskForChallenge,
                                challenge: Some(Challenge {
                                    id: "mock-challenge-id".to_string(),
                                    description: "You are given an array in which you need to sort all the numbers.\nImplement this!".to_string(),
                                    answer_type: ChallengeAnswerType::Text,
                                }),
                                pet: None,
                            })
                            .unwrap(),
                        ))
                        .unwrap()
                        .into());
                }
            }
        } else if path == CHALLENGE_ANS_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    return Ok(http::Response::builder()
                        .status(200)
                        .body(Body::from(
                            serde_json::to_string(&ChallengeAnswerAPIResult {
                                feed_result: None,
                                status: ChallengeAnswerStatus::Incorrect,
                            })
                            .unwrap(),
                        ))
                        .unwrap()
                        .into());
                }
            }
        } else if path == PLAY_PATH {
            let token = req.headers().get("Authorization");
            if !token.is_none() {
                let token = token.unwrap().to_str();
                if token.is_ok() && token.unwrap() == "Bearer ".to_owned() + MOCK_TOKEN {
                    return Ok(http::Response::builder()
                        .status(200)
                        .body(Body::from(
                            serde_json::to_string(&PlayAPIResult {
                                animation: None,
                                text_before_animation: Some(format!(
                                    "{} is in no mood to play right now! Please try later, or consider feeding them instead or just taking a break!", 
                                    PET.name
                                )),
                                status: PlayStatus::TooMuchPlay,
                                pet: Some(PET.clone()),
                            })
                            .unwrap(),
                        ))
                        .unwrap()
                        .into());
                }
            }
        }
        next.run(req, extensions).await
    }
}

fn generate_pet_status_animation() -> Animation {
    let fps = 30;
    let total_frames = 100;
    let mut windows = vec![];

    let mut curr_window_start: u64 = 0;
    let mut curr_frame: u64 = 0;
    let mut previous_delta_x_from_center: i16 = 0;
    let mut previous_delta_y_from_center: i16 = 0;
    let mut previous_image: Option<String> = None;
    let mut previous_colours: Option<Vec<Vec<String>>> = None;
    let mut total_jumps = 0;
    let mut last_jump_frame = 0;
    while curr_frame < total_frames {
        let delta_x_from_center = 0;
        let delta_y_from_center = if curr_frame == 0 || curr_frame % 15 != 0 || total_jumps >= 3 {
            if curr_frame > 1 && (curr_frame - last_jump_frame <= 3) {
                if curr_frame - last_jump_frame == 1 {
                    previous_delta_y_from_center - 1
                } else {
                    previous_delta_y_from_center
                }
            } else {
                std::cmp::min(0, previous_delta_y_from_center + 2)
            }
        } else {
            total_jumps += 1;
            last_jump_frame = curr_frame;
            previous_delta_y_from_center - 1
        };
        let (image, colours) = get_pet_status_animation_for_frame(
            &PET,
            curr_frame,
            delta_y_from_center < 0,
            total_jumps >= 3,
        );
        if previous_image.as_ref().is_some()
            && previous_colours.as_ref().is_some()
            && previous_delta_x_from_center == delta_x_from_center
            && previous_delta_y_from_center == delta_y_from_center
            && previous_image.as_ref().unwrap() == &image
            && previous_colours.as_ref().unwrap() == &colours
        {
            curr_frame += 1;
        } else {
            if previous_image.is_some() && previous_colours.is_some() {
                windows.push(AnimationWindow {
                    start_frame_inclusive: curr_window_start,
                    end_frame_inclusive: curr_frame - 1,
                    image: previous_image.unwrap(),
                    colours: previous_colours.unwrap(),
                    delta_x_from_center: previous_delta_x_from_center,
                    delta_y_from_center: previous_delta_y_from_center,
                });
            }
            previous_delta_x_from_center = delta_x_from_center;
            previous_delta_y_from_center = delta_y_from_center;
            curr_window_start = curr_frame;
            previous_image = Some(image);
            previous_colours = Some(colours);
            curr_frame += 1;
        }
    }
    windows.push(AnimationWindow {
        start_frame_inclusive: curr_window_start,
        end_frame_inclusive: curr_frame - 1,
        image: previous_image.unwrap(),
        colours: previous_colours.unwrap(),
        delta_x_from_center: previous_delta_x_from_center,
        delta_y_from_center: previous_delta_y_from_center,
    });

    return Animation { windows, fps };
}

fn get_pet_status_animation_for_frame(
    pet: &Pet,
    curr_frame: u64,
    is_in_the_air: bool,
    is_done_jumping: bool,
) -> (String, Vec<Vec<String>>) {
    let ear_colour = "#0000ff";
    let eye_colour = match pet.happiness {
        h if h < 20.0 => "#ff0000", // red
        h if h < 70.0 => "#0000ff", // blue
        _ => "#00ff00",             // green
    };
    let mut eyes = match pet.happiness {
        h if h < 20.0 => "x.x",
        h if h < 70.0 => "o.o",
        _ => "^.^",
    };

    if is_done_jumping {
        eyes = if curr_frame % 40 == 0 && curr_frame != 0 {
            "-.-"
        } else {
            eyes
        };
    }

    if is_in_the_air {
        eyes = "-.-";
    }

    let tongue_colour = "#ff0000";
    let tongue = match pet.hunger {
        h if h < 20.0 => "U",
        h if h < 70.0 => "-",
        _ => "~",
    };

    let full_face = [
        format!("/\\_/\\"),
        format!("( {} )", eyes),
        format!("=  {}  =", tongue),
    ]
    .join("\n");

    let colours = vec![
        vec![
            ear_colour.to_string(),
            ear_colour.to_string(),
            ear_colour.to_string(),
            ear_colour.to_string(),
            "".to_string(),
        ],
        vec![
            "".to_string(),
            "".to_string(),
            eye_colour.to_string(),
            "".to_string(),
            eye_colour.to_string(),
        ],
        vec![
            "".to_string(),
            "".to_string(),
            "".to_string(),
            tongue_colour.to_string(),
        ],
    ];

    (full_face, colours)
}
