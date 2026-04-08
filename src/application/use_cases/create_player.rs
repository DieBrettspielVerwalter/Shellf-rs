use crate::domain::entities::player::Player;
use anyhow::Result;
use mantra_rust_macros::req;

/// A use case responsible for creating and persisting a new player.
///
/// This struct handles the orchestration between the domain entity validation
/// and the infrastructure layer for storage.
pub struct CreatePlayerUseCase {
    /// The repository used to persist and manage player data.
    pub player_repo: std::sync::Arc<dyn crate::domain::PlayerRepository>,
}

impl CreatePlayerUseCase {
    /// Executes the business logic to create a new player.
    ///
    /// The process involves validating the input through the `Player` domain entity
    /// and then persisting both the entity and the password via the repository.
    ///
    /// # Arguments
    ///
    /// * `nickname` - The unique display name for the player.
    /// * `first_name` - The player's legal first name.
    /// * `last_name` - The player's legal last name.
    /// * `email` - The player's email address, used for identification.
    /// * `details` - Optional additional information or profile bio.
    /// * `password` - The plain-text password to be processed and saved.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - The domain validation for the `Player` entity fails (e.g., invalid email format).
    /// - The persistence layer fails to save the record.
    #[req("UC.3")]
    pub async fn execute(
        &self,
        nickname: String,
        first_name: String,
        last_name: String,
        email: String,
        details: Option<String>,
        password: String,
    ) -> Result<Player> {
        let new_player = Player::new(email, nickname, first_name, last_name, details)?;

        self.player_repo.save(new_player, password).await
    }
}

/// A use case responsible for updating an existing player's information.
pub struct EditPlayerUseCase {
    /// The repository used to update player data.
    pub player_repo: std::sync::Arc<dyn crate::domain::PlayerRepository>,
}

impl EditPlayerUseCase {
    /// Updates an existing player identified by their current email.
    ///
    /// # Arguments
    ///
    /// * `old_email` - The current email address used to identify the record in the database.
    /// * `player` - The updated `Player` domain entity containing the new data.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the repository fails to perform the update
    /// (e.g., player not found or database connection issues).
    pub async fn execute(&self, old_email: &str, player: Player) -> Result<()> {
        self.player_repo.update(old_email, player).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::mockups::*;
    use anyhow::anyhow;
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn create_player_use_case_executes_and_saves_player() {
        let mut player_repo = MockPlayerRepository::new();

        player_repo
            .expect_save()
            .withf(|player, _pw| {
                player.nickname == "TestPlayer" && player.email == "test@example.com"
            })
            .once()
            .returning(|player, _pw| Box::pin(async move { Ok(player) }));

        let use_case = CreatePlayerUseCase {
            player_repo: Arc::new(player_repo),
        };

        let result = use_case
            .execute(
                "TestPlayer".to_string(),
                "FirstName".to_string(),
                "LastName".to_string(),
                "test@example.com".to_string(),
                None,
                "secret_password".to_string(),
            )
            .await;

        assert!(result.is_ok());

        let player = result.unwrap();
        assert_eq!(player.nickname, "TestPlayer");
        assert_eq!(player.email, "test@example.com");
    }

    #[tokio::test]
    async fn create_player_use_case_fails_when_save_fails() {
        let mut player_repo = MockPlayerRepository::new();

        player_repo
            .expect_save()
            .once()
            .returning(|_, _| Box::pin(async move { Err(anyhow!("Database write failed")) }));

        let use_case = CreatePlayerUseCase {
            player_repo: Arc::new(player_repo),
        };

        let result = use_case
            .execute(
                "TestPlayer".to_string(),
                "FirstName".to_string(),
                "LastName".to_string(),
                "test@example.com".to_string(),
                None,
                "password".to_string(),
            )
            .await;

        assert!(result.is_err());
    }

    /// Verifies successful update of a player.
    ///
    /// Ensures:
    /// - `PlayerRepository::update` is called once with correct parameters.
    /// - The use case returns `Ok(())`.
    #[tokio::test]
    async fn edit_player_use_case_executes_successfully() {
        let mut player_repo = MockPlayerRepository::new();

        let updated_player = Player::new(
            "new@example.com".to_string(),
            "UpdatedPlayer".to_string(),
            "First".to_string(),
            "Last".to_string(),
            None,
        )
        .unwrap();

        player_repo
            .expect_update()
            .withf(|old_email, player| {
                old_email == "old@example.com"
                    && player.email == "new@example.com"
                    && player.nickname == "UpdatedPlayer"
            })
            .once()
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        let use_case = EditPlayerUseCase {
            player_repo: Arc::new(player_repo),
        };

        let result = use_case.execute("old@example.com", updated_player).await;

        assert!(result.is_ok());
    }

    /// Verifies that update errors are propagated.
    #[tokio::test]
    async fn edit_player_use_case_fails_when_repo_fails() {
        let mut player_repo = MockPlayerRepository::new();

        let updated_player = Player::new(
            "new@example.com".to_string(),
            "UpdatedPlayer".to_string(),
            "First".to_string(),
            "Last".to_string(),
            None,
        )
        .unwrap();

        player_repo
            .expect_update()
            .with(
                mockall::predicate::eq("old@example.com"),
                mockall::predicate::always(),
            )
            .once()
            .returning(|_, _| Box::pin(async move { Err(anyhow!("Update failed")) }));

        let use_case = EditPlayerUseCase {
            player_repo: Arc::new(player_repo),
        };

        let result = use_case.execute("old@example.com", updated_player).await;

        assert!(result.is_err());
    }
}
