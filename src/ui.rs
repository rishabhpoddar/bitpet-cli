use crate::pet;
use crossterm::style::{Color, Stylize};
use crossterm::{ExecutableCommand, QueueableCommand};
use std::{
    io::{Write, stdout},
    time::Duration,
};

use crate::error::CustomErrorTrait;

use crate::CommandResult;
const BOX_WIDTH: u16 = 45;
const BOX_HEIGHT: u16 = 10;

fn hex_to_rgb(hex: &str) -> Option<Color> {
    // expect "#RRGGBB"
    if !hex.starts_with('#') || hex.len() != 7 {
        return None;
    }
    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
    Some(Color::Rgb { r, g, b })
}

pub struct ImageDrawnArea {
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
}

pub fn draw_image_starting_at(
    stdout: &mut std::io::Stdout,
    image: &str,
    colours: &Vec<Vec<String>>,
    start_x: u16,
    start_y: u16,
) -> Result<ImageDrawnArea, Box<dyn CustomErrorTrait>> {
    let mut colourised_image = String::new();
    {
        for (i, line) in image.lines().enumerate() {
            let colour_line = colours.get(i).unwrap();
            let mut curr_line = String::new();
            for (j, ch) in line.chars().enumerate() {
                let hex = colour_line.get(j).unwrap();
                let styled = if hex == "" {
                    ch.to_string()
                } else if let Some(rgb) = hex_to_rgb(hex) {
                    ch.to_string().with(rgb).to_string()
                } else {
                    ch.to_string()
                };
                curr_line.push_str(&styled);
            }
            colourised_image.push_str(&curr_line);
            colourised_image.push('\n');
        }
    }

    stdout.queue(crossterm::cursor::MoveTo(start_x, start_y))?;
    let (orig_x, orig_y) = crossterm::cursor::position()?;

    for (i, line) in colourised_image.lines().enumerate() {
        stdout.queue(crossterm::cursor::MoveTo(orig_x, orig_y + i as u16))?;
        stdout.queue(crossterm::style::Print(line))?;
    }

    Ok(ImageDrawnArea {
        start_x,
        start_y,
        width: image.lines().map(|line| line.len()).max().unwrap() as u16,
        height: image.lines().count() as u16,
    })
}

pub fn pad_image_and_colours(
    image: String,
    colours: Vec<Vec<String>>,
    padding_char: Option<char>,
    default_colour: Option<String>,
) -> (String, Vec<Vec<String>>, usize, usize) {
    let lines: Vec<&str> = image.lines().collect();
    let max_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let max_height = lines.len();

    let mut padded_face = Vec::with_capacity(max_height);
    let mut padded_colours = Vec::with_capacity(max_height);

    for (i, line) in lines.iter().enumerate() {
        let line_len = line.len();
        let pad = (max_width - line_len) / 2;

        // pad face line
        let face_line = format!(
            "{}{}{}",
            padding_char.unwrap_or(' ').to_string().repeat(pad),
            line,
            padding_char
                .unwrap_or(' ')
                .to_string()
                .repeat(max_width - line_len - pad)
        );
        padded_face.push(face_line);

        // pad colour line
        let mut colour_line = vec![default_colour.clone().unwrap_or("".to_string()); pad];
        let input_colours = colours.get(i).cloned().unwrap_or_default();
        colour_line.extend((0..line_len).map(|j| {
            input_colours
                .get(j)
                .cloned()
                .unwrap_or(default_colour.clone().unwrap_or("".to_string()))
        }));
        colour_line.resize(max_width, default_colour.clone().unwrap_or("".to_string()));
        padded_colours.push(colour_line);
    }

    (
        padded_face.join("\n"),
        padded_colours,
        max_width,
        max_height,
    )
}

pub fn print_in_box<F>(
    mut render_in_box: F,
    max_number_of_frames: usize,
    fps: Option<u32>,
) -> CommandResult
where
    F: FnMut(
        &mut std::io::Stdout,
        u16,
        u16,
        u16,
        usize,
    ) -> Result<ImageDrawnArea, Box<dyn CustomErrorTrait>>,
{
    let mut stdout = stdout();
    stdout.execute(crossterm::cursor::Hide)?;
    stdout.execute(crossterm::cursor::SavePosition)?;
    let (mut w, mut h) = crossterm::terminal::size().unwrap();
    let mut frame: usize = 0;
    let mut is_showing_error = false;
    let mut older_image_drawn_area: Option<ImageDrawnArea> = None;
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
        if w <= BOX_WIDTH || h <= BOX_HEIGHT {
            if !is_showing_error {
                is_showing_error = true;
                stdout.execute(crossterm::cursor::SavePosition)?;
                stdout.execute(crossterm::style::Print(colored::Colorize::red(
                    "Error: Terminal too small to display your pet :(",
                )))?;
            }
        } else {
            is_showing_error = false;
            let horizontal_border = "─".repeat(BOX_WIDTH as usize - 2);
            stdout.queue(crossterm::style::Print(format!(
                "┌{}┐\n",
                horizontal_border
            )))?;
            for _ in 0..BOX_HEIGHT - 2 {
                stdout.queue(crossterm::style::Print("│"))?;
                stdout.queue(crossterm::cursor::MoveRight(BOX_WIDTH as u16 - 2))?;
                stdout.queue(crossterm::style::Print("│\n"))?;
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
            let image_drawn_area = render_in_box(
                &mut stdout,
                curr_position_of_cursor.1 - BOX_HEIGHT,
                BOX_WIDTH as u16,
                BOX_HEIGHT as u16,
                frame,
            )?;
            if let Some(older_area) = older_image_drawn_area {
                let mut areas_to_clear: Vec<(u16, u16)> = Vec::new();

                for y in older_area.start_y..older_area.start_y + older_area.height {
                    for x in older_area.start_x..older_area.start_x + older_area.width {
                        if x < image_drawn_area.start_x
                            || x >= image_drawn_area.start_x + image_drawn_area.width
                            || y < image_drawn_area.start_y
                            || y >= image_drawn_area.start_y + image_drawn_area.height
                        {
                            areas_to_clear.push((x, y));
                        }
                    }
                }

                for to_clear in areas_to_clear {
                    stdout.queue(crossterm::cursor::MoveTo(to_clear.0, to_clear.1))?;
                    stdout.queue(crossterm::style::Print(" "))?;
                }
            }
            older_image_drawn_area = Some(image_drawn_area);
            stdout.flush()?;
        }

        std::thread::sleep(Duration::from_millis(1000 / fps.unwrap_or(60) as u64));
        frame += 1;
    }

    let mut dy = BOX_HEIGHT;
    if w <= BOX_WIDTH || h <= BOX_HEIGHT {
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
