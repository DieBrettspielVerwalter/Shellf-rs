use crate::domain::{GameSession, GameSessionRepository};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlPool;

/// MariaDB-backed implementation of the `GameSessionRepository`.
///
/// This repository manages the lifecycle of game playthroughs (sessions).
/// It coordinates data across the `Partie` table (header data) and the
/// `Teilnahme` table (junction table for players and their results).
pub struct SqlGameSessionRepository {
    pool: MySqlPool,
}

impl SqlGameSessionRepository {
    /// Creates a new instance of `SqlGameSessionRepository`.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameSessionRepository for SqlGameSessionRepository {
    /// Transactionally persists a new game session and its participants.
    ///
    /// The process involves:
    /// 1. Calling the `Partie_Erstellen` stored procedure to initialize the session.
    /// 2. Iterating through the provided `participants` to populate the `Teilnahme` table.
    ///
    /// # Returns
    /// The unique integer ID assigned to the new session.
    async fn save(&self, session: GameSession, participants: Vec<String>) -> Result<i32> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "CALL Partie_Erstellen(?, ?)",
            session.game_copy_id,
            session.date
        )
        .execute(&mut *tx)
        .await?;

        let rec = sqlx::query!("SELECT LAST_INSERT_ID() as id")
            .fetch_one(&mut *tx)
            .await?;
        let new_id = rec.id as i32;

        for email in participants {
            sqlx::query!(
                "INSERT INTO Teilnahme (Partie_ID, Spieler_Email) VALUES (?, ?) 
                 ON DUPLICATE KEY UPDATE Spieler_Email = Spieler_Email",
                new_id,
                email
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(new_id)
    }

    /// Retrieves all recorded game sessions, aggregating participants into a single string.
    ///
    /// This method uses `GROUP_CONCAT` to efficiently fetch the list of player emails
    /// associated with each session in one query.
    async fn all(&self) -> Vec<GameSession> {
        let rows = sqlx::query!(
            r#"
            SELECT
                p.Partie_ID as "id: i32",
                p.Partiedatum as "date",
                p.Spielkopie_ID as "copy_id: i32",
                GROUP_CONCAT(t.Spieler_Email) as "participants: String"
            FROM Partie p
            LEFT JOIN Teilnahme t ON p.Partie_ID = t.Partie_ID
            GROUP BY p.Partie_ID
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(|r| GameSession {
                id: Some(r.id),
                game_copy_id: r.copy_id,
                date: r.date,
                _game_night_id: None,
                results: r.participants,
            })
            .collect()
    }

    /// Removes a game session and its associated participation records.
    ///
    /// Explicitly deletes entries from the `Teilnahme` table before removing the
    /// session header from the `Partie` table to maintain referential integrity.
    async fn delete(&self, id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!("DELETE FROM Teilnahme WHERE Partie_ID = ?", id)
            .execute(&mut *tx)
            .await?;
        sqlx::query!("DELETE FROM Partie WHERE Partie_ID = ?", id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Transactionally updates a session's metadata and synchronizes its participant list.
    ///
    /// This method performs a "delete-and-reinsert" strategy for participants to
    /// ensure the database state matches the provided list exactly.
    async fn update(&self, id: i32, session: GameSession, participants: Vec<String>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE Partie SET Partiedatum = ?, Spielkopie_ID = ? WHERE Partie_ID = ?",
            session.date,
            session.game_copy_id,
            id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!("DELETE FROM Teilnahme WHERE Partie_ID = ?", id)
            .execute(&mut *tx)
            .await?;
        for email in participants {
            sqlx::query!(
                "INSERT INTO Teilnahme (Partie_ID, Spieler_Email) VALUES (?, ?)",
                id,
                email
            )
            .execute(&mut *tx)
            .await?;
        }

        if let Some(rang) = session.results {
            sqlx::query!(
                "UPDATE Teilnahme SET Spielerrang = ? WHERE Partie_ID = ? AND Spieler_Email = USER_EMAIL()",
                rang.parse::<i32>().unwrap_or(0), id
            ).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Fetches all participant emails registered for a specific session ID.
    async fn get_participants(&self, partie_id: i32) -> Result<Vec<String>> {
        let rows = sqlx::query!(
            "SELECT Spieler_Email FROM Teilnahme WHERE Partie_ID = ?",
            partie_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.Spieler_Email).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::repositories::tests::GameSessionRepositoryContractTests,
        infrastructure::repositories::sql::test_utils::TestDbGuard,
    };
    use async_trait::async_trait;
    use sqlx::MySqlPool;

    struct SqlGameSessionRepoContract {
        pool: MySqlPool,
        admin_pool: MySqlPool,
    }

    #[async_trait]
    impl GameSessionRepositoryContractTests for SqlGameSessionRepoContract {
        async fn create_repo(&self) -> Box<dyn GameSessionRepository> {
            Box::new(SqlGameSessionRepository::new(self.pool.clone()))
        }
        async fn ensure_dependencies(&self, game_name: &str, year: i32, owner: &str) -> i32 {
            // 1. Verlag_Adresse anlegen (unterste Ebene)
            let addr_result = sqlx::query!(
                "INSERT IGNORE INTO Verlag_Adresse (PLZ, Ort, Strasse, Hausnummer) 
                VALUES (?, ?, ?, ?)",
                "12345",
                "Teststadt",
                "Testweg",
                "1"
            )
            .execute(&self.admin_pool)
            .await
            .expect("Failed to insert Verlag_Adresse");

            let addr_id = addr_result.last_insert_id();

            // 2. Verlag anlegen (referenziert Adresse)
            let verlag = "Testverlag";
            sqlx::query!(
                "INSERT IGNORE INTO Verlag (Verlagsname, Verlagsadresse) VALUES (?, ?)",
                verlag,
                addr_id as i32
            )
            .execute(&self.admin_pool)
            .await
            .expect("Failed to insert Verlag");

            // 3. Spiel anlegen (referenziert Verlag)
            sqlx::query!(
                "INSERT IGNORE INTO Spiel (Spieltitel, Erscheinungsjahr, Verlagsname) VALUES (?, ?, ?)",
                game_name, year as i16, verlag
            )
            .execute(&self.admin_pool).await.expect("Failed to insert Spiel");

            // 4. Besitzer anlegen (Person + Spieler-Rolle), damit FK in Spielkopie erfüllt ist
            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname) 
                VALUES (?, 'Testbesitzer', 'Max', 'Mustermann')",
                owner
            )
            .execute(&self.admin_pool)
            .await
            .expect("Failed to insert Person");

            sqlx::query!("INSERT IGNORE INTO Spieler (Email) VALUES (?)", owner)
                .execute(&self.admin_pool)
                .await
                .expect("Failed to insert Spieler");

            // 5. Spielkopie anlegen
            let copy_result = sqlx::query!(
                "INSERT INTO Spielkopie (Spieltitel, Erscheinungsjahr, Besitzer_Email) VALUES (?, ?, ?)",
                game_name,
                year as i16,
                owner
            )
            .execute(&self.admin_pool).await.expect("Failed to insert Spielkopie");

            // 6. Die generierte ID zurückgeben
            copy_result.last_insert_id() as i32
        }
        async fn ensure_person_dependencies(&self, name: &str, email: &str) {
            // 1. Basis-Person anlegen
            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname) 
                VALUES (?, ?, ?, ?)",
                email,
                name,
                "Test",
                "User"
            )
            .execute(&self.admin_pool)
            .await
            .unwrap();

            // 2. WICHTIG: Die Person muss auch in der Tabelle 'Spieler' existieren!
            sqlx::query!("INSERT IGNORE INTO Spieler (Email) VALUES (?)", email)
                .execute(&self.admin_pool)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__save_and_all() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameSessionRepoContract { pool, admin_pool };
        contract.test__save_and_all().await;
    }
}
