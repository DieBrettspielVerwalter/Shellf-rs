#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use anyhow::Result;
use expectrl::process::unix::{PtyStream, UnixProcess};
use expectrl::process::{Healthcheck, NonBlocking};
use expectrl::{Expect, Regex, Session};
use sqlx::MySqlPool;
use std::process::{Command, Stdio};
use std::time::Duration;

type PtySession =
    Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>>;

// --- DB SETUP (COPY) ---
pub struct TestDbGuard {
    pub pool: MySqlPool,
    pub db_url: String,
}

impl TestDbGuard {
    pub async fn new() -> Self {
        let base_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        // let db_name = format!("test_db_{}", uuid::Uuid::new_v4().simple());

        let root_url = base_url.replace("/SpieleDB", "/");
        let admin_pool = MySqlPool::connect(&root_url)
            .await
            .expect("Failed to connect to MariaDB root");

        // sqlx::raw_sql(&format!(
        //     "CREATE DATABASE `{}` DEFAULT CHARACTER SET utf8mb4;",
        //     db_name
        // ))
        // .execute(&admin_pool)
        // .await
        // .unwrap();

        // let isolated_url = base_url.replace("/SpieleDB", &format!("/{}", db_name));
        let pool = MySqlPool::connect(&base_url).await.unwrap();

        // let migrator = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        //     .await
        //     .unwrap();
        // migrator.run(&pool).await.unwrap();
        // for migration in migrator.iter() {
        //     println!(
        //         "Running migration: {} [{}]  -  for {}",
        //         migration.version,
        //         migrator.version_exists(migration.version),
        //         db_name
        //     );
        //     // if let Err(e) = migration.run(&pool).await {
        //     //     eprintln!("Migration {} failed: {:?}", migration.version, e);
        //     //     break;
        //     // }
        // }

        // sqlx::query("INSERT INTO Person (Email, Personenvorname, Personennachname) VALUES ('maria@spiele.de', 'Maria', 'Muster');").execute(&pool).await.unwrap();
        // sqlx::query("CREATE USER IF NOT EXISTS 'maria@spiele.de'@'%' IDENTIFIED BY '';")
        //     .execute(&pool)
        //     .await
        //     .unwrap();
        // sqlx::query("GRANT spieler_role TO 'maria@spiele.de'@'%';")
        //     .execute(&pool)
        //     .await
        //     .unwrap();
        // sqlx::query("SET DEFAULT ROLE spieler_role FOR 'maria@spiele.de'@'%';")
        //     .execute(&pool)
        //     .await
        //     .unwrap();
        // sqlx::query(
        //     "INSERT INTO Spieler (Email, Nickname) VALUES ('maria@spiele.de', 'mariaplay');",
        // )
        // .execute(&pool)
        // .await
        // .unwrap();

        // sqlx::query("GRANT manager_role TO 'maria@spiele.de'@'%';")
        //     .execute(&pool)
        //     .await
        //     .unwrap();
        // sqlx::query("SET DEFAULT ROLE manager_role FOR 'maria@spiele.de'@'%';")
        //     .execute(&pool)
        //     .await
        //     .unwrap();
        // sqlx::query(
        //     "GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Einfuegen TO 'maria@spiele.de'@'%';",
        // )
        // .execute(&pool)
        // .await
        // .unwrap();
        // sqlx::query(
        //     "GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Loeschen TO 'maria@spiele.de'@'%';",
        // )
        // .execute(&pool)
        // .await
        // .unwrap();
        // sqlx::query(
        //     "GRANT EXECUTE ON PROCEDURE Spieler_Ausleihe_Einfuegen TO 'maria@spiele.de'@'%';",
        // )
        // .execute(&pool)
        // .await
        // .unwrap();
        // sqlx::query("GRANT EXECUTE ON PROCEDURE Partie_Erstellen TO 'maria@spiele.de'@'%';")
        //     .execute(&pool)
        //     .await
        //     .unwrap();

        Self {
            pool,
            db_url: base_url,
            // db_name,
            // admin_pool,
        }
    }
}

// --- SPAWN ---
fn spawn_cli(db_url: &str) -> Result<PtySession> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_game_manager"));
    // cmd.stdout(Stdio::piped());
    // cmd.stderr(Stdio::piped());
    cmd.env("DATABASE_URL", db_url);
    cmd.env("LANG", "de_DE.UTF-8");
    cmd.env("LC_ALL", "de_DE.UTF-8");
    cmd.arg("--nocapture");

    println!(env!("CARGO_BIN_EXE_game_manager"));

    let mut p = Session::spawn(cmd)?;
    p.set_expect_timeout(Some(Duration::from_millis(200)));
    p.set_blocking(true)?;
    let p: Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>> =
        expectrl::session::log(p, std::io::stdout())?;
    Ok(p)
}

// --- LOGIN ---
fn login(p: &mut PtySession) -> Result<()> {
    p.expect(Regex("=== SHELLF LOGIN ==="))?;
    p.expect(Regex("E-Mail / DB-User"))?;
    p.send_line("maria@spiele.de")?;
    p.expect(Regex("Password"))?;
    p.send_line("")?;
    p.expect(Regex("Login successful"))?;
    Ok(())
}

