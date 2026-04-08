use crate::domain::entities::game_night::GameNight;
use anyhow::Result;
use async_trait::async_trait;

/// Repository abstraction for managing `GameNight` entities.
///
/// Provides an interface for saving and retrieving game nights,
/// enabling different storage backends (in-memory, SQL, etc.).
#[mockall::automock]
#[async_trait]
pub trait GameNightRepository: Send + Sync {
    /// Persists a new `GameNight`.
    ///
    /// # Arguments
    ///
    /// * `game_night` - The `GameNight` entity to save.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails.
    async fn save(&self, game_night: GameNight) -> Result<i32>;

    /// Retrieves all `GameNight` entities.
    ///
    /// # Returns
    ///
    /// A vector of all saved `GameNight` objects.
    async fn all(&self) -> Vec<GameNight>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::domain::entities::game_night::GameNight;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    /// Contract tests for `GameNightRepository` implementations.
    ///
    /// Ensures consistent behavior for saving and retrieving game nights across
    /// different repository backends.
    #[async_trait]
    pub trait GameNightRepositoryContractTests {
        /// Creates a fresh instance of the repository under test.
        async fn create_repo(&self) -> Box<dyn GameNightRepository>;

        /// Ensures that required host and participant records exist in the database.
        async fn ensure_host_and_participants(&self, host_email: &str, participants: &[String]);

        /// Ensures that a game copy exists and returns its ID.
        async fn ensure_game_copy_exists(
            &self,
            game_name: &str,
            year: i32,
            test_owner: &str,
        ) -> i32;

        /// Ensures that a game session exists and returns its ID.
        async fn ensure_game_session_exists(
            &self,
            game_copy_id: i32,
            date: NaiveDate,
            game_night: i32,
            participants: Vec<&str>,
        ) -> i32;

        /// Verifies that `save` persists game nights and `all` returns them.
        #[allow(non_snake_case)]
        async fn test__save_and_all(&self) {
            let repo = self.create_repo().await;

            let host_email = "maria@spiele.de";
            let participants = vec!["maria@spiele.de".to_string(), "john@spiele.de".to_string()];

            // Ensure all prerequisites exist
            self.ensure_host_and_participants(host_email, &participants)
                .await;
            let elden = self
                .ensure_game_copy_exists("Elden Ring", 2022, "maria@spiele.de")
                .await;
            let catan = self
                .ensure_game_copy_exists("Catan", 1995, "john@spiele.de")
                .await;

            // Create GameNight instances
            let mut night1 = GameNight {
                id: -1,
                date: NaiveDate::from_ymd_opt(2026, 3, 3).unwrap(),
                notes: Some("First night".to_string()),
                participants: participants.clone(),
                suggested_copies: vec![elden],
            };

            let mut night2 = GameNight {
                id: -1,
                date: NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
                notes: None,
                participants: participants.clone(),
                suggested_copies: vec![catan],
            };

            // Save nights
            night1.id = repo.save(night1.clone()).await.expect("Save night1 failed");
            night2.id = repo.save(night2.clone()).await.expect("Save night2 failed");

            self.ensure_game_session_exists(
                elden,
                night1.date,
                night1.id,
                vec!["maria@spiele.de", "john@spiele.de"],
            )
            .await;
            self.ensure_game_session_exists(
                elden,
                night2.date,
                night2.id,
                vec!["maria@spiele.de", "john@spiele.de"],
            )
            .await;

            let nights = repo.all().await;
            println!("NIGHTS: {nights:?}");

            // Assertions
            assert_eq!(nights.len(), 2, "Expected exactly 2 game nights in the DB");

            // Compare content without relying on ID
            assert!(nights
                .iter()
                .any(|n| n.date == night1.date && n.notes == night1.notes));
            assert!(nights
                .iter()
                .any(|n| n.date == night2.date && n.notes == night2.notes));

            // Verify participants are loaded
            for night in nights {
                assert!(night.participants.contains(&"john@spiele.de".to_string()));
            }
        }
    }
}
