pub mod game_copy_repository;
pub mod game_night_repository;
pub mod game_repository;
pub mod game_session_repository;
pub mod player_repository;

/// Utilities for infrastructure-level testing and database isolation.
#[cfg(test)]
pub mod test_utils {
    use sqlx::{
        mysql::{MySqlConnectOptions, MySqlPoolOptions},
        MySqlPool,
    };
    use uuid::Uuid;

    /// A RAII (Resource Acquisition Is Initialization) guard for isolated test databases.
    ///
    /// This guard creates a unique, temporary database for each test to ensure
    /// absolute isolation and prevent side effects between parallel test runs.
    /// The database is automatically dropped when the guard is out of scope.
    pub struct TestDbGuard {
        /// The connection pool for the isolated test database.
        pub pool: MySqlPool,
        /// The unique name of the temporary database.
        db_name: String,
        /// An administrative connection pool used to manage database lifecycle.
        pub admin_pool: MySqlPool,
    }

    impl TestDbGuard {
        /// Creates a new isolated database and initializes it with the required schema.
        ///
        /// This method:
        /// 1. Connects to the database server using administrative privileges.
        /// 2. Generates a unique database name.
        /// 3. Executes the schema defined in `Testpool.sql`.
        ///
        /// # Returns
        ///
        /// A new `TestDbGuard` instance ready for testing.
        ///
        /// # Panics
        ///
        /// Panics if the `DATABASE_URL` environment variable is missing or if
        /// the database creation/initialization fails.
        pub async fn new() -> Self {
            let base_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let db_name = format!("test_db_{}", Uuid::new_v4().simple());
            let root_url = base_url.replace("/SpieleDB", "/");
            let admin_pool = MySqlPool::connect(&root_url)
                .await
                .expect("Failed to connect to MariaDB root");

            sqlx::query(&format!("CREATE DATABASE `{}`", db_name))
                .execute(&admin_pool)
                .await
                .expect("Failed to create test database");

            let isolated_url = base_url.replace("/SpieleDB", &format!("/{}", db_name));
            let pool = MySqlPool::connect(&isolated_url)
                .await
                .expect("Failed to connect to isolated pool");

            // let schema = include_str!("../../../../Testpool.sql");
            // raw_sql(schema).execute(&pool).await.unwrap();
            // sqlx::migrate!("./migrations").run(&pool).await.unwrap();

            let migrator = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
                .await
                .unwrap();
            migrator.run(&pool).await.unwrap();
            for migration in migrator.iter() {
                println!(
                    "Running migration: {} [{}]  -  for {}",
                    migration.version,
                    migrator.version_exists(migration.version),
                    db_name
                );
                // if let Err(e) = migration.run(&pool).await {
                //     eprintln!("Migration {} failed: {:?}", migration.version, e);
                //     break;
                // }
            }

            sqlx::query("INSERT INTO Person (Email, Personenvorname, Personennachname) VALUES ('maria@spiele.de', 'Maria', 'Muster');").execute(&pool).await.unwrap();
            sqlx::query("CREATE USER IF NOT EXISTS 'maria@spiele.de'@'%' IDENTIFIED BY '';")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("GRANT spieler_role TO 'maria@spiele.de'@'%';")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("SET DEFAULT ROLE spieler_role FOR 'maria@spiele.de'@'%';")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query(
                "INSERT INTO Spieler (Email, Nickname) VALUES ('maria@spiele.de', 'mariaplay');",
            )
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query("GRANT manager_role TO 'maria@spiele.de'@'%';")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("SET DEFAULT ROLE manager_role FOR 'maria@spiele.de'@'%';")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Einfuegen TO 'maria@spiele.de'@'%';").execute(&pool).await.unwrap();
            sqlx::query(
                "GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Loeschen TO 'maria@spiele.de'@'%';",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "GRANT EXECUTE ON PROCEDURE Spieler_Ausleihe_Einfuegen TO 'maria@spiele.de'@'%';",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query("GRANT EXECUTE ON PROCEDURE Partie_Erstellen TO 'maria@spiele.de'@'%';")
                .execute(&pool)
                .await
                .unwrap();

            // let maries_url = isolated_url.replace("root", "marie@spiele.de:12345");
            let opts = MySqlConnectOptions::new()
                .socket(std::env::var("MYSQL_UNIX_PORT").expect("MYSQL_UNIX_PORT not set"))
                .username("maria@spiele.de")
                .database(&db_name.clone());

            println!("Verbindung wird aufgebaut als `maria@spiele.de`...");

            let pool = MySqlPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect_with(opts)
                .await
                .expect("Login fehlgeschlagen!");

            let root_url = base_url.replace("/SpieleDB", &format!("/{db_name}"));
            let admin_pool = MySqlPool::connect(&root_url)
                .await
                .expect("Failed to connect to MariaDB root");

            println!("✅ Erfolgreich angemeldet!");

            Self {
                pool,
                db_name,
                admin_pool,
            }
        }
    }

    impl Drop for TestDbGuard {
        /// Automatically drops the temporary database when the guard is dropped.
        ///
        /// Note: This spawns a background task since `drop` is synchronous,
        /// but database cleanup requires asynchronous operations.
        fn drop(&mut self) {
            let db = self.db_name.clone();
            let admin = self.admin_pool.clone();
            tokio::spawn(async move {
                let _ = sqlx::query(&format!("DROP DATABASE `{}`", db))
                    .execute(&admin)
                    .await;
            });
        }
    }
}
