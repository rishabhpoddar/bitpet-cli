use crate::pet;
use colored::*;
use crossterm::{ExecutableCommand, QueueableCommand};
use std::{
    io::{Write, stdout},
    time::Duration,
};

use crate::CommandResult;
const BOX_WIDTH: u16 = 45;
const BOX_HEIGHT: u16 = 10;

pub fn draw_image_starting_at(
    stdout: &mut std::io::Stdout,
    image: &str,
    start_x: u16,
    start_y: u16,
) -> CommandResult {
    stdout.queue(crossterm::cursor::MoveTo(start_x, start_y))?;
    let (orig_x, orig_y) = crossterm::cursor::position()?;

    for (i, line) in image.lines().enumerate() {
        stdout.queue(crossterm::cursor::MoveTo(orig_x, orig_y + i as u16))?;
        stdout.queue(crossterm::style::Print(line))?;
    }

    Ok(())
}

pub fn pad_image(image: &str) -> (String, usize, usize) {
    let max_width = image.lines().map(|line| line.len()).max().unwrap();
    let max_height = image.lines().count();
    let padded_face: Vec<String> = image
        .lines()
        .map(|line| {
            let len = line.len();
            if len < max_width {
                // Center pad with spaces
                let pad = (max_width - len) / 2;
                format!(
                    "{}{}{}",
                    " ".repeat(pad),
                    line,
                    " ".repeat(max_width - len - pad)
                )
            } else {
                line.to_string()
            }
        })
        .collect();

    (padded_face.join("\n"), max_width, max_height)
}

pub fn print_in_box<F>(
    mut render_in_box: F,
    max_number_of_frames: usize,
    fps: Option<u32>,
) -> CommandResult
where
    F: FnMut(&mut std::io::Stdout, u16, u16, u16, usize) -> CommandResult,
{
    let mut stdout = stdout();
    stdout.execute(crossterm::cursor::SavePosition)?;
    let (mut w, mut h) = crossterm::terminal::size().unwrap();
    let mut frame: usize = 0;
    let mut is_showing_error = false;
    while frame < max_number_of_frames {
        while crossterm::event::poll(Duration::from_secs(0))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                }
                _ => (),
            }
        }
        stdout.execute(crossterm::cursor::RestorePosition)?;
        if w < BOX_WIDTH || h < BOX_HEIGHT {
            if !is_showing_error {
                is_showing_error = true;
                stdout.execute(crossterm::cursor::SavePosition)?;
                stdout.execute(crossterm::style::Print(
                    "Error: Terminal too small to display your pet :(".red(),
                ))?;
            }
        } else {
            // TODO: In here, we are first drawing an empty box, and then drawing the pet in it which causes a flicker.. ideally, we want to form the whole box first, and then just draw once.
            is_showing_error = false;
            let horizontal_border = "─".repeat(BOX_WIDTH as usize - 2);
            stdout.queue(crossterm::style::Print(format!(
                "┌{}┐\n",
                horizontal_border
            )))?;
            for _ in 0..BOX_HEIGHT - 2 {
                stdout.queue(crossterm::style::Print(format!(
                    "│{}│\n",
                    " ".repeat(BOX_WIDTH as usize - 2)
                )))?;
            }
            stdout.queue(crossterm::style::Print(format!(
                "└{}┘\n",
                horizontal_border
            )))?;
            let curr_position_of_cursor = crossterm::cursor::position()?;
            stdout.queue(crossterm::cursor::MoveTo(
                0,
                curr_position_of_cursor.1 - BOX_HEIGHT,
            ))?;
            stdout.queue(crossterm::cursor::SavePosition)?;
            render_in_box(
                &mut stdout,
                curr_position_of_cursor.1 - BOX_HEIGHT,
                BOX_WIDTH as u16,
                BOX_HEIGHT as u16,
                frame,
            )?;
            stdout.flush()?;
        }

        std::thread::sleep(Duration::from_millis(1000 / fps.unwrap_or(60) as u64));
        frame += 1;
    }

    let mut dy = BOX_HEIGHT;
    if w < BOX_WIDTH || h < BOX_HEIGHT {
        dy = 2;
    }
    stdout.execute(crossterm::cursor::RestorePosition)?;
    let curr_position_of_cursor = crossterm::cursor::position()?;
    stdout.execute(crossterm::cursor::MoveTo(0, curr_position_of_cursor.1 + dy))?;
    Ok(())
}

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
