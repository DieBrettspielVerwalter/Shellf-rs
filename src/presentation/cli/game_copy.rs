use crate::domain::{Game, GameCopy, Player};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};
use mantra_rust_macros::req;

/// Starts the interactive dialog to register a new physical game copy in a player's collection.
///
/// This function allows the user to pick a base game from the existing database
/// to create a concrete instance (`GameCopy`).
///
/// # Arguments
/// * `games` - A slice of available [`Game`] metadata entities.
///
/// # Returns
/// A `Result` containing the tuple `(GameName, Year)` for use-case processing, or `None` if canceled.
#[req("UC.2.REQ.009")]
#[req("REQ.009")]
pub fn run_interactive_copy_creation(games: &[Game]) -> Result<Option<(String, i32)>> {
    let theme = ColorfulTheme::default();
    let names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();

    println!("\n--- Register Game Copy ---");
    let sel = FuzzySelect::with_theme(&theme)
        .with_prompt("Select the game from your collection (Esc to cancel)")
        .items(&names)
        .interact_opt()?;

    match sel {
        Some(idx) => {
            let selected_game = &games[idx];
            Ok(Some((selected_game.name.clone(), selected_game.year)))
        }
        None => Ok(None),
    }
}

/// Lists all physical game copies currently "on the shelf."
#[req("UC.9.REQ.062")]
#[req("REQ.062")]
pub fn display_copy_list(copies: &[GameCopy]) {
    println!("\n--- Shelf ---");
    if copies.is_empty() {
        println!("The shelf is currently empty.");
        return;
    }
    for c in copies {
        println!("• {} [ID: {}]", c.game_name, c.id);
    }
}

/// Displays a formatted table of all currently lent games.
///
/// Shows key lending information: the unique copy ID, the game title,
/// the borrower's email, and the date the lending started.
pub fn display_lent_copy_list(copies: &[GameCopy]) {
    println!(
        "\n{:<5} {:<25} {:<20} {:<12}",
        "ID", "Game", "Lent to", "Since"
    );
    println!("{}", "-".repeat(65));
    for c in copies {
        let borrower = c.borrower_email.as_deref().unwrap_or("-");
        let date = c
            .borrow_date
            .map(|d| d.format("%d.%m.%Y").to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<5} {:<25} {:<20} {:<12}",
            c.id, c.game_name, borrower, date
        );
    }
}

/// Provides a simple selection dialog to choose a specific game copy by its ID.
pub fn select_game_copy(copies: &[GameCopy]) -> Result<Option<String>> {
    let items: Vec<String> = copies
        .iter()
        .map(|c| format!("{} ({})", c.game_name, c.id))
        .collect();
    let sel = FuzzySelect::new()
        .with_prompt("Select copy")
        .items(&items)
        .interact_opt()?;
    Ok(sel.map(|i| copies[i].id.to_string()))
}

/// Filters the list for lent games and allows the user to select one for return.
pub fn select_lent_copy(copies: &[GameCopy]) -> Result<Option<String>> {
    let lent: Vec<&GameCopy> = copies.iter().filter(|c| c.is_lent).collect();
    let items: Vec<String> = lent
        .iter()
        .map(|c| format!("{} ({})", c.game_name, c.id))
        .collect();
    let sel = FuzzySelect::new()
        .with_prompt("Select return")
        .items(&items)
        .interact_opt()?;
    Ok(sel.map(|i| lent[i].id.to_string()))
}

/// Search dialog for a specific copy including detailed owner information.
pub fn select_game_copy_from_list(copies: &[GameCopy]) -> Option<GameCopy> {
    if copies.is_empty() {
        println!("No game copies on the shelf.");
        return None;
    }
    let items: Vec<String> = copies
        .iter()
        .map(|c| format!("{} (ID: {}, Owner: {})", c.game_name, c.id, c.owner_id))
        .collect();

    let selection = Select::new()
        .with_prompt("Which copy would you like to edit?")
        .items(&items)
        .interact_opt()
        .ok()??;

    Some(copies[selection].clone())
}

/// Starts an interactive editor for a game copy, allowing ownership transfers and ID corrections.
pub fn run_interactive_copy_edit(
    mut copy: GameCopy,
    players: &[Player],
) -> Result<Option<GameCopy>> {
    let theme = ColorfulTheme::default();
    println!("\n--- Edit Game Copy (ID: {}) ---", copy.id);

    copy.id = Input::with_theme(&theme)
        .with_prompt("New Copy ID")
        .default(copy.id)
        .interact()?;

    let player_names: Vec<String> = players
        .iter()
        .map(|p| format!("{} ({})", p.nickname, p.email))
        .collect();

    let default_idx = players
        .iter()
        .position(|p| p.email == copy.owner_id)
        .unwrap_or(0);

    let selection = Select::with_theme(&theme)
        .with_prompt("New Owner")
        .items(&player_names)
        .default(default_idx)
        .interact_opt()?;

    if let Some(idx) = selection {
        copy.owner_id = players[idx].email.clone();
        Ok(Some(copy))
    } else {
        Ok(None)
    }
}
