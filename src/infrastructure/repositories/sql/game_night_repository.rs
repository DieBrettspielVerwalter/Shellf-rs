use crate::domain::{GameNight, GameNightRepository};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlPool;

/// MariaDB-backed implementation of the `GameNightRepository`.
///
/// This repository manages the lifecycle of game night events, including
/// the complex mapping of participants through relational joins and
/// string aggregation.
pub struct SqlGameNightRepository {
    pool: MySqlPool,
}

impl SqlGameNightRepository {
    /// Creates a new instance of `SqlGameNightRepository`.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameNightRepository for SqlGameNightRepository {
    /// Persists a new game night to the database.
    ///
    /// The first participant in the `participants` list is automatically
    /// designated as the host (Gastgeber) to satisfy database constraints.
    ///
    /// # Errors
    /// Returns an error if the participant list is empty or if the database
    /// transaction fails.
    async fn save(&self, night: GameNight) -> Result<i32> {
        let mut tx = self.pool.begin().await?;

        let host = night.participants.first().ok_or_else(|| {
            anyhow::anyhow!("A game night requires at least one participant (host)")
        })?;

        let id = sqlx::query!(
            "INSERT INTO Spieleabend (Spieleabenddatum, Spieleabendnotizen, Gastgeber_Email) VALUES (?, ?, ?)",
            night.date,
            night.notes,
            host
        )
            .execute(&mut *tx)
            .await?
            .last_insert_id() as i32;

        tx.commit().await?;
        Ok(id)
    }

