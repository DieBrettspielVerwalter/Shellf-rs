use crate::domain::{Game, GameRepository};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlPool;

/// MariaDB-backed implementation of the `GameRepository`.
///
/// This repository manages the complex persistence of board games, coordinating
/// data across multiple tables including `Spiel`, `Person`, `Autor`, `Verlag`,
/// and `Designs`. It utilizes a mix of raw SQL and stored procedures.
pub struct SqlGameRepository {
    pool: MySqlPool,
}

impl SqlGameRepository {
    /// Creates a new `SqlGameRepository` instance.
    ///
    /// # Arguments
    ///
    /// * `pool` - A `MySqlPool` connection pool for database interactions.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameRepository for SqlGameRepository {
    /// Persists a `Game` entity by ensuring all relational dependencies exist.
    ///
    /// The process involves:
    /// 1. Creating a dummy address and publisher if it doesn't exist.
    /// 2. Registering authors as `Person` and `Autor` entities.
    /// 3. Calling the `SpielEinfuegenMitDetails` stored procedure.
    /// 4. Updating BGG ratings and author associations (`Designs`).
    ///
    /// # Arguments
    ///
    /// * `game` - The domain `Game` entity to save.
    ///
    /// # Errors
    ///
    /// Returns an error if any database operation fails.
    async fn save(&self, game: Game) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        if let Some(pub_name) = &game.publisher {
            let addr_res =
                sqlx::query!(
                "INSERT INTO Verlag_Adresse (PLZ, Ort, Strasse, Hausnummer) VALUES (?, ?, ?, ?)",
                "00000", "Unbekannt", "Dummy-Str.", "0"
            )
                .execute(&mut *tx)
                .await?;

            let addr_id = addr_res.last_insert_id() as i32;

            sqlx::query!(
                "INSERT IGNORE INTO Verlag (Verlagsname, Verlagsadresse) VALUES (?, ?)",
                pub_name,
                addr_id
            )
            .execute(&mut *tx)
            .await?;
        }

