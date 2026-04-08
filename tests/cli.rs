#![allow(missing_docs)]
// use anyhow::Result;
// use expectrl::process::unix::{PtyStream, UnixProcess};
// use expectrl::process::NonBlocking;
// use expectrl::{self, ControlCode, Expect, Regex, Session};
// use sqlx::MySqlPool;
// use std::process::Command;
// use std::time::Duration;

// type PtySession =
//     Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>>;

// // --- 1. TEST-DATABASE SETUP ---
// pub struct TestDbGuard {
//     pub pool: MySqlPool,
//     pub db_url: String,
//     db_name: String,
//     admin_pool: MySqlPool,
// }

// impl TestDbGuard {
//     pub async fn new() -> Self {
//         let base_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//         // let db_name = format!("test_db_{}", uuid::Uuid::new_v4().simple());
//         let db_name = "SpieleDB".into();

//         let root_url = base_url.replace("/SpieleDB", "/");
//         let admin_pool = MySqlPool::connect(&root_url)
//             .await
//             .expect("Failed to connect to MariaDB root");

//         // sqlx::raw_sql(&format!(
//         //     "CREATE DATABASE `{}` DEFAULT CHARACTER SET utf8mb4;",
//         //     db_name
//         // ))
//         // .execute(&admin_pool)
//         // .await
//         // .unwrap();

//         // let isolated_url = base_url.replace("/SpieleDB", &format!("/{}", db_name));
//         let pool = MySqlPool::connect(&base_url).await.unwrap();

//         // let migrator = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
//         //     .await
//         //     .unwrap();
//         // migrator.run(&pool).await.unwrap();
//         // for migration in migrator.iter() {
//         //     println!(
//         //         "Running migration: {} [{}]  -  for {}",
//         //         migration.version,
//         //         migrator.version_exists(migration.version),
//         //         db_name
//         //     );
//         //     // if let Err(e) = migration.run(&pool).await {
//         //     //     eprintln!("Migration {} failed: {:?}", migration.version, e);
//         //     //     break;
//         //     // }
//         // }

//         // sqlx::query("INSERT INTO Person (Email, Personenvorname, Personennachname) VALUES ('maria@spiele.de', 'Maria', 'Muster');").execute(&pool).await.unwrap();
//         // sqlx::query("CREATE USER IF NOT EXISTS 'maria@spiele.de'@'%' IDENTIFIED BY '';")
//         //     .execute(&pool)
//         //     .await
//         //     .unwrap();
//         // sqlx::query("GRANT spieler_role TO 'maria@spiele.de'@'%';")
//         //     .execute(&pool)
//         //     .await
//         //     .unwrap();
//         // sqlx::query("SET DEFAULT ROLE spieler_role FOR 'maria@spiele.de'@'%';")
//         //     .execute(&pool)
//         //     .await
//         //     .unwrap();
//         // sqlx::query(
//         //     "INSERT INTO Spieler (Email, Nickname) VALUES ('maria@spiele.de', 'mariaplay');",
//         // )
//         // .execute(&pool)
//         // .await
//         // .unwrap();

//         // sqlx::query("GRANT manager_role TO 'maria@spiele.de'@'%';")
//         //     .execute(&pool)
//         //     .await
//         //     .unwrap();
//         // sqlx::query("SET DEFAULT ROLE manager_role FOR 'maria@spiele.de'@'%';")
//         //     .execute(&pool)
//         //     .await
//         //     .unwrap();
//         // sqlx::query(
//         //     "GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Einfuegen TO 'maria@spiele.de'@'%';",
//         // )
//         // .execute(&pool)
//         // .await
//         // .unwrap();
//         // sqlx::query(
//         //     "GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Loeschen TO 'maria@spiele.de'@'%';",
//         // )
//         // .execute(&pool)
//         // .await
//         // .unwrap();
//         // sqlx::query(
//         //     "GRANT EXECUTE ON PROCEDURE Spieler_Ausleihe_Einfuegen TO 'maria@spiele.de'@'%';",
//         // )
//         // .execute(&pool)
//         // .await
//         // .unwrap();
//         // sqlx::query("GRANT EXECUTE ON PROCEDURE Partie_Erstellen TO 'maria@spiele.de'@'%';")
//         //     .execute(&pool)
//         //     .await
//         //     .unwrap();

