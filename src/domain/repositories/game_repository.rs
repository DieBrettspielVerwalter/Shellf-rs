use crate::domain::entities::game::Game;
use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

/// Repository trait for managing the persistence and lifecycle of `Game` entities.
///
/// This trait defines the abstract interface for storing, retrieving, and
/// modifying core game metadata. It uses a combination of name and release
/// year as a unique identifier for games.
#[automock]
#[async_trait]
pub trait GameRepository: Send + Sync {
    /// Persists a new game or updates an existing one in the system.
    ///
    /// # Arguments
    ///
    /// * `game` - The `Game` entity to be saved.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the persistence layer fails to
    /// write the record.
    async fn save(&self, game: Game) -> Result<()>;

    /// Retrieves all games currently registered in the system.
    ///
    /// # Returns
    ///
    /// A `Vec<Game>` containing all stored game records.
    async fn all(&self) -> Vec<Game>;

    /// Retrieves a specific game identified by its name and release year.
    ///
    /// # Arguments
    ///
    /// * `name` - The title of the game.
    /// * `year` - The publication year of the game.
    ///
    /// # Returns
    ///
    /// `Some(Game)` if a matching record exists, otherwise `None`.
    #[allow(dead_code)]
    async fn get_by_name_and_year(&self, name: &str, year: i32) -> Option<Game>;

    /// Removes a game from the system.
    ///
    /// # Arguments
    ///
    /// * `name` - The title of the game to be deleted.
    /// * `year` - The publication year of the game to be deleted.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the deletion fails or the record
    /// cannot be found.
    async fn delete(&self, name: &str, year: i32) -> Result<()>;

    /// Updates an existing game record with new data.
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current title used to locate the record.
    /// * `old_year` - The current year used to locate the record.
    /// * `game` - The updated `Game` entity.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the update cannot be performed
    /// or the original record does not exist.
    async fn update(&self, old_name: &str, old_year: i32, game: Game) -> Result<()>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use async_trait::async_trait;

    /// Contract tests for any implementation of `GameRepository`.
    ///
    /// This trait defines a set of reusable tests that ensure consistent behavior
    /// across all repository implementations (e.g., SQL, In-Memory). Any concrete
    /// implementation must pass these tests to guarantee correct domain behavior.
    #[async_trait]
    pub trait GameRepositoryContractTests {
        /// Creates a fresh instance of the repository under test.
        async fn create_repo(&self) -> Box<dyn GameRepository>;

        /// Ensures that the required publisher metadata exists for referential integrity.
        async fn ensure_verlag_dependencies(&self, publisher: &str);

        /// Ensures that the required author metadata exists for referential integrity.
        async fn ensure_autor_dependencies(&self, autor: &str);

        /// Verifies that a game can be saved and retrieved by its unique identity.
        #[allow(non_snake_case)]
        async fn test__save_and_get_by_name(&self) {
            let repo = self.create_repo().await;
            self.ensure_verlag_dependencies("FromSoftware").await;
            self.ensure_verlag_dependencies("Kosmos").await;
            self.ensure_autor_dependencies("Klaus Teuber").await;
            self.ensure_autor_dependencies("Hidetaka Miyazaki").await;

            let game = Game::new(
                "Elden Ring".to_string(),
                2022,
                "1-4".to_string(),
                120,
                16,
                "Action RPG".to_string(),
                Some("FromSoftware".to_string()),
                vec!["Hidetaka Miyazaki".into()],
                4.8,
            );

            let catan = Game::new(
                "Catan".to_string(),
                1995,
                "3-4".to_string(),
                90,
                10,
                "Board Game".to_string(),
                Some("Kosmos".to_string()),
                vec!["Klaus Teuber".into()],
                4.2,
            );

            repo.save(game.clone()).await.unwrap();
            repo.save(catan.clone()).await.unwrap();

            let loaded = repo.get_by_name_and_year("Elden Ring", 2022).await;

            assert!(loaded.map(|l| l.eq(&game)).unwrap_or(false));
        }

        /// Verifies that requesting a non-existent game returns `None`.
        #[allow(non_snake_case)]
        async fn test__get_by_name__not_found(&self) {
            let repo = self.create_repo().await;

            let result = repo.get_by_name_and_year("unknown", 2000).await;

            assert!(result.is_none());
        }

        /// Verifies that all saved games are enumerated correctly by `all()`.
        #[allow(non_snake_case)]
        async fn test__all__returns_all_saved_games(&self) {
            let repo = self.create_repo().await;
            self.ensure_verlag_dependencies("FromSoftware").await;
            self.ensure_verlag_dependencies("Supergiant Games").await;
            self.ensure_autor_dependencies("Greg Kasavin").await;
            self.ensure_autor_dependencies("Hidetaka Miyazaki").await;

            let elden_ring = Game::new(
                "Elden Ring".to_string(),
                2022,
                "1-4".to_string(),
                120,
                16,
                "Action RPG".to_string(),
                Some("FromSoftware".to_string()),
                vec!["Hidetaka Miyazaki".to_string()],
                4.8,
            );

            let hades = Game::new(
                "Hades".to_string(),
                2020,
                "1".to_string(),
                45,
                12,
                "Roguelike".to_string(),
                Some("Supergiant Games".to_string()),
                vec!["Greg Kasavin".to_string()],
                4.6,
            );

            repo.save(elden_ring.clone()).await.unwrap();
            repo.save(hades.clone()).await.unwrap();

            let games = repo.all().await;

            assert_eq!(games.len(), 2);
            assert!(games.contains(&elden_ring));
            assert!(games.iter().any(|g| g.name == "Hades" && g.year == 2020));
        }
    }
}
