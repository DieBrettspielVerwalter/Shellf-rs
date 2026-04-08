use chrono::NaiveDate;

/// A domain entity representing a recorded game session.
///
/// This struct captures the details of a single gameplay session,
/// including the game copy used, the date of play, optional association
/// with a game night, and any recorded results.
///
/// It belongs to the domain layer and is independent of persistence
/// or presentation concerns.
#[derive(Debug, Clone, Default)]
pub struct GameSession {
    /// The unique identifier of the session.
    ///
    /// This is optional because it is typically assigned by the persistence layer.
    pub id: Option<i32>,

    /// The identifier of the game copy that was played.
    pub game_copy_id: i32,

    /// The date on which the session took place.
    pub date: NaiveDate,

    /// Optional reference to an associated game night.
    ///
    /// This allows linking a session to a planned event but is not required.
    pub _game_night_id: Option<i32>,

    /// Optional results or outcome of the session.
    ///
    /// This may include scores, rankings, or free-form notes.
    pub results: Option<String>,
}