//         Self {
//             pool,
//             db_url: base_url,
//             db_name,
//             admin_pool,
//         }
//     }
// }

// impl Drop for TestDbGuard {
//     fn drop(&mut self) {
//         let db_name = self.db_name.clone();
//         let admin_pool = self.admin_pool.clone();

//         tokio::spawn(async move {
//             let _ = sqlx::raw_sql(&format!("DROP DATABASE `{}`;", db_name))
//                 .execute(&admin_pool)
//                 .await;
//         });
//     }
// }

// // --- 2. HELPER FUNCTIONS FOR CLI INTERACTION ---
// fn spawn_cli(db_url: &str) -> Result<PtySession> {
//     let mut cmd = Command::new(env!("CARGO_BIN_EXE_game_manager"));
//     cmd.env("LANG", "de_DE.UTF-8");
//     cmd.env("LC_ALL", "de_DE.UTF-8");
//     cmd.env("DATABASE_URL", db_url);

//     let mut p = Session::spawn(cmd)?;
//     p.set_expect_timeout(Some(Duration::from_millis(200)));
//     let _ = p.set_blocking(true);
//     let p: Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>> =
//         expectrl::session::log(p, std::io::stdout())?;
//     Ok(p)
// }

// // --- 2. HELPER FUNCTIONS FOR CLI INTERACTION ---
// fn spawn_cli_offline(db_url: &str) -> Result<PtySession> {
//     let mut cmd = Command::new("unshare");
//     cmd.args([
//         "--user",
//         "--map-root-user",
//         "--net",
//         env!("CARGO_BIN_EXE_game_manager"),
//     ]);
//     cmd.env("LANG", "de_DE.UTF-8");
//     cmd.env("LC_ALL", "de_DE.UTF-8");
//     cmd.env("DATABASE_URL", db_url);

//     let mut p = Session::spawn(cmd)?;
//     p.set_expect_timeout(Some(Duration::from_millis(200)));
//     let _ = p.set_blocking(true);
//     let p: Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>> =
//         expectrl::session::log(p, std::io::stdout())?;
//     Ok(p)
// }
// // --- 2. HELPER FUNCTIONS FOR CLI INTERACTION ---
// fn spawn_cli_debug(db_url: &str) -> Result<PtySession> {
//     let mut cmd = Command::new(env!("CARGO_BIN_EXE_game_manager"));
//     cmd.arg("--debug");
//     cmd.env("LANG", "de_DE.UTF-8");
//     cmd.env("LC_ALL", "de_DE.UTF-8");
//     cmd.env("DATABASE_URL", db_url);

//     let mut p = Session::spawn(cmd)?;
//     p.set_expect_timeout(Some(Duration::from_millis(200)));
//     let _ = p.set_blocking(true);
//     let p: Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>> =
//         expectrl::session::log(p, std::io::stdout())?;
//     Ok(p)
// }

// // --- 2. HELPER FUNCTIONS FOR CLI INTERACTION ---
// fn spawn_cli_debug_offline(db_url: &str) -> Result<PtySession> {
//     let mut cmd = Command::new("unshare");
//     cmd.args([
//         "--user",
//         "--map-root-user",
//         "--net",
//         env!("CARGO_BIN_EXE_game_manager"),
//         "--debug"
//     ]);
//     cmd.env("LANG", "de_DE.UTF-8");
//     cmd.env("LC_ALL", "de_DE.UTF-8");
//     cmd.env("DATABASE_URL", db_url);

//     let mut p = Session::spawn(cmd)?;
//     p.set_expect_timeout(Some(Duration::from_millis(200)));
//     let _ = p.set_blocking(true);
//     let p: Session<UnixProcess, expectrl::stream::log::LogStream<PtyStream, std::io::Stdout>> =
//         expectrl::session::log(p, std::io::stdout())?;
//     Ok(p)
// }

