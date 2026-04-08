use super::common::input_date;
use crate::domain::{GameCopy, GameNight, Player};
use anyhow::Result;
use chrono::NaiveDate;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use mantra_rust_macros::req;

/// Data Transfer Object (DTO) used to collect information for planning a scheduled game night.
pub struct GameNightInputData {
    /// The intended calendar date for the event.
    pub date: NaiveDate,
    /// Optional administrative or social notes.
    pub notes: Option<String>,
    /// A collection of player identifiers (emails) attending the event.
    pub participants: Vec<String>,
    /// A collection of unique identifiers for the `GameCopy` entities suggested for play.
    pub suggested_copies: Vec<String>,
}

/// Starts the interactive multi-step dialog for planning a new game night.
///
/// This function coordinates three main input phases:
/// 1. **Date Selection**: Uses the shared `input_date` utility.
/// 2. **Participant Selection**: A `MultiSelect` interface to choose from registered players.
/// 3. **Game Selection**: A `MultiSelect` interface to choose available copies from the shelf.
///
/// # Arguments
///
/// * `players` - A slice of all registered [`Player`] entities.
/// * `copies` - A slice of all available [`GameCopy`] entities.
///
/// # Returns
///
/// A `Result` containing `Some(GameNightInputData)` if confirmed, or `None` if the user aborted (Esc).
#[req("UC.6.REQ.030")]
#[req("UC.6.REQ.031")]
#[req("UC.6.REQ.032")]
#[req("UC.6.REQ.033")]
#[req("REQ.030")]
#[req("REQ.031")]
#[req("REQ.032")]
#[req("REQ.033")]
pub fn run_interactive_game_night_planning(
    players: &[Player],
    copies: &[GameCopy],
) -> Result<Option<GameNightInputData>> {
    let theme = ColorfulTheme::default();
    println!("\n--- Plan Game Night (Esc to cancel) ---");

    // Standardize date input via the common helper
    let date = input_date("Date of the event", chrono::Local::now().date_naive())?;

    // Select participants using their nicknames for display
    let p_names: Vec<String> = players.iter().map(|p| p.nickname.clone()).collect();
    let p_sel = MultiSelect::with_theme(&theme)
        .with_prompt("Participants (Space to select, Enter to confirm, Esc to cancel)")
        .items(&p_names)
        .interact_opt()?;

    let participants = match p_sel {
        Some(indices) => indices.iter().map(|&i| players[i].email.clone()).collect(),
        None => return Ok(None),
    };

    // Select game suggestions from the current inventory
    let c_names: Vec<String> = copies
        .iter()
        .map(|c| format!("{} ({})", c.game_name, c.id))
        .collect();
    let c_sel = MultiSelect::with_theme(&theme)
        .with_prompt("Games on offer (Space to select, Enter to confirm, Esc to cancel)")
        .items(&c_names)
        .interact_opt()?;

    let suggested_copies = match c_sel {
        Some(indices) => indices.iter().map(|&i| copies[i].id.to_string()).collect(),
        None => return Ok(None),
    };

    Ok(Some(GameNightInputData {
        date,
        notes: None,
        participants,
        suggested_copies,
    }))
}

/// Renders a summary list of all scheduled game nights to the console.
///
/// Shows the date of the event and the total count of confirmed guests.
pub fn display_game_night_list(nights: &[GameNight]) {
    println!("\n--- Scheduled Nights ---");
    if nights.is_empty() {
        println!("No scheduled game nights found.");
        return;
    }
    for n in nights {
        println!("• {} | {} guests", n.date, n.participants.len());
    }
}
