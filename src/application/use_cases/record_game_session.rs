use crate::domain::{GameSession, GameSessionRepository};
use anyhow::Result;
use chrono::NaiveDate;
use mantra_rust_macros::req;
use std::sync::Arc;

/// A request object representing the input required to record a game session.
///
/// This struct belongs to the application layer and serves as a boundary
/// between external inputs (e.g., CLI, API) and the domain logic. It ensures
/// that all required data is explicitly provided in a structured format.
pub struct RecordGameSessionRequest {
    /// The identifier of the game copy being played.
    pub game_copy_id: i32, // Hier nutzen wir direkt i32 statt String

    /// The date on which the session took place.
    pub date: NaiveDate,

    /// A list of participants (e.g., player email addresses).
    pub participants: Vec<String>,

    /// Optional results or outcome of the session.
    pub results: Option<String>,
}

/// A use case responsible for recording and persisting a game session.
///
/// This struct orchestrates the creation of a `GameSession` domain entity
/// and delegates persistence to the repository layer, including associated
/// participants.
pub struct RecordGameSessionUseCase {
    /// The repository used to persist and manage game session data.
    pub session_repo: Arc<dyn GameSessionRepository>,
}

impl RecordGameSessionUseCase {
    /// Executes the business logic to record a game session.
    ///
    /// The process involves:
    /// - Constructing a `GameSession` domain entity from the request.
    /// - Delegating persistence to the `GameSessionRepository`.
    /// - Passing along participant data for association.
    ///
    /// # Arguments
    ///
    /// * `request` - A `RecordGameSessionRequest` containing all required input data.
    ///
    /// # Returns
    ///
    /// Returns the generated unique identifier (`i32`) of the newly recorded session.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The repository fails to persist the session
    ///   (e.g., database failure or connectivity issues).
    #[req("UC.7")]
    pub async fn execute(&self, request: RecordGameSessionRequest) -> Result<i32> {
        let session = GameSession {
            id: None,
            game_copy_id: request.game_copy_id,
            date: request.date,
            _game_night_id: None,
            results: request.results,
        };

        // Wir reichen die Teilnehmerliste an das Repo weiter
        self.session_repo.save(session, request.participants).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::mockups::*;
    use anyhow::anyhow;
    use chrono::NaiveDate;
    use std::sync::Arc;

    /// Verifies successful recording of a game session.
    ///
    /// This test ensures that:
    /// - A `GameSession` entity is constructed correctly.
    /// - The repository `save` method is called exactly once.
    /// - The use case returns the session ID on success.
    #[tokio::test]
    async fn record_game_session_use_case_saves_session_successfully() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        let participants = vec![
            "player1@example.com".to_string(),
            "player2@example.com".to_string(),
        ];
        let participants_clone = participants.clone();

        let mut repo = MockGameSessionRepository::new();
        repo.expect_save()
            .withf(move |session, parts| {
                session.game_copy_id == 123 && session.date == date && parts == &participants
            })
            .once()
            .returning(|_, _| Box::pin(async { Ok(999) }));

        let use_case = RecordGameSessionUseCase {
            session_repo: Arc::new(repo),
        };

        let input = RecordGameSessionRequest {
            game_copy_id: 123,
            date,
            participants: participants_clone,
            results: None,
        };

        let result = use_case.execute(input).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 999);
    }

    /// Verifies that persistence failure is properly propagated.
    ///
    /// This test ensures that:
    /// - The repository `save` method returns an error.
    /// - The use case propagates the error.
    #[tokio::test]
    async fn record_game_session_use_case_propagates_save_failure() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

        let mut repo = MockGameSessionRepository::new();
        repo.expect_save()
            .once()
            .returning(|_, _| Box::pin(async { Err(anyhow!("DB write failed")) }));

        let use_case = RecordGameSessionUseCase {
            session_repo: Arc::new(repo),
        };

        let input = RecordGameSessionRequest {
            game_copy_id: 123,
            date,
            participants: vec![],
            results: None,
        };

        let result = use_case.execute(input).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DB write failed"));
    }

    /// Verifies that sessions can be recorded without a game night reference.
    ///
    /// This test ensures that:
    /// - `game_night_id` may be `None`.
    /// - Empty participants are allowed.
    /// - The repository is still called exactly once.
    #[tokio::test]
    async fn record_game_session_use_case_allows_none_game_night_and_empty_participants() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

        let mut repo = MockGameSessionRepository::new();
        repo.expect_save()
            .withf(|session, parts| session._game_night_id.is_none() && parts.is_empty())
            .once()
            .returning(|_, _| Box::pin(async { Ok(101) }));

        let use_case = RecordGameSessionUseCase {
            session_repo: Arc::new(repo),
        };

        let input = RecordGameSessionRequest {
            game_copy_id: 456,
            date,
            participants: vec![],
            results: None,
        };

        let result = use_case.execute(input).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 101);
    }

    /// Verifies that an invalid `game_copy_id` string causes an error.
    #[tokio::test]
    async fn record_game_session_use_case_fails_with_invalid_copy_id() {
        use chrono::NaiveDate;
        use std::sync::Arc;

        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

        let mut repo = MockGameSessionRepository::new();

        // Mock save to return an error if game_copy_id is invalid
        repo.expect_save()
            .withf(|session, _| session.game_copy_id < 0)
            .returning(|_, _| Box::pin(async { Err(anyhow::anyhow!("Invalid game copy ID")) }));

        let use_case = RecordGameSessionUseCase {
            session_repo: Arc::new(repo),
        };

        let input = RecordGameSessionRequest {
            game_copy_id: -1, // invalid
            date,
            participants: vec![],
            results: None,
        };

        let result = use_case.execute(input).await;

        // The use case should return an error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid game copy ID");
    }
}
