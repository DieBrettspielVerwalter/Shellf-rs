use crate::domain::*;
use anyhow::Result;
use mantra_rust_macros::req;
use std::sync::Arc;

/// A use case responsible for creating and persisting a new game copy.
///
/// This struct orchestrates the creation of a `GameCopy` domain entity
/// and delegates persistence to the repository layer.
pub struct CreateGameCopyUseCase {
    /// The repository used to persist and manage game copy data.
    pub copy_repo: Arc<dyn GameCopyRepository>,
}

impl CreateGameCopyUseCase {
    /// Executes the business logic to create a new game copy.
    ///
    /// The process involves:
    /// - Constructing a new `GameCopy` entity with provided data.
    /// - Delegating persistence to the `GameCopyRepository`.
    ///
    /// # Arguments
    ///
    /// * `game_name` - The name of the game associated with the copy.
    /// * `game_year` - The release year of the game.
    ///
    /// # Returns
    ///
    /// Returns the generated unique identifier (`i32`) of the newly created game copy.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The repository fails to persist the game copy (e.g., database failure).
    #[req("UC.2")]
    pub async fn execute(&self, game_name: String, game_year: i32) -> Result<i32> {
        let new_copy = GameCopy {
            id: 0, // Die 0 ist nur ein Platzhalter
            game_name,
            game_year,
            ..Default::default()
        };
        let real_id = self.copy_repo.save(new_copy).await?;

        Ok(real_id)
    }
}

/// A use case responsible for deleting an existing game copy.
///
/// This struct handles the removal of a game copy from persistence
/// by delegating the operation to the repository layer.
pub struct DeleteGameCopyUseCase {
    /// The repository used to manage game copy persistence.
    pub copy_repo: Arc<dyn GameCopyRepository>,
}

impl DeleteGameCopyUseCase {
    /// Executes the business logic to delete a game copy.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the game copy to delete.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The repository fails to delete the game copy
    ///   (e.g., copy not found or database failure).
    pub async fn execute(&self, id: i32) -> Result<()> {
        self.copy_repo.delete(id).await
    }
}

/// A use case responsible for updating an existing game copy.
///
/// This struct coordinates updating game copy data by delegating
/// the operation to the repository layer.
pub struct EditGameCopyUseCase {
    /// The repository used to manage game copy persistence.
    pub copy_repo: Arc<dyn GameCopyRepository>,
}

impl EditGameCopyUseCase {
    /// Executes the business logic to update a game copy.
    ///
    /// # Arguments
    ///
    /// * `old_id` - The current identifier of the game copy to update.
    /// * `updated_copy` - The updated `GameCopy` entity containing new data.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The repository fails to perform the update
    ///   (e.g., copy not found or database failure).
    pub async fn execute(&self, old_id: i32, updated_copy: GameCopy) -> Result<()> {
        self.copy_repo.update(old_id, updated_copy).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::mockups::*;
    use anyhow::anyhow;
    use std::sync::Arc;
    use tokio;

    /// Verifies successful creation of a game copy.
    ///
    /// Ensures:
    /// - `save` is called once with correct GameCopy data.
    /// - The returned ID is propagated correctly.
    #[tokio::test]
    async fn create_game_copy_executes_successfully() {
        let mut repo = MockGameCopyRepository::new();

        repo.expect_save()
            .withf(|copy| copy.id == 0 && copy.game_name == "Zelda" && copy.game_year == 2023)
            .once()
            .returning(|_| Ok(42));

        let use_case = CreateGameCopyUseCase {
            copy_repo: Arc::new(repo),
        };

        let result = use_case.execute("Zelda".to_string(), 2023).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    /// Verifies that creation fails when repository save fails.
    #[tokio::test]
    async fn create_game_copy_fails_when_save_fails() {
        let mut repo = MockGameCopyRepository::new();

        repo.expect_save()
            .once()
            .returning(|_| Err(anyhow!("DB failure")));

        let use_case = CreateGameCopyUseCase {
            copy_repo: Arc::new(repo),
        };

        let result = use_case.execute("Zelda".to_string(), 2023).await;

        assert!(result.is_err());
    }

    /// Verifies successful deletion of a game copy.
    ///
    /// Ensures:
    /// - `delete` is called once with correct ID.
    /// - Returns Ok on success.
    #[tokio::test]
    async fn delete_game_copy_executes_successfully() {
        let mut repo = MockGameCopyRepository::new();

        repo.expect_delete()
            .with(mockall::predicate::eq(123))
            .once()
            .returning(|_| Ok(()));

        let use_case = DeleteGameCopyUseCase {
            copy_repo: Arc::new(repo),
        };

        let result = use_case.execute(123).await;

        assert!(result.is_ok());
    }

    /// Verifies that deletion errors are propagated.
    #[tokio::test]
    async fn delete_game_copy_fails_when_repo_fails() {
        let mut repo = MockGameCopyRepository::new();

        repo.expect_delete()
            .with(mockall::predicate::eq(123))
            .once()
            .returning(|_| Err(anyhow!("Delete failed")));

        let use_case = DeleteGameCopyUseCase {
            copy_repo: Arc::new(repo),
        };

        let result = use_case.execute(123).await;

        assert!(result.is_err());
    }

    /// Verifies successful update of a game copy.
    ///
    /// Ensures:
    /// - `update` is called with correct ID and updated entity.
    /// - Returns Ok on success.
    #[tokio::test]
    async fn edit_game_copy_executes_successfully() {
        let mut repo = MockGameCopyRepository::new();

        let updated = GameCopy {
            id: 999, // this is part of the struct, but repo uses old_id separately
            game_name: "Mario".to_string(),
            game_year: 2020,
            ..Default::default()
        };

        repo.expect_update()
            .withf(move |id, copy| {
                *id == 123 && copy.game_name == "Mario" && copy.game_year == 2020
            })
            .once()
            .returning(|_, _| Ok(()));

        let use_case = EditGameCopyUseCase {
            copy_repo: Arc::new(repo),
        };

        let result = use_case.execute(123, updated).await;

        assert!(result.is_ok());
    }

    /// Verifies that update errors are propagated.
    #[tokio::test]
    async fn edit_game_copy_fails_when_repo_fails() {
        let mut repo = MockGameCopyRepository::new();

        let updated = GameCopy {
            id: 999,
            game_name: "Mario".to_string(),
            game_year: 2020,
            ..Default::default()
        };

        repo.expect_update()
            .with(mockall::predicate::eq(123), mockall::predicate::always())
            .once()
            .returning(|_, _| Err(anyhow!("Update failed")));

        let use_case = EditGameCopyUseCase {
            copy_repo: Arc::new(repo),
        };

        let result = use_case.execute(123, updated).await;

        assert!(result.is_err());
    }
}
