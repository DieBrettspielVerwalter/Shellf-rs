use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A domain entity representing a physical or digital copy of a game.
///
/// This struct models the core state of a game copy within the system,
/// including ownership, lending status, and associated metadata.
/// It is part of the domain layer and should remain free of infrastructure concerns.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GameCopy {
    /// The unique identifier of the game copy.
    pub id: i32,

    /// The name of the game.
    pub game_name: String,

    /// The release year of the game.
    pub game_year: i32,

    /// The identifier of the owner of this game copy.
    pub owner_id: String,

    /// Indicates whether the game copy is currently lent out.
    pub is_lent: bool,

    /// The email address of the current borrower, if the game is lent.
    pub borrower_email: Option<String>, // Email des aktuellen Ausleihers

    /// The date when the game was borrowed.
    pub borrow_date: Option<NaiveDate>, // Wann wurde es ausgeliehen?

    /// The due date by which the game should be returned.
    pub due_date: Option<NaiveDate>, // Bis wann soll es zurück sein?
}

#[cfg(test)]
impl GameCopy {
    /// Creates a new `GameCopy` instance for testing purposes.
    ///
    /// This helper constructor simplifies test setup by initializing
    /// a non-lent game copy with default lending-related fields.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the game copy.
    /// * `game_name` - The name of the game.
    /// * `game_year` - The release year of the game.
    /// * `owner_id` - The identifier of the owner.
    ///
    /// # Returns
    ///
    /// A `GameCopy` instance with default lending state (`not lent`).
    pub fn new(id: i32, game_name: String, game_year: i32, owner_id: String) -> Self {
        GameCopy {
            id,
            game_name,
            game_year,
            owner_id,
            is_lent: false,
            borrower_email: None,
            borrow_date: None,
            due_date: None,
        }
    }
}
