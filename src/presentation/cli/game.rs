use crate::domain::Game;
use crate::infrastructure::api::bgg_client::BggClient;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Select};
use mantra_rust_macros::req;

/// Data transfer object (DTO) used to collect board game information interactively.
///
/// This struct serves as a temporary container for game details during user input
/// sessions in the CLI. Values collected here can later be mapped to the domain
/// [`Game`] entity and persisted using the appropriate repository.
///
/// # Fields
///
/// * `title` - The name of the board game as provided by the user.
/// * `year` - The release year of the game.
/// * `players` - The recommended player range, typically in the format `"min-max"`.
/// * `duration` - Average playtime in minutes.
/// * `age` - Recommended minimum age for players.
/// * `category` - The game's category or genre (e.g., "Strategy", "Family").
/// * `publisher` - Name of the publisher responsible for releasing the game.
/// * `authors` - A list of designer(s) or author(s) of the game.
/// * `rating` - Community or BGG rating, represented as a float (0.0 - 10.0).
pub struct InteractiveGameData {
    /// The title of the board game.
    pub title: String,
    /// The release year of the game.
    pub year: i32,
    /// Recommended number of players in `"min-max"` format.
    pub players: String,
    /// Average playtime in minutes.
    pub duration: i32,
    /// Minimum recommended age for players.
    pub age: i32,
    /// Category or genre of the game.
    pub category: String,
    /// Name of the publisher.
    pub publisher: String,
    /// List of authors/designers.
    pub authors: Vec<String>,
    /// Community or BoardGameGeek rating.
    pub rating: f32,
}

/// Orchestrates the interactive process for capturing new game data.
///
/// This function guides the user through a multi-step dialog:
/// 1. **Search**: Search for a game title.
/// 2. **Enrichment**: Optionally fetch high-quality metadata from BoardGameGeek.
/// 3. **Verification**: Allow the user to review and manually override any field.
///
/// # Arguments
/// * `bgg` - A reference to the [`BggClient`] used for external API lookups.
///
/// # Returns
/// A `Result` containing `Some(InteractiveGameData)` if completed, or `None` if aborted.
#[req("UC.1.REQ.001")]
#[req("UC.1.REQ.002")]
#[req("UC.1.REQ.003")]
#[req("UC.1.REQ.004")]
#[req("UC.1.REQ.005")]
#[req("UC.1.REQ.006")]
#[req("UC.1.REQ.007")]
#[req("UC.1.REQ.008")]
#[req("REQ.001")]
#[req("REQ.002")]
#[req("REQ.003")]
#[req("REQ.004")]
#[req("REQ.005")]
#[req("REQ.006")]
#[req("REQ.007")]
#[req("REQ.008")]
pub async fn run_interactive_capture(bgg: &BggClient) -> Result<Option<InteractiveGameData>> {
    let theme = ColorfulTheme::default();
    println!("\n--- Capture New Game (Empty input or Esc to cancel) ---");

    let search_term: String = Input::with_theme(&theme)
        .with_prompt("Search (Game Title)")
        .allow_empty(true)
        .interact_text()?;

    if search_term.trim().is_empty() {
        return Ok(None);
    }

    let mut official_title = search_term.clone();
    let mut bgg_authors = Vec::new();
    let mut bgg_rating = 0.0;
    let mut year_suggest = 2024;
    let mut publisher_suggest = String::new();
    let mut category_suggest = String::new();

    if Confirm::with_theme(&theme)
        .with_prompt("Fetch data from BoardGameGeek?")
        .default(true)
        .interact()?
    {
        println!("🔍 Searching BGG for '{}'...", search_term);
        let results = bgg.search(&search_term).await?;

        if !results.is_empty() {
            let display_items: Vec<String> = results
                .iter()
                .map(|i| {
                    format!(
                        "{} ({})",
                        i.name.value,
                        i.year.as_ref().map(|y| y.value.as_str()).unwrap_or("?")
                    )
                })
                .collect();

            let selection = FuzzySelect::with_theme(&theme)
                .with_prompt("Which game do you mean? (Esc to cancel)")
                .items(&display_items)
                .interact_opt()?;

            match selection {
                Some(idx) => {
                    let selected = &results[idx];
                    official_title = selected.name.value.clone();
                    year_suggest = selected
                        .year
                        .as_ref()
                        .and_then(|y| y.value.parse().ok())
                        .unwrap_or(2024);

                    println!("⏳ Fetching details for '{}'...", official_title);
                    let details = bgg.get_details(&selected.id).await?;

                    bgg_rating = details.rating;
                    bgg_authors = details.authors;
                    publisher_suggest = details.publisher.unwrap_or_default();
                    category_suggest = details.categories.first().cloned().unwrap_or_default();

                    println!("✅ BGG data successfully loaded.");
                }
                None => return Ok(None),
            }
        } else {
            println!("⚠️ No matches found on BGG. Using manual input.");
        }
    }

    println!("\n--- Data Verification (Empty input to cancel) ---");

    let final_title: String = Input::with_theme(&theme)
        .with_prompt("Confirm Title")
        .default(official_title)
        .allow_empty(true)
        .interact_text()?;
    if final_title.trim().is_empty() {
        return Ok(None);
    }

    let final_year: i32 = Input::with_theme(&theme)
        .with_prompt("Release Year")
        .default(year_suggest)
        .interact()?;

    let final_publisher: String = Input::with_theme(&theme)
        .with_prompt("Publisher")
        .default(publisher_suggest)
        .allow_empty(true)
        .interact_text()?;
    if final_publisher.trim().is_empty() {
        return Ok(None);
    }

    let final_category: String = Input::with_theme(&theme)
        .with_prompt("Category")
        .default(category_suggest)
        .allow_empty(true)
        .interact_text()?;
    if final_category.trim().is_empty() {
        return Ok(None);
    }

    let rating_input: String = Input::with_theme(&theme)
        .with_prompt("BGG Rating")
        .default(format!("{:.1}", bgg_rating))
        .interact_text()?;
    let final_rating: f32 = rating_input.parse().unwrap_or(bgg_rating);

    let mut final_authors = Vec::new();
    if !bgg_authors.is_empty() {
        println!("\nConfirm Authors (Empty input to cancel):");
        for b_author in bgg_authors {
            let edited: String = Input::with_theme(&theme)
                .with_prompt("Author")
                .default(b_author)
                .allow_empty(true)
                .interact_text()?;
            if edited.trim().is_empty() {
                return Ok(None);
            }
            final_authors.push(edited);
        }
    }

    while Confirm::with_theme(&theme)
        .with_prompt("Add another author manually?")
        .default(false)
        .interact()?
    {
        let new_author: String = Input::with_theme(&theme)
            .with_prompt("Author Name")
            .allow_empty(true)
            .interact_text()?;
        if new_author.trim().is_empty() {
            break;
        }
        final_authors.push(new_author);
    }

    let players: String = Input::with_theme(&theme)
        .with_prompt("Player Count")
        .default("1-4".into())
        .interact_text()?;

    let duration: i32 = Input::with_theme(&theme)
        .with_prompt("Duration (Min)")
        .default(60)
        .interact()?;

    let age: i32 = Input::with_theme(&theme)
        .with_prompt("Age Recommendation")
        .default(10)
        .interact()?;

    Ok(Some(InteractiveGameData {
        title: final_title,
        year: final_year,
        players,
        duration,
        age,
        category: final_category,
        publisher: final_publisher,
        authors: final_authors,
        rating: final_rating,
    }))
}

