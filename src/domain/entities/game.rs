use serde::{Deserialize, Serialize};

/// Represents a board game entity within the domain.
///
/// This struct holds the core metadata for a game, including its mechanical
/// properties (players, duration, age) and its categorical information.
/// It is designed to be easily serializable for persistence and API exchange.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Game {
    /// The official title of the board game.
    pub name: String,

    /// The original release or publication year.
    pub year: i32,

    /// A descriptive string representing the supported player count (e.g., "2-4").
    pub players: String,

    /// The average playtime of a single session in minutes.
    pub duration: i32,

    /// The minimum recommended age for players.
    pub age: i32,

    /// The primary genre or category of the game (e.g., "Strategy", "Worker Placement").
    pub category: String,

    /// The name of the publishing company, if available.
    pub publisher: Option<String>,

    /// A collection of names representing the game's designers or authors.
    pub authors: Vec<String>,

    /// The numerical rating of the game, typically on a scale from 0.0 to 10.0.
    pub rating: f32,
}

#[cfg(test)]
impl Game {
    /// Creates a new `Game` instance for testing purposes.
    ///
    /// This helper constructor allows concise setup of fully populated
    /// `Game` entities in tests without relying on multiple field assignments.
    ///
    /// # Arguments
    ///
    /// * `name` - The official title of the game.
    /// * `year` - The publication year.
    /// * `players` - Supported player count description (e.g., "2-4").
    /// * `duration` - Average playtime in minutes.
    /// * `age` - Minimum recommended player age.
    /// * `category` - The genre or category of the game.
    /// * `publisher` - Optional publisher name.
    /// * `authors` - List of designers or authors.
    /// * `rating` - Numerical rating (e.g., 0.0–10.0 scale).
    ///
    /// # Returns
    ///
    /// A fully initialized `Game` instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        year: i32,
        players: String,
        duration: i32,
        age: i32,
        category: String,
        publisher: Option<String>,
        authors: Vec<String>,
        rating: f32,
    ) -> Self {
        Game {
            name,
            year,
            players,
            duration,
            age,
            category,
            publisher,
            authors,
            rating,
        }
    }
}
