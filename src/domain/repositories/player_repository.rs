use crate::domain::Player;
use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

/// Repository trait for managing the persistence and lifecycle of `Player` entities.
///
/// This trait defines the abstract interface for storing and retrieving player
/// information, including sensitive data handling like passwords during creation.
#[async_trait]
#[automock]
pub trait PlayerRepository: Send + Sync {
    /// Persists a new player and their associated password.
    ///
    /// # Arguments
    ///
    /// * `player` - The `Player` domain entity to be saved.
    /// * `password` - The plain-text password to be hashed and stored by the implementation.
    ///
    /// # Returns
    ///
    /// A `Result` containing the successfully saved `Player` entity.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the persistence layer fails or
    /// if a player with the same email already exists.
    async fn save(&self, player: Player, password: String) -> Result<Player>;

    /// Retrieves all registered players from the system.
    ///
    /// # Returns
    ///
    /// A `Vec<Player>` containing all player records.
    async fn all(&self) -> Vec<Player>;

    /// Retrieves a specific player by their unique email address.
    ///
    /// # Arguments
    ///
    /// * `email` - The email address used to identify the player.
    ///
    /// # Returns
    ///
    /// `Some(Player)` if a matching record exists, otherwise `None`.
    #[allow(dead_code)]
    async fn get_by_email(&self, email: &str) -> Option<Player>;

    /// Updates an existing player's information.
    ///
    /// # Arguments
    ///
    /// * `email` - The current email address used to locate the player record.
    /// * `player` - The updated `Player` entity containing the new data.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the update fails or the player
    /// cannot be found.
    async fn update(&self, email: &str, player: Player) -> Result<()>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::domain::entities::player::Player;
    use async_trait::async_trait;

    /// Contract tests for any implementation of `PlayerRepository`.
    ///
    /// These tests ensure consistent behavior across all repository
    /// implementations (e.g., SQL, In-Memory) for managing `Player` entities.
    #[async_trait]
    #[allow(dead_code)]
    pub trait PlayerRepositoryContractTests {
        /// Creates a fresh instance of the repository under test.
        async fn create_repo(&self) -> Box<dyn PlayerRepository>;

        /// Verifies that a player can be saved and subsequently retrieved by email.
        #[allow(non_snake_case)]
        async fn test__save_and_get_by_email(&self) {
            let repo = self.create_repo().await;

            let player = Player {
                nickname: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                first_name: "Alice".to_string(),
                last_name: "Test".to_string(),
                details: None,
            };

            repo.save(player.clone(), "password123".into())
                .await
                .unwrap();

            let loaded = repo.get_by_email("alice@example.com").await;
            assert!(loaded.map(|p| p.email == player.email).unwrap_or(false));
        }

        /// Verifies that requesting a non-existent email returns `None`.
        #[allow(non_snake_case)]
        async fn test__get_by_email__not_found(&self) {
            let repo = self.create_repo().await;

            let result = repo.get_by_email("unknown@example.com").await;

            assert!(result.is_none());
        }

        /// Verifies that `all` returns all persisted players correctly.
        #[allow(non_snake_case)]
        async fn test__all__returns_all_saved_players(&self) {
            let repo = self.create_repo().await;

            let alice = Player {
                nickname: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                first_name: "Alice".to_string(),
                last_name: "Test".to_string(),
                details: None,
            };

            let bob = Player {
                nickname: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                first_name: "Bob".to_string(),
                last_name: "Test".to_string(),
                details: None,
            };

            repo.save(alice.clone(), "pw1".into()).await.unwrap();
            repo.save(bob.clone(), "pw2".into()).await.unwrap();

            let players = repo.all().await;
            assert!(players.len() >= 2);
            assert!(players.iter().any(|p| p.email == "alice@example.com"));
            assert!(players.iter().any(|p| p.email == "bob@example.com"));
        }
    }
}