/// Renders a list of all registered games to the console in a bulleted format.
#[req("UC.9.REQ.060")]
#[req("REQ.060")]
pub fn display_game_list(games: &[Game]) {
    println!("\n--- Registered Games ---");
    if games.is_empty() {
        println!("No games found.");
        return;
    }
    for g in games {
        println!(
            "• {} ({}) | ⭐ {:.1} | 🏢 {} | 🏷️ {} | ✍️ {}",
            g.name,
            g.year,
            g.rating,
            g.publisher.as_deref().unwrap_or("N/A"),
            g.category.clone(),
            g.authors.join(", ")
        );
    }
}

/// Provides a selection menu to choose a specific game from a list.
pub fn select_game(games: &[Game]) -> Option<Game> {
    if games.is_empty() {
        println!("No games found.");
        return None;
    }
    let items: Vec<String> = games
        .iter()
        .map(|g| format!("{} ({})", g.name, g.year))
        .collect();
    let selection = Select::new()
        .with_prompt("Select a game")
        .items(&items)
        .interact_opt()
        .ok()??;
    Some(games[selection].clone())
}

/// Starts an interactive editor for an existing game's metadata.
pub fn run_interactive_game_edit(mut game: Game) -> Result<Option<Game>> {
    let theme = ColorfulTheme::default();
    println!("\n--- Edit Game: {} ---", game.name);

    game.name = Input::with_theme(&theme)
        .with_prompt("Title")
        .default(game.name)
        .interact_text()?;

    game.year = Input::with_theme(&theme)
        .with_prompt("Release Year")
        .default(game.year)
        .interact()?;

    let current_pub = game.publisher.clone().unwrap_or_default();
    let new_pub: String = Input::with_theme(&theme)
        .with_prompt("Publisher")
        .default(current_pub)
        .interact_text()?;
    game.publisher = if new_pub.trim().is_empty() {
        None
    } else {
        Some(new_pub)
    };

    let current_authors = game.authors.join(", ");
    let authors_input: String = Input::with_theme(&theme)
        .with_prompt("Authors (comma separated)")
        .default(current_authors)
        .interact_text()?;
    game.authors = authors_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    game.category = Input::with_theme(&theme)
        .with_prompt("Category")
        .default(game.category)
        .interact_text()?;

    game.age = Input::with_theme(&theme)
        .with_prompt("Age Recommendation")
        .default(game.age)
        .interact()?;

    game.players = Input::with_theme(&theme)
        .with_prompt("Player Count")
        .default(game.players)
        .interact_text()?;

    game.duration = Input::with_theme(&theme)
        .with_prompt("Duration (Min)")
        .default(game.duration)
        .interact()?;

    Ok(Some(game))
}
