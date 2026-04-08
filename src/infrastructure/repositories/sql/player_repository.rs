use crate::domain::{Player, PlayerRepository};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlPool;

/// MariaDB-backed implementation of the `PlayerRepository`.
///
/// This repository manages both the domain data for players and the
/// underlying database infrastructure by creating actual MariaDB users
/// for each registered player.
pub struct SqlPlayerRepository {
    pool: MySqlPool,
}

impl SqlPlayerRepository {
    /// Creates a new instance of `SqlPlayerRepository`.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlayerRepository for SqlPlayerRepository {
    /// Registers a new player by creating a database user and inserting domain records.
    ///
    /// This is a two-step process:
    /// 1. **Infrastructure**: Creates a MariaDB user identified by the provided password
    ///    and grants the `spieler_role`.
    /// 2. **Domain**: Inserts records into the `Person` and `Spieler` tables within a transaction.
    ///
    /// # Errors
    /// Returns an error if the user creation fails, the email is already taken,
    /// or the transaction cannot be committed.
    async fn save(&self, player: Player, password: String) -> Result<Player> {
        // 1. Database User Management (Infrastructure Level)
        let create_user_sql = format!(
            "CREATE USER IF NOT EXISTS '{}'@'%' IDENTIFIED BY '{}'",
            player.email, password
        );
        sqlx::query(&create_user_sql).execute(&self.pool).await?;

        let grant_sql = format!("GRANT spieler_role TO '{}'@'%'", player.email);
        sqlx::query(&grant_sql).execute(&self.pool).await?;

        // 2. Data Persistence (Domain Level)
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO Person (Email, Personenvorname, Personennachname, Personname, Notizen) VALUES (?, ?, ?, ?, ?)",
            player.email, player.first_name, player.last_name,
            format!("{} {}", player.first_name, player.last_name),
            player.details
        ).execute(&mut *tx).await?;

        sqlx::query!(
            "INSERT INTO Spieler (Email, Nickname) VALUES (?, ?)",
            player.email,
            player.nickname
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(player)
    }

    /// Fetches all players by joining the `Person` and `Spieler` tables.
    async fn all(&self) -> Vec<Player> {
        let rows = sqlx::query!(
            r#"
            SELECT
                p.Email, s.Nickname, p.Personenvorname, p.Personennachname, p.Notizen
            FROM Person p
            JOIN Spieler s ON p.Email = s.Email
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(|r| Player {
                email: r.Email,
                nickname: r.Nickname.unwrap_or_default(),
                first_name: r.Personenvorname,
                last_name: r.Personennachname,
                details: r.Notizen,
            })
            .collect()
    }

    /// Retrieves a single player by their email address.
    ///
    /// Returns `None` if the player does not exist or the join fails.
    async fn get_by_email(&self, email: &str) -> Option<Player> {
        let r = sqlx::query!(
            r#"
            SELECT
                p.Email, s.Nickname, p.Personenvorname, p.Personennachname, p.Notizen
            FROM Person p
            JOIN Spieler s ON p.Email = s.Email
            WHERE p.Email = ?
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        .ok()?;

        Some(Player {
            email: r.Email,
            nickname: r.Nickname.unwrap_or_default(),
            first_name: r.Personenvorname,
            last_name: r.Personennachname,
            details: r.Notizen,
        })
    }

    /// Transactionally updates player records in both `Person` and `Spieler` tables.
    ///
    /// # Arguments
    /// * `old_email` - The current email used to identify the records.
    /// * `player` - The updated domain entity.
    async fn update(&self, old_email: &str, player: Player) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE Person SET Email = ?, Personennachname = ?, Personenvorname = ?, Notizen = ? WHERE Email = ?",
            player.email, player.last_name, player.first_name, player.details, old_email
        ).execute(&mut *tx).await?;

        sqlx::query!(
            "UPDATE Spieler SET Nickname = ? WHERE Email = ?",
            player.nickname,
            player.email
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::repositories::tests::PlayerRepositoryContractTests,
        infrastructure::repositories::sql::test_utils::TestDbGuard,
    };
    use async_trait::async_trait;
    use sqlx::MySqlPool;

    struct SqlPlayerRepoContract {
        pool: MySqlPool,
    }

    #[async_trait]
    impl PlayerRepositoryContractTests for SqlPlayerRepoContract {
        async fn create_repo(&self) -> Box<dyn PlayerRepository> {
            Box::new(SqlPlayerRepository::new(self.pool.clone()))
        }
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__save_and_get_by_email() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.admin_pool.clone();

        let contract = SqlPlayerRepoContract { pool };
        contract.test__save_and_get_by_email().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__get_by_email__not_found() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.admin_pool.clone();

        let contract = SqlPlayerRepoContract { pool };
        contract.test__get_by_email__not_found().await;
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn sql__test__all__returns_all_saved_players() {
        let _guard = TestDbGuard::new().await;
        let pool = _guard.admin_pool.clone();

        let contract = SqlPlayerRepoContract { pool };
        contract.test__all__returns_all_saved_players().await;
    }
}