    /// Retrieves all game nights, including an aggregated list of participants.
    ///
    /// This method uses a subquery with `GROUP_CONCAT` to efficiently fetch
    /// all participants for each game night in a single relational query.
    async fn all(&self) -> Vec<GameNight> {
        let rows = sqlx::query!(
            r#"
            SELECT
                sa.Spieleabend_ID AS "Spieleabend_ID: i32",
                sa.Spieleabenddatum,
                sa.Spieleabendnotizen,
                (SELECT GROUP_CONCAT(DISTINCT t.Spieler_Email)
                 FROM Partie p
                 JOIN Teilnahme t ON p.Partie_ID = t.Partie_ID
                 WHERE p.Spieleabend_ID = sa.Spieleabend_ID) as teilnehmer_liste
            FROM Spieleabend sa
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(|r| {
                let participants: Vec<String> = r
                    .teilnehmer_liste
                    .map(|list| list.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                GameNight {
                    id: r.Spieleabend_ID,
                    date: r.Spieleabenddatum,
                    notes: r.Spieleabendnotizen,
                    participants,
                    suggested_copies: Vec::new(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::repositories::tests::GameNightRepositoryContractTests,
        infrastructure::repositories::sql::test_utils::TestDbGuard,
    };
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use sqlx::MySqlPool;

    /// Implementation of contract tests for the SQL-based GameNight repository.
    struct SqlGameNightRepoContract {
        pool: MySqlPool,
    }

    #[async_trait]
    impl GameNightRepositoryContractTests for SqlGameNightRepoContract {
        /// Returns a boxed repository instance for generic contract testing.
        async fn create_repo(&self) -> Box<dyn GameNightRepository> {
            Box::new(SqlGameNightRepository::new(self.pool.clone()))
        }

        /// Sets up player and host records required by the database schema.
        async fn ensure_host_and_participants(&self, host_email: &str, participants: &[String]) {
            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname)
                VALUES (?, ?, 'Host', 'User')",
                host_email,
                host_email
            )
            .execute(&self.pool)
            .await
            .unwrap();

            sqlx::query!("INSERT IGNORE INTO Spieler (Email) VALUES (?)", host_email)
                .execute(&self.pool)
                .await
                .unwrap();

            for email in participants {
                sqlx::query!(
                    "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname)
                    VALUES (?, ?, 'Participant', 'User')",
                    email, email
                ).execute(&self.pool).await.unwrap();

                sqlx::query!("INSERT IGNORE INTO Spieler (Email) VALUES (?)", email)
                    .execute(&self.pool)
                    .await
                    .unwrap();
            }
        }

        /// Performs a multi-stage setup to create a valid `GameCopy` with all its dependencies.
        ///
        /// This method follows the relational hierarchy:
        /// Address -> Publisher -> Game -> Person -> Player -> GameCopy.
        async fn ensure_game_copy_exists(
            &self,
            game_name: &str,
            year: i32,
            test_owner: &str,
        ) -> i32 {
            let addr_result = sqlx::query!(
                "INSERT IGNORE INTO Verlag_Adresse (PLZ, Ort, Strasse, Hausnummer)
                VALUES (?, ?, ?, ?)",
                "12345",
                "Teststadt",
                "Testweg",
                "1"
            )
            .execute(&self.pool)
            .await
            .expect("Failed to insert Verlag_Adresse");

            let addr_id = addr_result.last_insert_id();

            let verlag = "Testverlag";
            sqlx::query!(
                "INSERT IGNORE INTO Verlag (Verlagsname, Verlagsadresse) VALUES (?, ?)",
                verlag,
                addr_id as i32
            )
            .execute(&self.pool)
            .await
            .expect("Failed to insert Verlag");

            sqlx::query!(
                "INSERT IGNORE INTO Spiel (Spieltitel, Erscheinungsjahr, Verlagsname) VALUES (?, ?, ?)",
                game_name, year as i16, verlag
            )
                .execute(&self.pool).await.expect("Failed to insert Spiel");

            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname)
                VALUES (?, 'Testbesitzer', 'Max', 'Mustermann')",
                test_owner
            )
            .execute(&self.pool)
            .await
            .expect("Failed to insert Person");

            sqlx::query!("INSERT IGNORE INTO Spieler (Email) VALUES (?)", test_owner)
                .execute(&self.pool)
                .await
                .expect("Failed to insert Spieler");

            let copy_result = sqlx::query!(
                "INSERT INTO Spielkopie (Spieltitel, Erscheinungsjahr, Besitzer_Email) VALUES (?, ?, ?)",
                game_name,
                year as i16,
                test_owner
            )
                .execute(&self.pool).await.expect("Failed to insert Spielkopie");

            copy_result.last_insert_id() as i32
        }

        /// Ensures that a `GameSession` (Partie) exists with the given participants.
        /// Returns the new session ID.
        async fn ensure_game_session_exists(
            &self,
            game_copy_id: i32,
            date: NaiveDate,
            game_night: i32,
            participants: Vec<&str>,
        ) -> i32 {
            // First, insert the session via the stored procedure
            let session_id = sqlx::query!(
                "INSERT INTO Partie (Partiedatum, Spielkopie_ID, Spieleabend_ID) VALUES (?, ?, ?)",
                date,
                game_copy_id,
                game_night
            )
            .execute(&self.pool)
            .await
            .expect("Failed to insert Partie")
            .last_insert_id() as i32;

            // Ensure each participant exists in the Person and Spieler tables
            for email in participants {
                sqlx::query!(
                    "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname)
                    VALUES (?, 'Testperson', 'Max', 'Mustermann')",
                    email
                )
                .execute(&self.pool)
                .await
                .expect("Failed to insert Person");

                sqlx::query!("INSERT IGNORE INTO Spieler (Email) VALUES (?)", email)
                    .execute(&self.pool)
                    .await
                    .expect("Failed to insert Spieler");

                // Add participant to the session (Teilnahme)
                sqlx::query!(
                    "INSERT INTO Teilnahme (Partie_ID, Spieler_Email) VALUES (?, ?)
                    ON DUPLICATE KEY UPDATE Spieler_Email = Spieler_Email",
                    session_id,
                    email
                )
                .execute(&self.pool)
                .await
                .expect("Failed to insert Teilnahme");
            }

            session_id
        }
    }

    /// Integration test that validates the SQL implementation against a real (isolated) database.
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__save_and_all() {
        let _guard = TestDbGuard::new().await;
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameNightRepoContract { pool: admin_pool };
        contract.test__save_and_all().await;
    }
}