// fn login(p: &mut PtySession, email: &str) -> Result<()> {
//     p.expect(Regex("=== SHELLF LOGIN ==="))?;
//     // p.send(ControlCode::Cancel)?;
//     p.expect(Regex("E-Mail / DB-User"))?;
//     p.send_line(email)?;
//     p.expect(Regex("Password"))?;
//     p.send_line("\n")?;
//     p.expect(Regex("Login successful"))?;
//     Ok(())
// }

// pub mod main_menu {
//     use super::*;

//     pub fn check_main_menu(p: &mut PtySession) -> Result<()> {
//         // p.expect(Regex("Main Menu"))?;
//         p.expect(Regex("❯.*Manage Games & Shelf"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Players & Lending"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Planning & Sessions"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Display Help"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Exit"))?;

//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Manage Games & Shelf"))?;
//         Ok(())
//     }

//     pub fn navigate_to(p: &mut PtySession, item: &str) -> Result<()> {
//         let menu_items = [
//             "Manage Games & Shelf",
//             "Players & Lending",
//             "Planning & Sessions",
//             "Display Help",
//             "Exit",
//         ];

//         for (idx, name) in menu_items.iter().enumerate() {
//             if *name == item {
//                 for _ in 0..idx {
//                     p.send("\x1b[B")?;
//                     p.expect("❯")?;
//                 }
//                 p.send("\n")?;
//                 return Ok(());
//             }
//         }
//         anyhow::bail!(format!("Menu item not found: {}", item))
//     }
// }

// pub mod spiele_regal {
//     use super::*;

//     pub fn check_spiele_regal_menu(p: &mut PtySession) -> Result<()> {
//         // p.expect(Regex("Shelf Management"))?;
//         p.expect(Regex("❯.*Capture New Game"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Show Game List"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Edit Game"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Delete Game"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Add Game Copy"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Edit Game Copy"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Delete Game Copy"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Show Shelf"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*<< Back"))?;

//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Capture New Game"))?;
//         Ok(())
//     }

//     pub fn navigate_to(p: &mut PtySession, item: &str) -> Result<()> {
//         let menu_items = [
//             "Capture New Game",
//             "Show Game List",
//             "Edit Game",
//             "Delete Game",
//             "Add Game Copy",
//             "Edit Game Copy",
//             "Delete Game Copy",
//             "Show Shelf",
//             "<< Back",
//         ];

//         for (idx, name) in menu_items.iter().enumerate() {
//             if *name == item {
//                 for _ in 0..idx {
//                     p.send("\x1b[B")?;
//                     p.expect("❯")?;
//                 }
//                 p.send("\n")?;
//                 return Ok(());
//             }
//         }
//         anyhow::bail!(format!("Menu item not found: {}", item))
//     }
// }
// pub mod spieler_verleih {
//     use super::*;

//     pub fn check_spieler_verleih_menu(p: &mut PtySession) -> Result<()> {
//         // p.expect(Regex("Players & Lending"))?;
//         p.expect(Regex("❯.*New Player"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Show Player List"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Edit Player"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Lend Game"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Return Game"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Show Lending Overview"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*<< Back"))?;

//         // Reset cursor to top
//         p.send("\x1b[B")?;
//         Ok(())
//     }

//     pub fn navigate_to(p: &mut PtySession, item: &str) -> Result<()> {
//         let menu_items = [
//             "New Player",
//             "Show Player List",
//             "Edit Player",
//             "Lend Game",
//             "Return Game",
//             "Show Lending Overview",
//             "<< Back",
//         ];

//         for (idx, name) in menu_items.iter().enumerate() {
//             if *name == item {
//                 for _ in 0..idx {
//                     p.send("\x1b[B")?;
//                     p.expect("❯")?;
//                 }
//                 p.send("\n")?;
//                 return Ok(());
//             }
//         }
//         anyhow::bail!(format!("Menu item not found: {}", item))
//     }
// }

// pub mod planung_partien {
//     use super::*;

//     pub fn check_planung_partien_menu(p: &mut PtySession) -> Result<()> {
//         // p.expect(Regex("Planning & Sessions"))?;
//         p.expect(Regex("❯.*Scheduled Game Nights"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Plan New Game Night"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Show My Shelf"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Record Session"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Edit Session"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Delete Session"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*Show Session History"))?;
//         p.send("\x1b[B")?;
//         p.expect(Regex("❯.*<< Back"))?;

