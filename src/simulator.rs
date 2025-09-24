mod constants;
mod error;
mod git;
mod http_mocking;
pub mod model;
pub mod pet_sim;
mod utils;

use crate::http_mocking::{PET, Pet};

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
            (10, Action::Feed(1)),
            (11, Action::Feed(1)),
            (13, Action::Play),
            // (14, Action::Sleep),
        ] // 2 commits
    }
}

fn simulate(dev: &mut dyn DevProfile, days: u64, pet: &mut Pet) {
    for day in 0..days {
        // Dev actions
        for action in dev.actions_for_day(day) {
            match action {
                (time_delta, Action::Feed(commits)) => {
                    let target_time = day * 24 + time_delta;
                    let result: Result<(), &'static str> =
                        crate::http_mocking::handle_feed(pet, commits as u64, target_time);
                    if let Err(e) = result {
                        println!("Error: {}", e);
                    }
                }
                (time_delta, Action::Play) => {
                    let target_time = day * 24 + time_delta;
                    let result = crate::http_mocking::handle_play(pet, target_time);
                    if let Err(e) = result {
                        println!("Error: {}", e);
                    }
                }
                (time_delta, Action::Sleep) => {
                    let target_time = day * 24 + time_delta;
                    let result = crate::http_mocking::handle_sleep(pet, target_time);
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
