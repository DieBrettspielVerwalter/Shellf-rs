use crate::domain::entities::game_copy::GameCopy;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

/// A repository trait defining persistence operations for `GameCopy` entities.
///
/// This trait abstracts all data access logic related to game copies,
/// including CRUD operations and domain-specific behaviors such as lending
/// and returning copies.
///
/// Implementations of this trait may interact with different storage backends
/// (e.g., in-memory, SQL database) but must adhere to the same contract.
#[mockall::automock]
#[async_trait]
pub trait GameCopyRepository: Send + Sync {
    /// Persists a new `GameCopy` and returns its generated ID.
    ///
    /// # Arguments
    ///
    /// * `copy` - The `GameCopy` entity to be saved.
    ///
    /// # Returns
    ///
    /// The unique identifier (`i32`) assigned to the saved copy.
    async fn save(&self, copy: GameCopy) -> Result<i32>;

    /// Retrieves a `GameCopy` by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the game copy.
    ///
    /// # Returns
    ///
    /// Returns `Some(GameCopy)` if found, otherwise `None`.
    #[allow(dead_code)]
    async fn get_by_id(&self, id: i32) -> Option<GameCopy>; // Wichtig!

    /// Retrieves all stored game copies.
    ///
    /// # Returns
    ///
    /// A collection of all `GameCopy` entities.
    async fn all(&self) -> Vec<GameCopy>;

    /// Deletes a game copy by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the game copy to delete.
    ///
    /// # Errors
    ///
    /// Returns an error if the deletion fails
    /// (e.g., copy not found or database issue).
    async fn delete(&self, id: i32) -> Result<()>;

    /// Updates an existing game copy.
    ///
    /// # Arguments
    ///
    /// * `old_id` - The current identifier of the copy to update.
    /// * `copy` - The updated `GameCopy` entity.
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    async fn update(&self, old_id: i32, copy: GameCopy) -> Result<()>;

    /// Marks a game copy as lent to a borrower.
    ///
    /// # Arguments
    ///
    /// * `copy_id` - The ID of the game copy.
    /// * `borrower_email` - The email of the borrower.
    /// * `start_date` - The date when the lending begins.
    /// * `due_date` - Optional return due date.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// (e.g., copy not found, already lent, or persistence issue).
    async fn lend_copy(
        &self,
        copy_id: i32,
        borrower_email: &str,
        start_date: NaiveDate,
        due_date: Option<NaiveDate>,
    ) -> Result<()>;

    /// Marks a game copy as returned.
    ///
    /// # Arguments
    ///
    /// * `copy_id` - The ID of the game copy.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    /// (e.g., copy not found or persistence issue).
    async fn return_copy(&self, copy_id: i32) -> Result<()>;

    /// Retrieves all currently lent game copies.
    ///
    /// # Returns
    ///
    /// A collection of all `GameCopy` entities that are currently lent out.
    ///
    /// # Errors
    ///
    /// Returns an error if the retrieval fails.
    async fn all_lent(&self) -> Result<Vec<GameCopy>>; // Neu hinzugefügt
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::domain::entities::game_copy::GameCopy;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    /// Contract tests for any implementation of `GameCopyRepository`.
    ///
    /// These tests ensure that the repository correctly handles persistence, retrieval,
    /// deletion, and lending/returning of `GameCopy` entities. Any concrete implementation
    /// (e.g., in-memory, SQL-based) must pass these tests to guarantee consistent behavior.
    #[async_trait]
    #[allow(dead_code)]
    pub trait GameCopyRepositoryContractTests {
        /// Creates a fresh instance of the repository under test.
        ///
        /// Must return a boxed instance of `GameCopyRepository` for use in the tests.
        async fn create_repo(&self) -> Box<dyn GameCopyRepository>;

        /// Provides access to the underlying database pool.
        ///
        /// This is primarily used for debugging or auxiliary queries during tests.
        async fn pool(&self) -> sqlx::MySqlPool;

        /// Ensures that required game dependencies exist (e.g., for foreign key constraints).
        async fn ensure_dependencies(&self, game_name: &str, year: i32);

        /// Ensures that required player dependencies exist.
        async fn ensure_person_dependencies(&self, name: &str, email: &str);