//         // Reset cursor to top
//         p.send("\x1b[B")?;
//         Ok(())
//     }

//     pub fn navigate_to(p: &mut PtySession, item: &str) -> Result<()> {
//         let menu_items = [
//             "Scheduled Game Nights",
//             "Plan New Game Night",
//             "Show My Shelf",
//             "Record Session",
//             "Edit Session",
//             "Delete Session",
//             "Show Session History",
//             "<< Back",
//         ];

//         for (idx, name) in menu_items.iter().enumerate() {
//             if *name == item {
//                 for _ in 0..idx {
//                     p.send("\x1b[B")?;
//                     p.expect("❯")?;
//                 }
//                 p.send("\n")?;
//                 return Ok(());
//             }
//         }
//         anyhow::bail!(format!("Menu item not found: {}", item))
//     }
// }

// mod menu_tests {
//     use tokio::time::sleep;

//     use super::*;

//     #[tokio::test]
//     async fn check_main_menu_first() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         // Check and navigate each main menu item
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::check_spiele_regal_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "Capture New Game")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "<< Back")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_main_menu_second() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         main_menu::navigate_to(&mut p, "Players & Lending")?;
//         println!("1------------------------------------------------------");
//         spieler_verleih::check_spieler_verleih_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spieler_verleih::navigate_to(&mut p, "New Player")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spieler_verleih::navigate_to(&mut p, "<< Back")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     // #[tokio::test]
//     // async fn check_main_menu_third() -> Result<()> {
//     //     use std::io::Write;
//     //     let db_guard = TestDbGuard::new().await;
//     //     let mut p = spawn_cli(&db_guard.db_url)?;

//     //     // Login first
//     //     crate::login(&mut p, "maria@spiele.de")?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("------------------------------------------------------");

//     //     main_menu::navigate_to(&mut p, "Planning & Sessions")?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("2------------------------------------------------------");
//     //     planung_partien::check_planung_partien_menu(&mut p)?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("------------------------------------------------------");
//     //     // Guessed translation: "Partie erfassen" -> "Record Session"
//     //     planung_partien::navigate_to(&mut p, "Record Session")?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("------------------------------------------------------");
//     //     p.send(ControlCode::Escape)?;
//     //     std::thread::sleep(Duration::from_millis(50));
//     //     p.get_stream_mut().flush()?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("2------------------------------------------------------");
//     //     planung_partien::check_planung_partien_menu(&mut p)?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("------------------------------------------------------");
//     //     planung_partien::navigate_to(&mut p, "<< Back")?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("------------------------------------------------------");
//     //     main_menu::check_main_menu(&mut p)?;
//     //     sleep(Duration::from_millis(60)).await;
//     //     println!("------------------------------------------------------");

//     //     Ok(())
//     // }
// }

// mod function_tests {
//     use tokio::time::sleep;

//     use super::*;

//     #[tokio::test]
//     async fn check_neues_spiel_erstellen() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         // Check and navigate each main menu item
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::check_spiele_regal_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "Capture New Game")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("Uno\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Fetch data from BoardGameGeek\\?"))?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         for _ in 0..38 {
//             p.send("\t")?;
//         }
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Confirm Title"))?;  // TODO
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("E------------------------------------------------------");
//         p.expect(Regex("Release Year"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Publisher"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Category"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("B------------------------------------------------------");
//         p.expect(Regex("BGG Rating"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("A------------------------------------------------------");
//         p.expect(Regex("Author"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         sleep(Duration::from_millis(60)).await;
//         p.send("y")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("Testauthor\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("Weiteren------------------------------------------------------");
//         p.expect(Regex("Add another author manually"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("S------------------------------------------------------");
//         p.expect(Regex("Player Count"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("D------------------------------------------------------");
//         p.expect(Regex("Duration"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("Al------------------------------------------------------");
//         p.expect(Regex("Age Recommendation"))?;
//         p.send("\n")?;

//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "<< Back")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_neues_spiel_erstellen_ohne_bgg_ergebnis() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         // Check and navigate each main menu item
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::check_spiele_regal_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "Capture New Game")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("jkdlfsdfaweroiuwer\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Fetch data from BoardGameGeek\\?"))?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("No matches found on BGG. Using manual input."))?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Confirm Title"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("E------------------------------------------------------");
//         p.expect(Regex("Release Year"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Publisher"))?;
//         p.send("\n")?;

