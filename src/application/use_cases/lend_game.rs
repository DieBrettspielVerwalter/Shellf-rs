use crate::domain::repositories::game_copy_repository::GameCopyRepository;
use crate::domain::repositories::player_repository::PlayerRepository;
use anyhow::Result;
use chrono::NaiveDate;
use mantra_rust_macros::req;
use std::sync::Arc;

/// A use case responsible for lending a game copy to a player.
///
/// This struct coordinates the interaction between the application layer
/// and the persistence layer for lending operations. It ensures that the
/// correct repository method is invoked with properly transformed inputs.
pub struct LendGameUseCase {
    /// The repository used to manage game copy persistence and lending operations.
    pub copy_repo: Arc<dyn GameCopyRepository>,

    /// The repository used to access player data.
    ///
    /// Note: Currently not used in the implementation, but included for future
    /// validation or enrichment logic (e.g., verifying borrower existence).
    pub _player_repo: Arc<dyn PlayerRepository>,
}

// src/application/use_cases/lend_game.rs
impl LendGameUseCase {
    /// Executes the business logic to lend a game copy to a player.
    ///
    /// The process involves:
    /// - Parsing and validating the `copy_id`.
    /// - Delegating the lending operation to the `GameCopyRepository`.
    ///
    /// # Arguments
    ///
    /// * `copy_id` - The identifier of the game copy to be lent (string form, parsed internally).
    /// * `borrower_email` - The email address of the player borrowing the game.
    /// * `start_date` - The date when the lending period begins.
    /// * `due` - Optional due date for returning the game.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The `copy_id` cannot be parsed into a valid integer.
    /// - The repository fails to perform the lending operation
    ///   (e.g., copy not found, already lent, or database failure).
    #[req("UC.4")]
    pub async fn execute(
        &self,
        copy_id: &str,
        borrower_email: String,
        start_date: NaiveDate,
        due: Option<NaiveDate>,
    ) -> Result<()> {
        let id = copy_id.parse::<i32>()?;
        // Wir rufen direkt die lend_copy Methode des Repos auf
        self.copy_repo
            .lend_copy(id, &borrower_email, start_date, due)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::mockups::*;
    use anyhow::anyhow;
    use chrono::NaiveDate;
    use std::sync::Arc;
    use tokio;

    /// Verifies successful lending of a game copy.
    ///
    /// This test ensures that:
    /// - `GameCopyRepository::lend_copy` is called exactly once with correct parameters.
    /// - The use case returns `Ok(())` on success.
    /// This validates correct orchestration logic and guarantees that
    /// domain lookup and persistence interaction are correctly coordinated.
    #[tokio::test]
    async fn lend_game_use_case_executes_successfully() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Copy repository mock
        let mut copy_repo = MockGameCopyRepository::new();
        copy_repo
            .expect_lend_copy()
            .with(
                mockall::predicate::eq(123),
                mockall::predicate::eq("test@example.com"),
                mockall::predicate::eq(start_date),
                mockall::predicate::eq(None),
            )
            .once()
            .returning(|_, _, _, _| Ok(()));

        let use_case = LendGameUseCase {
            copy_repo: Arc::new(copy_repo),
            _player_repo: Arc::new(MockPlayerRepository::new()), // not used in current implementation
        };

        let result = use_case
            .execute("123", "test@example.com".to_string(), start_date, None)
            .await;

        assert!(result.is_ok());
    }

    /// Verifies that lending fails if the copy ID cannot be parsed.
    ///
    /// This test ensures that:
    /// - Invalid copy ID input is caught and returned as an error.
    #[tokio::test]
    async fn lend_game_use_case_fails_with_invalid_copy_id() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let use_case = LendGameUseCase {
            copy_repo: Arc::new(MockGameCopyRepository::new()),
            _player_repo: Arc::new(MockPlayerRepository::new()),
        };

        let result = use_case
            .execute(
                "abc", // invalid integer
                "test@example.com".to_string(),
                start_date,
                None,
            )
            .await;

        assert!(result.is_err());
    }

    /// Verifies that persistence failures during lending are propagated.
    ///
    /// This test ensures that:
    /// - `GameCopyRepository::lend_copy` returns an error.
    /// - The use case propagates the repository error.
    #[tokio::test]
    async fn lend_game_use_case_fails_when_lend_copy_fails() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let mut copy_repo = MockGameCopyRepository::new();
        copy_repo
            .expect_lend_copy()
            .with(
                mockall::predicate::eq(123),
                mockall::predicate::eq("test@example.com"),
                mockall::predicate::eq(start_date),
                mockall::predicate::eq(None),
            )
            .once()
            .returning(|_, _, _, _| Err(anyhow!("Database write failed")));

        let use_case = LendGameUseCase {
            copy_repo: Arc::new(copy_repo),
            _player_repo: Arc::new(MockPlayerRepository::new()),
        };

        let result = use_case
            .execute("123", "test@example.com".to_string(), start_date, None)
            .await;

        assert!(result.is_err());
    }
}
