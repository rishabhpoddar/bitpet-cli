// src/pet_sim.rs
use crate::http_mocking::Pet;
use crate::model::PET_MODEL;

#[derive(Copy, Clone)]
pub enum Action {
    Feed,
    Play,
    Sleep,
    Tick,
}

fn one_hot(a: Action) -> [f64; 4] {
    match a {
        Action::Feed => [1.0, 0.0, 0.0, 0.0],
        Action::Play => [0.0, 1.0, 0.0, 0.0],
        Action::Sleep => [0.0, 0.0, 1.0, 0.0],
        Action::Tick => [0.0, 0.0, 0.0, 1.0],
    }
}

// IMPORTANT: Order must match `input_cols` used in training:
// ["hunger","energy","happiness","level","action_feed","action_play","action_sleep","action_tick","elapsed_time"]
fn build_features(pet: &Pet, action: Action, elapsed_hours: f64) -> [f64; 9] {
    let oh = one_hot(action);
    [
        pet.hunger as f64,
        pet.energy as f64,
        pet.happiness as f64,
        pet.level as f64,
        oh[0],
        oh[1],
        oh[2],
        oh[3],
        elapsed_hours,
    ]
}

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

pub fn apply_model_transition(pet: &mut Pet, action: Action, elapsed_hours: f64) {
    let x = build_features(pet, action, elapsed_hours);
    let y = PET_MODEL.predict(&x);

    // other stats: clamp and (optionally) round
    let next_hunger = clamp01(y[0]);
    let next_energy = clamp01(y[1]);
    let next_happiness = clamp01(y[2]);

    // IMPORTANT: don't round level per-step; keep fractional progress
    // y[3] is absolute next_level (as trained). Prevent drops if you want:
    let next_level_f = y[3].max(pet.level as f64);

    // write back (cast other stats to u64 if you like)
    pet.hunger = next_hunger;
    pet.energy = next_energy;
    pet.happiness = next_happiness;

    // accumulate level as float in a hidden local, then commit an integer view:
    // simplest: use ceil so any positive progress shows up
    pet.level = next_level_f;
}
