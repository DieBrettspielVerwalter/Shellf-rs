use crate::domain::{GameCopy, GameCopyRepository};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use sqlx::MySqlPool;

/// MariaDB-backed implementation of the `GameCopyRepository`.
///
/// This repository interacts with the database using `sqlx` and leverages
/// stored procedures and views to enforce business rules and security
/// at the database level.
pub struct SqlGameCopyRepository {
    pool: MySqlPool,
}

impl SqlGameCopyRepository {
    /// Creates a new `SqlGameCopyRepository` with the provided connection pool.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GameCopyRepository for SqlGameCopyRepository {
    /// Saves a new game copy using the `Spieler_EigeneKopie_Einfuegen` procedure.
    ///
    /// The procedure automatically handles the association with the current user.
    ///
    /// # Returns
    /// The ID of the newly inserted copy.
    async fn save(&self, copy: GameCopy) -> Result<i32> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "CALL Spieler_EigeneKopie_Einfuegen(?, ?)",
            copy.game_name,
            copy.game_year as i16
        )
        .execute(&mut *tx)
        .await?;

        let rec = sqlx::query!("SELECT LAST_INSERT_ID() as id")
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(rec.id as i32)
    }

    /// Retrieves all game copies, adjusting the scope based on the user's role.
    ///
    /// If the user has `dba_role` or is `root`, it fetches all records from the
    /// base tables. Otherwise, it uses the `Spieler_Kopien_und_Ausleihen` view
    /// for row-level security.
    async fn all(&self) -> Vec<GameCopy> {
        let identity = sqlx::query!("SELECT USER_EMAIL() as mail, CURRENT_ROLE() as role")
            .fetch_one(&self.pool)
            .await;

        let (_user_mail, is_admin) = match identity {
            Ok(rec) => {
                let mail = rec.mail.unwrap_or_default();
                let role = rec.role.unwrap_or_default();
                (mail.clone(), mail == "root" || role.contains("dba_role"))
            }
            _ => ("unknown".to_string(), false),
        };

        if is_admin {
            let result = sqlx::query!(
                r#"
                SELECT
                    sk.Kopie_ID, sk.Spieltitel, CAST(sk.Erscheinungsjahr AS SIGNED) as Jahr,
                    sk.Besitzer_Email, a.Spieler_Email as Ausleiher_Email, a.Ausleihstartdatum
                FROM Spielkopie sk
                LEFT JOIN Ausleihe a ON sk.Kopie_ID = a.Kopie_ID AND a.Ausleihenddatum = '9999-12-31'
                "#
            )
                .fetch_all(&self.pool)
                .await;

            match result {
                Ok(rows) => rows
                    .into_iter()
                    .map(|r| GameCopy {
                        id: r.Kopie_ID,
                        game_name: r.Spieltitel,
                        game_year: r.Jahr,
                        owner_id: r.Besitzer_Email,
                        is_lent: r.Ausleiher_Email.is_some(),
                        borrower_email: r.Ausleiher_Email,
                        borrow_date: r.Ausleihstartdatum,
                        ..Default::default()
                    })
                    .collect(),
                Err(_) => Vec::new(),
            }
        } else {
            let result = sqlx::query!(
                r#"
                SELECT
                    Kopie_ID, Spieltitel, CAST(Erscheinungsjahr AS SIGNED) as Jahr,
                    Besitzer_Email, Ausleiher_Email, Ausleihstartdatum
                FROM Spieler_Kopien_und_Ausleihen
                "#
            )
            .fetch_all(&self.pool)
            .await;

            match result {
                Ok(rows) => rows
                    .into_iter()
                    .map(|r| GameCopy {
                        id: r.Kopie_ID,
                        game_name: r.Spieltitel,
                        game_year: r.Jahr,
                        owner_id: r.Besitzer_Email,
                        is_lent: r.Ausleiher_Email.is_some(),
                        borrower_email: r.Ausleiher_Email,
                        borrow_date: r.Ausleihstartdatum,
                        ..Default::default()
                    })
                    .collect(),
                Err(_) => Vec::new(),
            }
        }
    }

    /// Fetches a single game copy by ID using the player-specific view.
    async fn get_by_id(&self, id: i32) -> Option<GameCopy> {
        let row = sqlx::query!(
            r#"
            SELECT
                Kopie_ID, Spieltitel, CAST(Erscheinungsjahr AS SIGNED) as Jahr,
                Besitzer_Email, Ausleiher_Email, Ausleihstartdatum
            FROM Spieler_Kopien_und_Ausleihen
            WHERE Kopie_ID = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()?;

        Some(GameCopy {
            id: row.Kopie_ID,
            game_name: row.Spieltitel,
            game_year: row.Jahr as i32,
            owner_id: row.Besitzer_Email,
            is_lent: row.Ausleiher_Email.is_some(),
            borrower_email: row.Ausleiher_Email,
            borrow_date: row.Ausleihstartdatum,
            ..Default::default()
        })
    }

    /// Initiates a lending process via the `Spieler_Ausleihe_Einfuegen` procedure.
    async fn lend_copy(
        &self,
        copy_id: i32,
        borrower_email: &str,
        start_date: NaiveDate,
        due_date: Option<NaiveDate>,
    ) -> Result<()> {
        sqlx::query!(
            "CALL Spieler_Ausleihe_Einfuegen(?, ?, ?, ?)",
            copy_id,
            borrower_email,
            start_date,
            due_date
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Retrieves all active lendings, handling the MariaDB '9999-12-31' placeholder for open returns.
    async fn all_lent(&self) -> Result<Vec<GameCopy>> {
        let identity = sqlx::query!("SELECT USER_EMAIL() as mail, CURRENT_ROLE() as role")
            .fetch_one(&self.pool)
            .await?;

        let user_mail = identity.mail.unwrap_or_default();
        let role = identity.role.unwrap_or_default();
        let is_admin = user_mail == "root" || role.contains("dba_role");

        if is_admin {
            let rows = sqlx::query!(
                r#"
                SELECT
                    a.Kopie_ID as "id: i32",
                    sk.Spieltitel as "game_name",
                    CAST(sk.Erscheinungsjahr AS SIGNED) as "game_year: i32",
                    sk.Besitzer_Email as "owner_id",
                    a.Spieler_Email as "borrower: String",
                    a.Ausleihstartdatum as "borrow_date",
                    a.Ausleihenddatum as "due_date?"
                FROM Ausleihe a
                JOIN Spielkopie sk ON a.Kopie_ID = sk.Kopie_ID
                WHERE a.Ausleihenddatum IS NULL
                   OR a.Ausleihenddatum >= CURDATE()
                   OR a.Ausleihenddatum = '9999-12-31'
                "#
            )
            .fetch_all(&self.pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|r| GameCopy {
                    id: r.id,
                    game_name: r.game_name,
                    game_year: r.game_year,
                    owner_id: r.owner_id,
                    is_lent: true,
                    borrower_email: Some(r.borrower),
                    borrow_date: Some(r.borrow_date),
                    due_date: match r.due_date {
                        Some(d) if d.year() == 9999 => None,
                        Some(d) => Some(d),
                        _ => None,
                    },
                })
                .collect())
        } else {
            let rows = sqlx::query!(
                r#"
                SELECT
                    v.Kopie_ID as "id: i32",
                    sk.Spieltitel as "game_name",
                    CAST(sk.Erscheinungsjahr AS SIGNED) as "game_year: i32",
                    sk.Besitzer_Email as "owner_id",
                    v.Spieler_Email as "borrower: String",
                    v.Ausleihstartdatum as "borrow_date",
                    v.Ausleihenddatum as "due_date?"
                FROM Spieler_Ausleihe v
                JOIN Spielkopie sk ON v.Kopie_ID = sk.Kopie_ID
                WHERE v.Ausleihenddatum IS NULL
                   OR v.Ausleihenddatum >= CURDATE()
                "#
            )
            .fetch_all(&self.pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|r| GameCopy {
                    id: r.id,
                    game_name: r.game_name,
                    game_year: r.game_year,
                    owner_id: r.owner_id,
                    is_lent: true,
                    borrower_email: Some(r.borrower),
                    borrow_date: Some(r.borrow_date),
                    due_date: match r.due_date {
                        Some(d) if d.year() == 9999 => None,
                        Some(d) => Some(d),
                        _ => None,
                    },
                })
                .collect())
        }
    }

    /// Sets the return date for an active lending record.
    async fn return_copy(&self, copy_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE Ausleihe SET Ausleihenddatum = CURDATE()
             WHERE Kopie_ID = ? AND (Ausleihenddatum >= CURDATE() OR Ausleihenddatum = '9999-12-31')",
            copy_id
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Deletes a game copy from the `Spielkopie` table.
    async fn delete(&self, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM Spielkopie WHERE Kopie_ID = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Updates the core data of a game copy.
    async fn update(&self, old_id: i32, copy: GameCopy) -> Result<()> {
        sqlx::query!(
            "UPDATE Spielkopie SET Kopie_ID = ?, Besitzer_Email = ? WHERE Kopie_ID = ?",
            copy.id,
            copy.owner_id,
            old_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::repositories::tests::GameCopyRepositoryContractTests,
        infrastructure::repositories::sql::test_utils::TestDbGuard,
    };
    use async_trait::async_trait;
    use sqlx::MySqlPool;

    struct SqlGameCopyRepoContract {
        pool: MySqlPool,
        admin_pool: MySqlPool,
    }

    #[async_trait]
    impl GameCopyRepositoryContractTests for SqlGameCopyRepoContract {
        async fn create_repo(&self) -> Box<dyn GameCopyRepository> {
            Box::new(SqlGameCopyRepository::new(self.pool.clone()))
        }

        async fn pool(&self) -> MySqlPool {
            self.pool.clone()
        }

        async fn ensure_dependencies(&self, game_name: &str, year: i32) {
            // Erstellt das Spiel, falls es nicht existiert (Foreign Key Schutz)
            sqlx::query!(
                "INSERT IGNORE INTO Spiel (Spieltitel, Erscheinungsjahr) VALUES (?, ?)",
                game_name,
                year as i16
            )
            .execute(&self.admin_pool)
            .await
            .unwrap();
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
    async fn sql__test__save_and_get_by_id() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameCopyRepoContract { pool, admin_pool };
        contract.test__save_and_get_by_id().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__get_by_id__not_found() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameCopyRepoContract { pool, admin_pool };
        contract.test__get_by_id__not_found().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__all__returns_all_saved_copies() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameCopyRepoContract { pool, admin_pool };
        contract.test__all__returns_all_saved_copies().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__delete_removes_copy() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameCopyRepoContract { pool, admin_pool };
        contract.test__delete_removes_copy().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__lend_and_return_copy() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.pool.clone();
        let admin_pool = _guard.admin_pool.clone();

        let contract = SqlGameCopyRepoContract { pool, admin_pool };
        contract.test__lend_and_return_copy().await;
    }
}
