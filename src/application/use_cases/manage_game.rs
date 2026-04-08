use crate::domain::entities::game::Game;
use crate::domain::repositories::game_repository::GameRepository;
use anyhow::Result;
use std::sync::Arc;

/// A use case responsible for removing a game from the system.
///
/// This struct acts as an application service that coordinates the deletion
/// of a game entity identified by its unique business keys (name and year).
pub struct DeleteGameUseCase {
    /// The repository used to perform the deletion in the persistence layer.
    pub game_repo: Arc<dyn GameRepository>,
}

impl DeleteGameUseCase {
    /// Executes the deletion of a specific game.
    ///
    /// # Arguments
    ///
    /// * `name` - The title of the game to be deleted.
    /// * `year` - The release year of the game to be deleted.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the repository fails to delete the record,
    /// for example, due to database connection issues or if the game does not exist.
    pub async fn execute(&self, name: String, year: i32) -> Result<()> {
        self.game_repo.delete(&name, year).await
    }
}

/// A use case responsible for updating an existing game's information.
///
/// This struct manages the replacement or modification of a game entity,
/// using the original name and year as identifiers.
pub struct EditGameUseCase {
    /// The repository used to persist the updated game data.
    pub game_repo: Arc<dyn GameRepository>,
}

impl EditGameUseCase {
    /// Updates an existing game with new data.
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current name of the game used to locate the record.
    /// * `old_year` - The current release year of the game used to locate the record.
    /// * `updated_game` - The new `Game` entity containing the updated information.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the update operation fails in the
    /// `game_repo` or if the specified game cannot be found.
    pub async fn execute(&self, old_name: String, old_year: i32, updated_game: Game) -> Result<()> {
        self.game_repo
            .update(&old_name, old_year, updated_game)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mockups::MockGameRepository;
    use anyhow::anyhow;
    use std::sync::Arc;
    use tokio;

    /// Verifies successful deletion of a game.
    ///
    /// Ensures:
    /// - `GameRepository::delete` is called once with correct parameters.
    /// - The use case returns `Ok(())`.
    #[tokio::test]
    async fn delete_game_executes_successfully() {
        let mut repo = MockGameRepository::new();

        repo.expect_delete()
            .with(
                mockall::predicate::eq("Zelda"),
                mockall::predicate::eq(2023),
            )
            .once()
            .returning(|_, _| Ok(()));

        let use_case = DeleteGameUseCase {
            game_repo: Arc::new(repo),
        };

        let result = use_case.execute("Zelda".to_string(), 2023).await;

        assert!(result.is_ok());
    }

    /// Verifies that deletion errors are propagated.
    #[tokio::test]
    async fn delete_game_fails_when_repo_fails() {
        let mut repo = MockGameRepository::new();

        repo.expect_delete()
            .with(
                mockall::predicate::eq("Zelda"),
                mockall::predicate::eq(2023),
            )
            .once()
            .returning(|_, _| Err(anyhow!("Delete failed")));

        let use_case = DeleteGameUseCase {
            game_repo: Arc::new(repo),
        };

        let result = use_case.execute("Zelda".to_string(), 2023).await;

        assert!(result.is_err());
    }

    /// Verifies successful update of a game.
    ///
    /// Ensures:
    /// - `GameRepository::update` is called with correct identifiers and updated entity.
    /// - The use case returns `Ok(())`.
    #[tokio::test]
    async fn edit_game_executes_successfully() {
        let mut repo = MockGameRepository::new();

        let updated_game = Game {
            name: "Zelda: TOTK".to_string(),
            year: 2023,
            ..Default::default()
        };

        repo.expect_update()
            .withf(move |name, year, game| {
                name == "Zelda" && *year == 2020 && game.name == "Zelda: TOTK" && game.year == 2023
            })
            .once()
            .returning(|_, _, _| Ok(()));

        let use_case = EditGameUseCase {
            game_repo: Arc::new(repo),
        };

        let result = use_case
            .execute("Zelda".to_string(), 2020, updated_game)
            .await;

        assert!(result.is_ok());
    }

    /// Verifies that update errors are propagated.
    #[tokio::test]
    async fn edit_game_fails_when_repo_fails() {
        let mut repo = MockGameRepository::new();

        let updated_game = Game {
            name: "Zelda: TOTK".to_string(),
            year: 2023,
            ..Default::default()
        };

        repo.expect_update()
            .with(
                mockall::predicate::eq("Zelda"),
                mockall::predicate::eq(2020),
                mockall::predicate::always(),
            )
            .once()
            .returning(|_, _, _| Err(anyhow!("Update failed")));

        let use_case = EditGameUseCase {
            game_repo: Arc::new(repo),
        };

        let result = use_case
            .execute("Zelda".to_string(), 2020, updated_game)
            .await;

        assert!(result.is_err());
    }
}