        /// Verifies that a saved copy can be retrieved by its ID.
        ///
        /// Steps:
        /// 1. Save a new `GameCopy`.
        /// 2. Retrieve it using `get_by_id`.
        /// 3. Assert the retrieved copy equals the saved one.
        #[allow(non_snake_case)]
        async fn test__save_and_get_by_id(&self) {
            let repo = self.create_repo().await;

            // Ensure the game exists (FK protection)
            self.ensure_dependencies("Elden Ring", 2022).await;

            // Ensure the owner exists as a player (FK protection)
            self.ensure_person_dependencies("Maria Test", "maria@spiele.de")
                .await;

            let mut copy =
                GameCopy::new(-1, "Elden Ring".to_string(), 2022, "maria@spiele.de".into());

            // Save and retrieve the assigned ID
            let copy_id = repo.save(copy.clone()).await.unwrap();
            copy.id = copy_id;
            println!("COPY: {copy:?}");

            let loaded = repo.get_by_id(copy_id).await;
            println!("LOADED: {loaded:?}");
            assert!(loaded.map(|c| c.eq(&copy)).unwrap_or(false));
        }

        /// Verifies that requesting a non-existent copy returns `None`.
        #[allow(non_snake_case)]
        async fn test__get_by_id__not_found(&self) {
            let repo = self.create_repo().await;

            let result = repo.get_by_id(0).await;
            assert!(result.is_none());
        }

        /// Verifies that all saved copies are returned by `all()`.
        #[allow(non_snake_case)]
        async fn test__all__returns_all_saved_copies(&self) {
            let repo = self.create_repo().await;

            // Ensure the games exist
            self.ensure_dependencies("Elden Ring", 2022).await;
            self.ensure_dependencies("Catan", 1995).await;

            // Ensure the owners exist
            // self.ensure_person_dependencies("Maria Test", "maria@spiele.de").await;
            self.ensure_person_dependencies("Tom Test", "tom@spiele.de")
                .await;

            let owner1 = "maria@spiele.de".into();
            let owner2 = "tom@spiele.de".into();

            let mut copy1 = GameCopy::new(-1, "Elden Ring".to_string(), 2022, owner1);
            let mut copy2 = GameCopy::new(-1, "Catan".to_string(), 1995, owner2);

            // 1. Save and get DB-assigned IDs
            let id1 = repo.save(copy1.clone()).await.unwrap();
            let id2 = repo.save(copy2.clone()).await.unwrap();

            // 2. Update test objects with DB IDs
            copy1.id = id1;
            copy2.id = id2;

            let copies = repo.all().await;

            // 3. Verify at least these two copies exist
            assert!(copies.len() >= 2);
            assert!(copies
                .iter()
                .any(|c| c.id == id1 && c.game_name == "Elden Ring"));
            assert!(copies.iter().any(|c| c.id == id2 && c.game_name == "Catan"));
        }

        /// Verifies that deleting a copy removes it from the repository.
        #[allow(non_snake_case)]
        async fn test__delete_removes_copy(&self) {
            let repo = self.create_repo().await;

            // Ensure the game and owner exist
            self.ensure_dependencies("Hades", 2020).await;
            // self.ensure_person_dependencies("Maria Test", "maria@spiele.de").await;

            let copy = GameCopy::new(-1, "Hades".to_string(), 2020, "maria@spiele.de".into());
            let copy_id = repo.save(copy.clone()).await.unwrap();

            // Delete and verify removal
            repo.delete(copy_id).await.unwrap();
            let loaded = repo.get_by_id(copy_id).await;
            assert!(loaded.is_none());
        }

        /// Verifies that lending and returning a copy updates its state correctly.
        ///
        /// Steps:
        /// 1. Save a new `GameCopy`.
        /// 2. Lend it to a borrower on a given date.
        /// 3. Return the copy.
        /// 4. Optional: assert that `is_lent` and borrower info were updated correctly
        ///    (depending on concrete implementation).
        #[allow(non_snake_case)]
        async fn test__lend_and_return_copy(&self) {
            let repo = self.create_repo().await;

            // Ensure the game exists
            self.ensure_dependencies("Elden Ring", 2022).await;

            // Ensure owner and borrower exist
            // self.ensure_person_dependencies("Maria Test", "maria@spiele.de").await;
            self.ensure_person_dependencies("Tom Test", "tom@spiele.de")
                .await;

            let owner = "maria@spiele.de".into();
            let borrower: String = "tom@spiele.de".into();
            let copy = GameCopy::new(-1, "Elden Ring".to_string(), 2022, owner);
            let copy_id = repo.save(copy.clone()).await.unwrap();

            let start_date = NaiveDate::from_ymd_opt(2026, 3, 3).unwrap();

            // Lend the copy

            println!(
                "{:?}",
                sqlx::query!("SELECT USER_EMAIL() AS user_email;")
                    .fetch_one(&self.pool().await)
                    .await
                    .unwrap()
            );
            repo.lend_copy(copy_id, &borrower.to_string(), start_date, None)
                .await
                .unwrap();

            // Return the copy
            repo.return_copy(copy_id).await.unwrap();
        }
    }
}
