use super::common::input_date;
use crate::domain::{GameCopy, GameNight, GameSession, Player};
use anyhow::Result;
use chrono::NaiveDate;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, MultiSelect, Select};
use mantra_rust_macros::req;
use uuid::Uuid;

/// Data Transfer Object (DTO) used to transport captured session data from the CLI to the Application Layer.
pub struct GameSessionInputData {
    /// The unique identifier of the physical game copy that was played.
    pub game_copy_id: String,
    /// The date when the session took place.
    pub date: NaiveDate,
    /// A collection of player identifiers (emails) who participated.
    pub participants: Vec<String>,
    /// Optional reference to a scheduled game night event.
    pub _game_night_id: Option<Uuid>,
    /// Raw result or ranking string (e.g., "1" for the winner).
    pub _results: String,
}

/// Orchestrates the interactive dialog for recording a new game session.
///
/// This function ensures referential integrity by requiring an existing [`GameCopy`].
/// It allows users to pick a game from their shelf and select multiple participants
/// from the player database.
///
/// # Returns
/// A `Result` containing `Some(GameSessionInputData)` if successful, or `None` if aborted.
#[req("UC.7.REQ.040")]
#[req("UC.7.REQ.042")]
#[req("UC.7.REQ.043")]
#[req("UC.8.REQ.050")]
#[req("UC.8.REQ.054")]
#[req("REQ.040")]
#[req("REQ.042")]
#[req("REQ.043")]
#[req("REQ.050")]
#[req("REQ.054")]
pub fn run_interactive_game_session_capture(
    copies: &[GameCopy],
    players: &[Player],
    _nights: &[GameNight],
) -> Result<Option<GameSessionInputData>> {
    if copies.is_empty() {
        println!("\n Your shelf is empty. Please register a game copy first.");
        return Ok(None);
    }

    let theme = ColorfulTheme::default();
    let c_names: Vec<String> = copies
        .iter()
        .map(|c| format!("{} (ID: {})", c.game_name, c.id))
        .collect();

    println!("\n--- Record New Session ---");
    let selection = FuzzySelect::with_theme(&theme)
        .with_prompt("Which copy was played? (Esc to cancel)")
        .items(&c_names)
        .interact_opt()?;

    match selection {
        Some(idx) => {
            let game_copy_id = copies[idx].id.to_string();
            let date = input_date("Session Date", chrono::Local::now().date_naive())?;

            let p_names: Vec<String> = players
                .iter()
                .map(|p| format!("{} ({})", p.nickname, p.email))
                .collect();

            let p_sel = MultiSelect::with_theme(&theme)
                .with_prompt("Who participated? (Space to toggle, Enter to confirm)")
                .items(&p_names)
                .interact()?;

            let participants = p_sel.iter().map(|&i| players[i].email.clone()).collect();

            let results: String = Input::with_theme(&theme)
                .with_prompt("Result/Rank (Number)")
                .allow_empty(true)
                .interact_text()?;

            Ok(Some(GameSessionInputData {
                game_copy_id,
                date,
                participants,
                _game_night_id: None,
                _results: results,
            }))
        }
        None => Ok(None),
    }
}

/// Renders a historical list of all recorded game sessions to the console.
pub fn display_game_session_list(sessions: &[GameSession]) {
    println!("\n--- Session History ---");
    if sessions.is_empty() {
        println!("No sessions recorded yet.");
        return;
    }
    for s in sessions {
        println!(
            "Date: {} | Copy-ID: {} | Session-ID: {:?}",
            s.date, s.game_copy_id, s.id
        );
    }
}

/// Provides a selection dialog to choose a session from the history for further actions.
pub fn select_game_session(sessions: &[GameSession]) -> Option<GameSession> {
    if sessions.is_empty() {
        return None;
    }
    let items: Vec<String> = sessions
        .iter()
        .map(|s| {
            format!(
                "ID: {} | Date: {} | Copy: {}",
                s.id.unwrap_or(0),
                s.date,
                s.game_copy_id
            )
        })
        .collect();

    let selection = Select::new()
        .with_prompt("Select Session")
        .items(&items)
        .interact_opt()
        .ok()??;
    Some(sessions[selection].clone())
}

/// Starts an interactive editor for an existing session.
///
/// This dialog is state-aware: it uses the `current_participants` to pre-select
/// the players in the `MultiSelect` list, providing a seamless editing experience.
///
/// # Arguments
/// * `session` - The current [`GameSession`] entity.
/// * `current_participants` - List of player emails currently associated with this session.
/// * `copies` - List of all physical copies for potential reassignment.
/// * `players` - List of all registered players for participant selection.
pub fn run_interactive_game_session_edit(
    mut session: GameSession,
    current_participants: Vec<String>,
    copies: &[GameCopy],
    players: &[Player],
) -> Result<Option<(GameSession, Vec<String>)>> {
    let theme = ColorfulTheme::default();
    println!("\n--- Edit Session (ID: {:?}) ---", session.id);

    let c_names: Vec<String> = copies
        .iter()
        .map(|c| format!("{} (ID: {})", c.game_name, c.id))
        .collect();
    let default_c = copies
        .iter()
        .position(|c| c.id == session.game_copy_id)
        .unwrap_or(0);
    let c_idx = Select::with_theme(&theme)
        .with_prompt("Game Copy")
        .items(&c_names)
        .default(default_c)
        .interact()?;
    session.game_copy_id = copies[c_idx].id;

    session.date = input_date("Date", session.date)?;

    // Synchronizing participant state for UI pre-selection
    let p_names: Vec<String> = players
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.email))
        .collect();
    let defaults: Vec<bool> = players
        .iter()
        .map(|p| current_participants.contains(&p.email))
        .collect();

    let p_sel = MultiSelect::with_theme(&theme)
        .with_prompt("Participants")
        .items(&p_names)
        .defaults(&defaults)
        .interact()?;
    let participants = p_sel.iter().map(|&i| players[i].email.clone()).collect();

    let res = session.results.clone().unwrap_or_default();
    let new_res: String = Input::with_theme(&theme)
        .with_prompt("Rank (Number)")
        .default(res)
        .interact_text()?;
    session.results = Some(new_res);

    Ok(Some((session, participants)))
}
