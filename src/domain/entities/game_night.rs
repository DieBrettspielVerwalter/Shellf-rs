use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A domain entity representing a planned game night event.
///
/// This struct models the essential data for organizing a game night,
/// including scheduling, participants, and suggested games.
/// It belongs to the domain layer and contains no infrastructure-specific logic.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct GameNight {
    /// The unique identifier of the game night.
    pub id: i32,

    /// The scheduled date of the game night.
    pub date: NaiveDate,

    /// Optional notes or additional information about the event.
    pub notes: Option<String>,

    /// A list of participants (e.g., player email addresses).
    pub participants: Vec<String>,

    /// A list of suggested game copy IDs for the session.
    pub suggested_copies: Vec<i32>,
}
