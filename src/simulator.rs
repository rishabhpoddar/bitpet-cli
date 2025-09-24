mod constants;
mod error;
mod git;
mod http_mocking;
mod utils;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;
pub mod model;
pub mod pet_sim;

use crate::http_mocking::{Commit, PET, Pet};
use std::collections::HashMap;

enum Action {
    Feed(u32), // number of commits
    Play,
    Sleep,
}

trait DevProfile {
    fn actions_for_day(&mut self, day: u64) -> Vec<(u64, Action)>;
}

struct DailyCoder;

impl DevProfile for DailyCoder {
    fn actions_for_day(&mut self, _day: u64) -> Vec<(u64, Action)> {
        vec![
            (10 * 3600 * 1000, Action::Feed(1)),
            (11 * 3600 * 1000, Action::Feed(1)),
            (13 * 3600 * 1000, Action::Play),
            // (14 * 3600 * 1000, Action::Sleep),
        ] // 2 commits
    }
}

fn get_start_of_day_ms_time(day_delta: u64) -> u64 {
    let mut now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    now += day_delta * 24 * 3600 * 1000;
    now - now % (24 * 3600 * 1000)
}

fn simulate(dev: &mut dyn DevProfile, days: u64, pet: &mut Pet) {
    for day in 0..days {
        let now: u64 = get_start_of_day_ms_time(day);
        // Dev actions
        for action in dev.actions_for_day(day) {
            match action {
                (time_delta, Action::Feed(commits)) => {
                    let target_time = now + time_delta;
                    utils::set_delta_ms_since_now(
                        target_time
                            - SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                    );
                    let mut repo_commits = HashMap::new();
                    for i in 0..commits {
                        repo_commits.insert(
                            "test".to_string(),
                            vec![Commit {
                                hash: format!("{}", Uuid::new_v4()),
                                time_since_epoch_ms: target_time
                                    - ((commits * 1000 * 60) + (i * 1000 * 60)) as u64,
                            }],
                        );
                    }
                    let result: Result<(), &'static str> =
                        crate::http_mocking::handle_feed(pet, &repo_commits);
                    if let Err(e) = result {
                        println!("Error: {}", e);
                    }
                }
                (time_delta, Action::Play) => {
                    let target_time = now + time_delta;
                    utils::set_delta_ms_since_now(
                        target_time
                            - SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                    );
                    let result = crate::http_mocking::handle_play(pet);
                    if let Err(e) = result {
                        println!("Error: {}", e);
                    }
                }
                (time_delta, Action::Sleep) => {
                    let target_time = now + time_delta;
                    utils::set_delta_ms_since_now(
                        target_time
                            - SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                    );
                    let result = crate::http_mocking::handle_sleep(pet);
                    if let Err(e) = result {
                        println!("Error: {}", e);
                    }
                }
            }
        }

        println!(
            "Day {} → Level: {}, Hunger: {}, Energy: {}, Happiness: {}, Streak: {}",
            day,
            format!("{:.1}", pet.level),
            format!("{:.1}", pet.hunger),
            format!("{:.1}", pet.energy),
            format!("{:.1}", pet.happiness),
            pet.streak_count
        );
    }
}

fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let mut dev = DailyCoder;
    let mut pet = PET.clone();
    simulate(&mut dev, 30, &mut pet);
}
