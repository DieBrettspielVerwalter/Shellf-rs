use crate::domain::Player;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Password, Select};
use mantra_rust_macros::req;

/// Data Transfer Object (DTO) used to collect comprehensive player information,
/// including infrastructure-level credentials.
pub struct PlayerInputData {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: String,
    pub details: Option<String>,
    pub password: String,
}

/// Orchestrates the interactive dialog for creating a new player and their
/// corresponding database user.
///
/// This function implements inline validation for the email address and
/// ensures credential integrity through a confirmed password prompt.
///
/// # Returns
/// A `Result` containing `Some(PlayerInputData)` if the process was completed,
/// or `None` if the user aborted.
#[req("UC.3.REQ.014")]
#[req("UC.3.REQ.015")]
#[req("UC.3.REQ.016")]
#[req("UC.3.REQ.018")]
#[req("REQ.014")]
#[req("REQ.015")]
#[req("REQ.016")]
#[req("REQ.018")]
pub fn run_interactive_player_creation() -> Result<Option<PlayerInputData>> {
    let theme = ColorfulTheme::default();
    println!("\n=== CREATE NEW PLAYER ===");

    // Email input with immediate format validation
    let email: String = Input::with_theme(&theme)
        .with_prompt("E-Mail Address")
        .allow_empty(true)
        .validate_with(|input: &String| {
            if input.contains('@') || input.is_empty() {
                Ok(())
            } else {
                Err("Invalid email address (must contain '@')")
            }
        })
        .interact_text()?;

    if email.is_empty() {
        return Ok(None);
    }

    // Password capture with confirmation logic
    let password = Password::with_theme(&theme)
        .with_prompt("Set Database Password")
        .with_confirmation("Confirm Password", "Passwords do not match!")
        .allow_empty_password(true)
        .interact()?;

    let first_name: String = Input::with_theme(&theme)
        .with_prompt("First Name")
        .interact_text()?;

    let last_name: String = Input::with_theme(&theme)
        .with_prompt("Last Name")
        .interact_text()?;

    let nickname: String = Input::with_theme(&theme)
        .with_prompt("Nickname")
        .interact_text()?;

    let details: String = Input::with_theme(&theme)
        .with_prompt("Notes (optional)")
        .allow_empty(true)
        .interact_text()?;

    let details_opt = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    Ok(Some(PlayerInputData {
        email,
        first_name,
        last_name,
        nickname,
        details: details_opt,
        password,
    }))
}

/// Renders a list of all registered player nicknames to the console.
#[req("UC.9.REQ.061")]
#[req("REQ.061")]
pub fn display_player_list(players: &[Player]) {
    println!("\n--- Player List ---");
    if players.is_empty() {
        println!("No players registered.");
        return;
    }
    for p in players {
        println!("• {}", p.nickname);
    }
}

/// Provides a selection menu to pick a player from a list, typically for editing purposes.
pub fn select_player(players: &[Player]) -> Option<Player> {
    if players.is_empty() {
        println!("No players registered.");
        return None;
    }
    let items: Vec<String> = players
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.email))
        .collect();

    let selection = Select::new()
        .with_prompt("Which player would you like to edit?")
        .items(&items)
        .interact_opt()
        .ok()??;

    Some(players[selection].clone())
}

/// Specialized selection dialog for the lending process using fuzzy search.
///
/// # Returns
/// A `Result` containing the selected player's email address as a unique identifier.
pub fn select_borrower(players: &[Player]) -> Result<Option<String>> {
    let items: Vec<String> = players
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.email))
        .collect();

    let sel = FuzzySelect::new()
        .with_prompt("Select Borrower")
        .items(&items)
        .interact_opt()?;

    Ok(sel.map(|i| players[i].email.clone()))
}

/// Starts an interactive editor for updating a player's nickname.
pub fn run_interactive_player_edit(mut player: Player) -> Result<Option<Player>> {
    let theme = ColorfulTheme::default();
    println!("\n--- Edit Player: {} ---", player.email);

    player.nickname = Input::with_theme(&theme)
        .with_prompt("New Nickname")
        .default(player.nickname)
        .interact_text()?;

    Ok(Some(player))
}