        for author_name in &game.authors {
            let email = format!("{}@autor.de", author_name.replace(' ', ".").to_lowercase());
            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname) VALUES (?, ?, ?, ?)",
                email, author_name, "Autor", "Unbekannt"
            ).execute(&mut *tx).await?;

            sqlx::query!("INSERT IGNORE INTO Autor (Autor_Email) VALUES (?)", email)
                .execute(&mut *tx)
                .await?;
        }

        let parts: Vec<&str> = game.players.split('-').collect();
        let min_p: i32 = parts[0].parse().unwrap_or(1);
        let max_p = parts
            .get(1)
            .map(|s| s.parse().unwrap_or(min_p))
            .unwrap_or(min_p);

        sqlx::query!(
            "CALL SpielEinfuegenMitDetails(?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            game.name,
            game.year as i16,
            game.category,
            game.age,
            game.publisher.as_deref().unwrap_or("Unbekannt"),
            game.duration,
            game.duration,
            min_p,
            max_p,
            game.authors
                .first()
                .map(|s| format!("{}@autor.de", s.replace(' ', ".").to_lowercase()))
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE Spiel SET BGG_Rating = ? WHERE Spieltitel = ? AND Erscheinungsjahr = ?",
            game.rating,
            game.name,
            game.year as i16
        )
        .execute(&mut *tx)
        .await?;

        for author_name in &game.authors {
            let email = format!("{}@autor.de", author_name.replace(' ', ".").to_lowercase());
            sqlx::query!(
                "INSERT IGNORE INTO Designs (Spieltitel, Erscheinungsjahr, Autor_Email) VALUES (?, ?, ?)",
                game.name, game.year as i16, email
            ).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Retrieves all games, aggregating authors via `GROUP_CONCAT`.
    ///
    /// This method also fetches the player range from the `Spiel_Spieleranzahl` table
    /// and maps the result to the domain `Game` entity.
    ///
    /// # Returns
    ///
    /// A vector of all `Game` entities found in the database.
    async fn all(&self) -> Vec<Game> {
        let rows = sqlx::query!(
            r#"
            SELECT
                s.Spieltitel,
                s.Erscheinungsjahr AS "Erscheinungsjahr: i32",
                s.Kategorie,
                s.Altersempfehlung AS "Altersempfehlung: i32",
                s.Verlagsname,
                s.Spieldauer_Durchschnitt AS "Spieldauer_Durchschnitt: i32",
                s.BGG_Rating AS "BGG_Rating: f32",
                GROUP_CONCAT(p.Personname) as autoren_liste,
                (SELECT CONCAT(min.Spieleranzahl_min, '-', min.Spieleranzahl_max)
                 FROM Spiel_Spieleranzahl min
                 WHERE min.Spieltitel = s.Spieltitel AND min.Erscheinungsjahr = s.Erscheinungsjahr
                 LIMIT 1) as spieler_range
            FROM Spiel s
            LEFT JOIN Designs d ON s.Spieltitel = d.Spieltitel AND s.Erscheinungsjahr = d.Erscheinungsjahr
            LEFT JOIN Person p ON d.Autor_Email = p.Email
            GROUP BY s.Spieltitel, s.Erscheinungsjahr
            "#
        )
            .fetch_all(&self.pool)
            .await
            .expect("Failed to load games from database");

        rows.into_iter()
            .map(|r| {
                let authors: Vec<String> = r
                    .autoren_liste
                    .map(|list| list.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                Game {
                    name: r.Spieltitel,
                    year: r.Erscheinungsjahr,
                    players: r.spieler_range.unwrap_or_else(|| "N/A".to_string()),
                    duration: r.Spieldauer_Durchschnitt.unwrap_or(0),
                    age: r.Altersempfehlung.unwrap_or(0),
                    category: r.Kategorie.unwrap_or_default(),
                    publisher: r.Verlagsname,
                    authors,
                    rating: r.BGG_Rating.unwrap_or(0.0),
                }
            })
            .collect()
    }

    /// Removes a game record from the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The title of the game to delete.
    /// * `year` - The year of the game to delete.
    ///
    /// # Errors
    ///
    /// Returns an error if the deletion fails.
    async fn delete(&self, name: &str, year: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM Spiel WHERE Spieltitel = ? AND Erscheinungsjahr = ?",
            name,
            year as i16
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Performs an in-memory search for a game by name and year within the fetched list.
    ///
    /// # Arguments
    ///
    /// * `name` - The title to search for.
    /// * `year` - The year to search for.
    ///
    /// # Returns
    ///
    /// An optional `Game` if a match is found.
    async fn get_by_name_and_year(&self, name: &str, year: i32) -> Option<Game> {
        self.all()
            .await
            .into_iter()
            .find(|g| g.name == name && g.year == year)
    }

    /// Transactionally updates a game and its related player counts and author links.
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current title of the game to update.
    /// * `old_year` - The current year of the game to update.
    /// * `g` - The new `Game` entity containing updated information.
    ///
    /// # Errors
    ///
    /// Returns an error if any part of the transaction fails.
    async fn update(&self, old_name: &str, old_year: i32, g: Game) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE Spiel SET Spieltitel = ?, Erscheinungsjahr = ?, Kategorie = ?,
                   Altersempfehlung = ?, Spieldauer_Durchschnitt = ?, Verlagsname = ?, BGG_Rating = ?
             WHERE Spieltitel = ? AND Erscheinungsjahr = ?",
            g.name, g.year as i16, g.category, g.age, g.duration,
            g.publisher.as_deref(), g.rating, old_name, old_year as i16
        ).execute(&mut *tx).await?;

        let parts: Vec<&str> = g.players.split('-').collect();
        let min_p: i32 = parts[0].parse().unwrap_or(1);
        let max_p = parts
            .get(1)
            .map(|s| s.parse().unwrap_or(min_p))
            .unwrap_or(min_p);

        sqlx::query!(
            "UPDATE Spiel_Spieleranzahl SET Spieleranzahl_min = ?, Spieleranzahl_max = ?
             WHERE Spieltitel = ? AND Erscheinungsjahr = ?",
            min_p,
            max_p,
            g.name,
            g.year as i16
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "DELETE FROM Designs WHERE Spieltitel = ? AND Erscheinungsjahr = ?",
            g.name,
            g.year as i16
        )
        .execute(&mut *tx)
        .await?;

        for author_name in g.authors {
            let email = format!("{}@autor.de", author_name.replace(' ', ".").to_lowercase());

            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname) VALUES (?, ?, ?, ?)",
                email, author_name, "Autor", "Unbekannt"
            ).execute(&mut *tx).await?;

            sqlx::query!("INSERT IGNORE INTO Autor (Autor_Email) VALUES (?)", email)
                .execute(&mut *tx)
                .await?;

            sqlx::query!(
                "INSERT INTO Designs (Spieltitel, Erscheinungsjahr, Autor_Email) VALUES (?, ?, ?)",
                g.name,
                g.year as i16,
                email
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::repositories::tests::GameRepositoryContractTests,
        infrastructure::repositories::sql::test_utils::TestDbGuard,
    };
    use async_trait::async_trait;
    use sqlx::MySqlPool;

    /// Adapter to allow the generic `GameRepositoryContractTests` to test `SqlGameRepository`.
    struct SqlGameRepoContract {
        pool: MySqlPool,
    }

    #[async_trait]
    impl GameRepositoryContractTests for SqlGameRepoContract {
        /// Creates a new repository instance for testing.
        async fn create_repo(&self) -> Box<dyn GameRepository> {
            Box::new(SqlGameRepository::new(self.pool.clone()))
        }

        /// Ensures that publisher dependencies exist in the database for tests.
        async fn ensure_verlag_dependencies(&self, publisher: &str) {
            let addr_result =
                sqlx::query!(
                "INSERT INTO Verlag_Adresse (PLZ, Ort, Strasse, Hausnummer) VALUES (?, ?, ?, ?)",
                "12345", "Teststadt", "Teststraße", "1a"
            )
                .execute(&self.pool)
                .await
                .expect("Failed to insert Verlag_Adresse");

            let addr_id = addr_result.last_insert_id();

            sqlx::query!(
                "INSERT INTO Verlag (Verlagsname, Verlagsadresse) VALUES (?, ?)",
                publisher,
                addr_id as i32
            )
            .execute(&self.pool)
            .await
            .expect("Failed to insert Verlag");
        }

        /// Ensures that author dependencies exist in the database for tests.
        async fn ensure_autor_dependencies(&self, autor: &str) {
            let email = format!("{}@autor.de", autor.replace(' ', ".").to_lowercase());

            sqlx::query!(
                "INSERT IGNORE INTO Person (Email, Personname, Personenvorname, Personennachname)
                VALUES (?, ?, ?, ?)",
                email,
                autor,
                "Test",
                "Autor"
            )
            .execute(&self.pool)
            .await
            .expect("Failed to ensure Person for Autor exists");

            sqlx::query!("INSERT IGNORE INTO Autor (Autor_Email) VALUES (?)", email)
                .execute(&self.pool)
                .await
                .expect("Failed to ensure Autor exists");
        }
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    /// Tests that a game can be saved and retrieved by name.
    async fn sql__test__save_and_get_by_name() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();

        let contract = SqlGameRepoContract { pool };
        contract.test__save_and_get_by_name().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    /// Tests that `get_by_name_and_year` returns None if the game does not exist.
    async fn sql__test__get_by_name__not_found() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();

        let contract = SqlGameRepoContract { pool };
        contract.test__get_by_name__not_found().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    /// Tests that `all` returns all saved games.
    async fn sql__test__all__returns_all_saved_games() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();

        let contract = SqlGameRepoContract { pool };
        contract.test__all__returns_all_saved_games().await;
    }
}