//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "<< Back")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_neues_spiel_erstellen_bgg_error() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli_offline(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         // Check and navigate each main menu item
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::check_spiele_regal_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "Capture New Game")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("Uno\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Fetch data from BoardGameGeek\\?"))?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect("Critical Error")?;
//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_spielkopie_anlegen() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         // Check and navigate each main menu item
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::check_spiele_regal_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "Add Game Copy")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         // for _ in 0..10 {
//             p.send("\t")?;
//         // }
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "<< Back")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_spiel_verleihen() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         main_menu::navigate_to(&mut p, "Players & Lending")?;
//         println!("1------------------------------------------------------");
//         spieler_verleih::check_spieler_verleih_menu(&mut p)?;
//         println!("------------------------------------------------------");
//         spieler_verleih::navigate_to(&mut p, "Lend Game")?;
//         println!("------------------------------------------------------");
//         p.send("\t")?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\t")?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spieler_verleih::navigate_to(&mut p, "<< Back")?;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_spieleliste_anzeigen() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli_offline(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         // Check and navigate each main menu item
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::check_spiele_regal_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spiele_regal::navigate_to(&mut p, "Show Game List")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_game_night_planning() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         main_menu::navigate_to(&mut p, "Planning & Sessions")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("2------------------------------------------------------");
//         planung_partien::check_planung_partien_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         planung_partien::navigate_to(&mut p, "Plan New Game Night")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         p.expect(Regex("Date of the event"))?;
//         p.send("\n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("2------------------------------------------------------");
//         p.expect(Regex("Participants"))?;
//         p.send(" \n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("2------------------------------------------------------");
//         p.expect(Regex("Games on offer"))?;
//         p.send(" \n")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("2------------------------------------------------------");
//         planung_partien::check_planung_partien_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         planung_partien::navigate_to(&mut p, "<< Back")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         main_menu::check_main_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         Ok(())
//     }

//     #[tokio::test]
//     async fn check_main_menu_second() -> Result<()> {
//         let db_guard = TestDbGuard::new().await;
//         let mut p = spawn_cli(&db_guard.db_url)?;

//         // Login first
//         crate::login(&mut p, "maria@spiele.de")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");

//         main_menu::navigate_to(&mut p, "Players & Lending")?;
//         sleep(Duration::from_millis(60)).await;
//         println!("1------------------------------------------------------");
//         spieler_verleih::check_spieler_verleih_menu(&mut p)?;
//         sleep(Duration::from_millis(60)).await;
//         println!("------------------------------------------------------");
//         spieler_verleih::navigate_to(&mut p, "Show Lending Overview")?;

//         Ok(())
//     }

//     /*mod debug_mode {
//         use super::*;

//         #[tokio::test]
//         async fn check_neues_spiel_erstellen() -> Result<()> {
//             let db_guard = TestDbGuard::new().await;
//             let mut p = spawn_cli_debug(&db_guard.db_url)?;

