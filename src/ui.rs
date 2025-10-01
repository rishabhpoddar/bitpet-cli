use crate::pet;
use colored::*;

pub fn get_pet_display(pet: &pet::Pet) -> String {
    // Calculate age in days
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let age_days = (current_time - pet.created_at) / (1000 * 60 * 60 * 24);

    // Color functions for different stats
    let hunger_color = get_hunger_color(pet.hunger);
    let happiness_color = get_happiness_color(pet.happiness);

    format!(
        "Here is how {} is feeling:\n- Level: {}\n- Hunger: {}\n- Happiness: {}\n- Coding streak days: {}\n- Age: {} days",
        pet.name,
        pet.level.to_string(),
        hunger_color,
        happiness_color,
        pet.streak.to_string(),
        age_days.to_string()
    )
}

fn get_hunger_color(hunger: f64) -> String {
    match hunger {
        0.0..=30.0 => format!("{}", hunger.to_string().green()),
        31.0..=75.0 => format!("{}", hunger.to_string().yellow()),
        _ => format!("{}", hunger.to_string().red()),
    }
}

fn get_happiness_color(happiness: f64) -> String {
    match happiness {
        0.0..=30.0 => format!("{}", happiness.to_string().red()),
        31.0..=75.0 => format!("{}", happiness.to_string().yellow()),
        _ => format!("{}", happiness.to_string().green()),
    }
}