// --- NAV HELPERS ---
fn nav_main(p: &mut PtySession, idx: usize) -> Result<()> {
    for _ in 0..idx {
        p.send("\x1b[B")?;
        p.expect("❯")?;
    }
    p.send("\n")?;
    Ok(())
}

fn nav_sub(p: &mut PtySession, idx: usize) -> Result<()> {
    for _ in 0..idx {
        p.send("\x1b[B")?;
        p.expect("❯")?;
    }
    p.send("\n")?;
    Ok(())
}

// =====================================================
// 1. Manage Games & Shelf
// =====================================================

#[tokio::test]
async fn mg_capture_new_game() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 0)?;
    p.expect(Regex("Search"))?;
    Ok(())
}

#[tokio::test]
async fn mg_show_game_list() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 1)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    p.expect(Regex("Registered Games"))?;
    Ok(())
}

#[tokio::test]
async fn mg_edit_game() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 2)?;
    p.expect(Regex("Select a game"))?;
    Ok(())
}

#[tokio::test]
async fn mg_delete_game() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 3)?;
    p.expect(Regex("Select a game"))?;
    p.send("\t\n")?;
    // p.expect(Regex("Do you really want to delete Game"))?;
    Ok(())
}

#[tokio::test]
async fn mg_add_game_copy() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 4)?;
    p.expect(Regex("Select the game from your collection"))?;
    Ok(())
}

#[tokio::test]
async fn mg_edit_game_copy() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 5)?;
    p.expect(Regex("Which copy would you like to edit"))?;
    Ok(())
}

#[tokio::test]
async fn mg_delete_game_copy() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 6)?;
    p.expect(Regex("Select copy"))?;
    Ok(())
}

#[tokio::test]
async fn mg_show_shelf() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 7)?;
    p.expect(Regex("Shelf"))?;
    Ok(())
}

#[tokio::test]
async fn mg_back() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 0)?;
    nav_sub(&mut p, 8)?;
    p.expect(Regex("Main Menu"))?;
    Ok(())
}

// =====================================================
// 2. Players & Lending
// =====================================================

#[tokio::test]
async fn pl_new_player() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 0)?;
    p.expect(Regex("CREATE NEW PLAYER"))?;
    Ok(())
}

#[tokio::test]
async fn pl_show_players() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 1)?;
    p.expect(Regex("Player List"))?;
    Ok(())
}

#[tokio::test]
async fn pl_edit_player() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 2)?;
    p.expect(Regex("Which player would you like to edit"))?;
    Ok(())
}

#[tokio::test]
async fn pl_lend_game() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 3)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("-------------------------------------");
    p.expect(Regex("Select copy"))?;
    Ok(())
}

#[tokio::test]
async fn pl_return_game() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 4)?;
    p.expect(Regex("Select return"))?;
    Ok(())
}

#[tokio::test]
async fn pl_show_lending() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 5)?;
    p.expect(Regex("Lent to"))?;
    Ok(())
}

#[tokio::test]
async fn pl_back() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 1)?;
    nav_sub(&mut p, 6)?;
    p.expect(Regex("Main Menu"))?;
    Ok(())
}

// =====================================================
// 3. Planning & Sessions
// =====================================================

#[tokio::test]
async fn ps_scheduled_nights() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 0)?;
    p.expect(Regex("Scheduled Nights"))?;
    Ok(())
}

#[tokio::test]
async fn ps_plan_new() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 1)?;
    p.expect(Regex("Plan Game Night"))?;
    Ok(())
}

#[tokio::test]
async fn ps_show_my_shelf() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 2)?;
    p.expect(Regex("Shelf"))?;
    Ok(())
}

#[tokio::test]
async fn ps_record_session() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 3)?;
    p.expect(Regex("Which copy was played"))?;
    Ok(())
}

#[tokio::test]
async fn ps_edit_session() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 4)?;
    p.expect(Regex("Select Session"))?;
    Ok(())
}

#[tokio::test]
async fn ps_delete_session() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 5)?;
    p.expect(Regex("Select Session"))?;
    Ok(())
}

#[tokio::test]
async fn ps_show_history() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 6)?;
    p.expect(Regex("Session History"))?;
    Ok(())
}

#[tokio::test]
async fn ps_back() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 2)?;
    nav_sub(&mut p, 7)?;
    p.expect(Regex("Main Menu"))?;
    Ok(())
}

// =====================================================
// 4. Display Help
// =====================================================

#[tokio::test]
async fn help_enter() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 3)?;
    p.expect(Regex("HELP SYSTEM"))?;
    Ok(())
}

// =====================================================
// 5. Exit
// =====================================================

#[tokio::test]
async fn exit_enter() -> Result<()> {
    let db = TestDbGuard::new().await;
    let mut p = spawn_cli(&db.db_url)?;
    login(&mut p)?;
    nav_main(&mut p, 4)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    // println!("ALIVE: {}", p.is_alive()?);
    // assert!(!p.is_alive()?);
    Ok(())
}