//             // Login first
//             crate::login(&mut p, "maria@spiele.de")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             // Check and navigate each main menu item
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::check_spiele_regal_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::navigate_to(&mut p, "Capture New Game")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("Uno\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect(Regex("Fetch data from BoardGameGeek\\?"))?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             for _ in 0..38 {
//                 p.send("\t")?;
//             }
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect(Regex("Confirm Title"))?;  // TODO
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("E------------------------------------------------------");
//             p.expect(Regex("Release Year"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect(Regex("Publisher"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect(Regex("Category"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("B------------------------------------------------------");
//             p.expect(Regex("BGG Rating"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("A------------------------------------------------------");
//             p.expect(Regex("Author"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             sleep(Duration::from_millis(60)).await;
//             p.send("y")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("Testauthor\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("Weiteren------------------------------------------------------");
//             p.expect(Regex("Add another author manually"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("S------------------------------------------------------");
//             p.expect(Regex("Player Count"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("D------------------------------------------------------");
//             p.expect(Regex("Duration"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("Al------------------------------------------------------");
//             p.expect(Regex("Age Recommendation"))?;
//             p.send("\n")?;

//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::navigate_to(&mut p, "<< Back")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             Ok(())
//         }

//         #[tokio::test]
//         async fn check_neues_spiel_erstellen_bgg_error() -> Result<()> {
//             let db_guard = TestDbGuard::new().await;
//             let mut p = spawn_cli_debug_offline(&db_guard.db_url)?;

//             // Login first
//             crate::login(&mut p, "maria@spiele.de")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             // Check and navigate each main menu item
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::check_spiele_regal_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::navigate_to(&mut p, "Capture New Game")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("Uno\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect(Regex("Fetch data from BoardGameGeek\\?"))?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect("Critical Error")?;
//             Ok(())
//         }

//         #[tokio::test]
//         async fn check_spielkopie_anlegen() -> Result<()> {
//             let db_guard = TestDbGuard::new().await;
//             let mut p = spawn_cli_debug(&db_guard.db_url)?;

//             // Login first
//             crate::login(&mut p, "maria@spiele.de")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             // Check and navigate each main menu item
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::check_spiele_regal_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::navigate_to(&mut p, "Add Game Copy")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             for _ in 0..10 {
//                 p.send("\t")?;
//             }
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::navigate_to(&mut p, "<< Back")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             Ok(())
//         }

//         #[tokio::test]
//         async fn check_spiel_verleihen() -> Result<()> {
//             let db_guard = TestDbGuard::new().await;
//             let mut p = spawn_cli_debug(&db_guard.db_url)?;

//             // Login first
//             crate::login(&mut p, "maria@spiele.de")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             main_menu::navigate_to(&mut p, "Players & Lending")?;
//             println!("1------------------------------------------------------");
//             spieler_verleih::check_spieler_verleih_menu(&mut p)?;
//             println!("------------------------------------------------------");
//             spieler_verleih::navigate_to(&mut p, "Lend Game")?;
//             println!("------------------------------------------------------");
//             p.send("\t")?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("\t")?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spieler_verleih::navigate_to(&mut p, "<< Back")?;
//             println!("------------------------------------------------------");
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             Ok(())
//         }

//         #[tokio::test]
//         async fn check_spieleliste_anzeigen() -> Result<()> {
//             let db_guard = TestDbGuard::new().await;
//             let mut p = spawn_cli_debug_offline(&db_guard.db_url)?;

//             // Login first
//             crate::login(&mut p, "maria@spiele.de")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             // Check and navigate each main menu item
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::navigate_to(&mut p, "Manage Games & Shelf")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::check_spiele_regal_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             spiele_regal::navigate_to(&mut p, "Show Game List")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             Ok(())
//         }

//         #[tokio::test]
//         async fn check_game_night_planning() -> Result<()> {
//             let db_guard = TestDbGuard::new().await;
//             let mut p = spawn_cli_debug(&db_guard.db_url)?;

//             // Login first
//             crate::login(&mut p, "maria@spiele.de")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             main_menu::navigate_to(&mut p, "Planning & Sessions")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("2------------------------------------------------------");
//             planung_partien::check_planung_partien_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             planung_partien::navigate_to(&mut p, "Plan New Game Night")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             p.expect(Regex("Date of the event"))?;
//             p.send("\n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("2------------------------------------------------------");
//             p.expect(Regex("Participants"))?;
//             p.send(" \n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("2------------------------------------------------------");
//             p.expect(Regex("Games on offer"))?;
//             p.send(" \n")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("2------------------------------------------------------");
//             planung_partien::check_planung_partien_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             planung_partien::navigate_to(&mut p, "<< Back")?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");
//             main_menu::check_main_menu(&mut p)?;
//             sleep(Duration::from_millis(60)).await;
//             println!("------------------------------------------------------");

//             Ok(())
//         }
//     }
// */
// }
