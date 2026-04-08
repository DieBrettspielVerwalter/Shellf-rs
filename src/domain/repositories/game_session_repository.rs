use crate::domain::entities::game_session::GameSession;
use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

/// Repository trait for managing the persistence of `GameSession` records.
///
/// This trait handles the storage of game playthroughs, including the
/// mapping of participants and result metadata.
#[async_trait]
#[automock]
pub trait GameSessionRepository: Send + Sync {
    /// Persists a new game session and its associated participants.
    ///
    /// # Arguments
    ///
    /// * `session` - The `GameSession` entity containing the core playthrough data.
    /// * `participants` - A collection of identifiers for players who took part in the session.
    ///
    /// # Returns
    ///
    /// The unique integer identifier (`i32`) assigned to the saved session.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the persistence layer fails to save the record.
    async fn save(&self, session: GameSession, participants: Vec<String>) -> Result<i32>;

    /// Retrieves all recorded game sessions from the system.
    ///
    /// # Returns
    ///
    /// A `Vec<GameSession>` containing all historical session records.
    async fn all(&self) -> Vec<GameSession>;

    /// Removes a game session and its related data from the system.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the session to be deleted.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the deletion fails or the record is not found.
    async fn delete(&self, id: i32) -> Result<()>;

    /// Updates an existing game session and its participant list.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the existing record to update.
    /// * `session` - The updated `GameSession` entity.
    /// * `participants` - The updated list of participants for this session.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the update cannot be performed.
    async fn update(&self, id: i32, session: GameSession, participants: Vec<String>) -> Result<()>;

    /// Retrieves the list of participants associated with a specific session.
    ///
    /// # Arguments
    ///
    /// * `partie_id` - The identifier of the game session.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<String>` of participant identifiers.
    async fn get_participants(&self, partie_id: i32) -> Result<Vec<String>>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::domain::entities::game_session::GameSession;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    /// Contract tests for any implementation of `GameSessionRepository`.
    ///
    /// Ensures consistent behavior across repository implementations for `GameSession` entities.
    /// Focuses on persistence, retrieval, participant handling, and transactional integrity.
    #[async_trait]
    #[allow(dead_code)]
    pub trait GameSessionRepositoryContractTests {
        /// Creates a fresh instance of the repository under test.
        async fn create_repo(&self) -> Box<dyn GameSessionRepository>;

        /// Ensures that all dependencies exist in the database to satisfy foreign key constraints.
        ///
        /// # Arguments
        ///
        /// * `game_name` - The name of the game to create or ensure exists.
        /// * `year` - The release year of the game.
        /// * `owner` - The email of the player who owns a copy of the game.
        ///
        /// # Returns
        ///
        /// The ID of the created game copy.
        async fn ensure_dependencies(&self, game_name: &str, year: i32, owner: &str) -> i32;

        /// Ensures that a person exists in the database and is also registered as a player.
        ///
        /// # Arguments
        ///
        /// * `name` - Full name of the person.
        /// * `email` - Email of the person.
        async fn ensure_person_dependencies(&self, name: &str, email: &str);

        /// Verifies that saving a session persists it correctly and allows retrieval via `all()`.
        ///
        /// Tests that participants are correctly aggregated into the `results` field.
        #[allow(non_snake_case)]
        async fn test__save_and_all(&self) {
            let repo = self.create_repo().await;

            self.ensure_person_dependencies("Alice", "alice@test.de")
                .await;
            self.ensure_person_dependencies("Bob", "bob@test.de").await;
            let owner = "maria@spiele.de";
            let copy_id = self.ensure_dependencies("Test Game", 2022, owner).await;

            let session1 = GameSession {
                id: None,
                game_copy_id: copy_id,
                date: NaiveDate::from_ymd_opt(2026, 3, 3).unwrap(),
                _game_night_id: None,
                results: None,
            };

            let session2 = GameSession {
                id: None,
                game_copy_id: copy_id,
                date: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
                _game_night_id: None,
                results: None,
            };

            let participants1 = vec!["alice@test.de".to_string(), "bob@test.de".to_string()];
            let participants2 = vec!["alice@test.de".to_string()];

            // Save the sessions
            repo.save(session1.clone(), participants1.clone())
                .await
                .unwrap();
            repo.save(session2.clone(), participants2.clone())
                .await
                .unwrap();

            // Retrieve all sessions
            let sessions = repo.all().await;

            // Verify sessions exist and participants are aggregated
            assert!(sessions.iter().any(|s| s.date == session1.date));
            assert!(sessions.iter().any(|s| s.date == session2.date));

            let loaded1 = sessions.iter().find(|s| s.date == session1.date).unwrap();
            let loaded2 = sessions.iter().find(|s| s.date == session2.date).unwrap();

            // Participants are stored in `results` field via GROUP_CONCAT
            for p in participants1 {
                assert!(loaded1.results.clone().unwrap_or_default().contains(&p));
            }
            for p in participants2 {
                assert!(loaded2.results.clone().unwrap_or_default().contains(&p));
            }
        }
    }
}
