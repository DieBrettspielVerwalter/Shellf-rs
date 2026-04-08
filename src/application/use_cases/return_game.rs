use crate::domain::repositories::game_copy_repository::GameCopyRepository;
use anyhow::Result;
use mantra_rust_macros::req;
use std::sync::Arc;

/// A use case responsible for processing the return of a lent game copy.
///
/// This orchestrates the update of a game copy's status to indicate it has
/// been returned to the library or owner.
pub struct ReturnGameUseCase {
    /// The repository used to update the return status of the game copy.
    pub copy_repo: Arc<dyn GameCopyRepository>,
}

impl ReturnGameUseCase {
    /// Executes the logic to return a game copy.
    ///
    /// This method parses the string-based identifier and triggers the
    /// update in the persistence layer.
    ///
    /// # Arguments
    ///
    /// * `copy_id` - A string slice representing the unique ID of the game copy to be returned.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The `copy_id` cannot be parsed into the required integer format.
    /// - The `copy_repo` fails to update the return status in the database.
    #[req("UC.4")]
    pub async fn execute(&self, copy_id: &str) -> Result<()> {
        let copy_id = copy_id.parse()?;
        self.copy_repo.return_copy(copy_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::mockups::*;
    use anyhow::anyhow;
    use mockall::predicate::*;
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn return_game_use_case_calls_repository_successfully() {
        let mut copy_repo = MockGameCopyRepository::new();
        copy_repo
            .expect_return_copy()
            .with(eq(123))
            .once()
            .returning(|_| Ok(()));

        let use_case = ReturnGameUseCase {
            copy_repo: Arc::new(copy_repo),
        };

        let result = use_case.execute("123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn return_game_use_case_propagates_repository_error() {
        let mut copy_repo = MockGameCopyRepository::new();
        copy_repo
            .expect_return_copy()
            .once()
            .returning(|_| Err(anyhow!("Database failure")));

        let use_case = ReturnGameUseCase {
            copy_repo: Arc::new(copy_repo),
        };

        let result = use_case.execute("456").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database failure"));
    }
}
