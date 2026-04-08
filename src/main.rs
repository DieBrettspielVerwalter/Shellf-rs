//! # Shellf 📖🎲
//!
//! `Shellf` ist ein digitaler Manager für deine Brettspielsammlung. Die Anwendung ermöglicht:
//! - Verwaltung von Spieldaten (inklusive BoardGameGeek-Anbindung)  
//! - Tracking physischer Spielkopien  
//! - Organisation von Spieleabenden  
//! - Protokollieren von Spielergebnissen  
//!
//! ## Architektur & Schichten 🏗️
//!
//! Das Projekt folgt strikt den Prinzipien der Clean Architecture (Hexagonale Architektur), um eine hohe Testbarkeit
//! und Unabhängigkeit von externen Technologien zu gewährleisten. Die Architektur ist in vier Schichten unterteilt:
//!
//! ### 1. Domain Layer (`src/domain`)
//! - Enthält die Business-Logik, Entitäten (`Game`, `Player`, `GameCopy` etc.) und Repository-Traits  
//! - Komplett unabhängig von anderen Modulen  
//!
//! ### 2. Application Layer (`src/application`)
//! - Beinhaltet die Use-Cases (Interaktoren)  
//! - Orchestriert den Datenfluss zwischen Domain und Außenwelt  
//! - Beispiel: `CaptureGameUseCase`  
//!
//! ### 3. Infrastructure Layer (`src/infrastructure`)
//! - Technische Implementierungen: SQLx-Repositories für MariaDB und API-Client für BoardGameGeek  
//! - Bindeglied zu externen Systemen und Datenbanken  
//!
//! ### 4. Presentation Layer (`src/presentation`)
//! - Schnittstelle zum Nutzer  
//! - Interaktives Command Line Interface (CLI) basierend auf `dialoguer`  
//!
//! ## Features ✨
//!
//! - **Spiel-Katalog**: Suche und Import von Metadaten über die BoardGameGeek XML API2  
//! - **Regal-Verwaltung**: Erfasse physische Spielkopien und weise sie Besitzern zu  
//! - **Spieler-Management**: Registriere Mitspieler inkl. MariaDB-Benutzer für granular Berechtigungen  
//! - **Verleih-System**: Tracke Ausleihen und Rückgabefristen  
//! - **Partien-Log**: Dokumentiere Spielergebnisse, Teilnehmer und Gewinner  
//! - **Event-Planung**: Plane Spieleabende mit Teilnehmerlisten und Spielevorschlägen  
//!
//! ## Tech Stack 🛠️
//!
//! - Sprache: Rust 🦀  
//! - Datenbank: MariaDB / MySQL  
//! - SQL-Framework: `sqlx` (mit Compile-time Query Check)  
//! - CLI-Toolkit: `dialoguer`  
//! - API: `reqwest` & `quick-xml` für BoardGameGeek-Integration  
//! - Asynchronität: `tokio`  
//!
//! ## Setup 🚀
//!
//! ### Voraussetzungen
//! - Laufende MariaDB-Instanz  
//! - `.env` Datei im Projektwurzelverzeichnis  
//! - Optional: BGG API-Key für BoardGameGeek-Integration  
//!
//! ### 1. Umgebungsvariablen
//! Erstelle eine `.env` Datei:
//! ```text
//! DATABASE_URL=mysql://localhost/SpieleDB
//! BGG_API_KEY=dein_api_key_hier
//! ```
//!
//! ### 2. Datenbank-Schema
//! Initialisiere die Datenbank mit `Shellf.sql` oder einem entsprechenden Schema-File.  
//! Dies erstellt Tabellen, Stored Procedures (z.B. `Spieler_EigeneKopie_Einfuegen`) und Rollen (`spieler_role`, `dba_role`).  
//!
//! ### 3. Starten
//! ```bash
//! # Normaler Start
//! cargo run
//!
//! # Debug-Modus (SQL-Verbindungsinfos)
//! cargo run -- --debug
//! ```
//!
//! ## Testing 🧪
//!
//! Das Projekt nutzt isolierte Integrationstests mit automatisiertem Datenbank-Lifecycle (`TestDbGuard`).  
//! Jeder Testlauf erstellt eine temporäre, eindeutige Datenbank:
//! ```bash
//! cargo test
//! ```
//!
//! ## Projektstruktur 📂
//!
//! ```text
//! src/
//! ├── domain/            # Entitäten & Repository-Definitionen
//! ├── application/       # Use Cases (Business Orchestration)
//! ├── infrastructure/    # SQL-Implementierungen & BGG-Client
//! ├── presentation/      # CLI Menüs & Dialoge
//! ├── app_context.rs     # Dependency Injection Container
//! └── main.rs            # Entry Point & DB-Login-Loop
//! ```
//!
//! Dieses Projekt legt großen Wert auf Clean Code und strikte Trennung von Belangen, um Wartbarkeit
//! und Erweiterbarkeit sicherzustellen.

mod app_context;
mod application;
mod domain;
mod infrastructure;
mod presentation;

use crate::app_context::AppContext;
use crate::presentation::cli;
use anyhow::Result;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::env;

/// The entry point of the Shellf application.
///
/// This function handles the application bootstrapping process:
/// 1. **Argument Parsing**: Checks for `--debug` or `--help` flags.
/// 2. **Environment Setup**: Loads configuration from `.env`.
/// 3. **Authentication**: Runs a login loop to establish a secure MariaDB connection.
/// 4. **Dependency Injection**: Initializes the `AppContext` with the established pool.
/// 5. **Execution**: Starts the central CLI navigation loop.
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Collect and parse CLI arguments
    let args: Vec<String> = env::args().collect();
    let is_debug = args.contains(&String::from("--debug")) || args.contains(&String::from("-d"));

    // 2. Early exit for help menu
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        cli::display_help_menu();
        return Ok(());
    }

    // 2. Early exit for version
    if args.contains(&String::from("--version")) || args.contains(&String::from("-v")) {
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // 3. Load environment variables
    dotenvy::dotenv().ok();

    // 4. MariaDB Authentication Loop
    // We keep asking for credentials until a successful connection to the
    // local Unix socket is established.
    let pool = loop {
        let (user, pass): (String, String) = cli::run_login_dialog()?;

        // Path to the MariaDB socket (configured via Nix/Flake)
        let socket_path = std::env::var("MYSQL_UNIX_PORT")
            .expect("MYSQL_UNIX_PORT which should point to a MARIADB socket file is not set!");

        if is_debug {
            print_sql_debug_info(&user, &socket_path, "SpieleDB");
        }

        let mut opts = MySqlConnectOptions::new()
            .socket(socket_path)
            .username(&user)
            .database("SpieleDB");

        if !pass.is_empty() {
            opts = opts.password(&pass);
        }

        println!("Connecting as {}...", user);

        match MySqlPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect_with(opts)
            .await
        {
            Ok(p) => break p,
            Err(e) => {
                eprintln!("\n❌ Login failed: {}", e);
                if is_debug {
                    eprintln!("[DEBUG] SQL Error Details: {:?}", e);
                }
                eprintln!("Please try again.\n");
            }
        }
    };

    println!("✅ Login successful!");

    // 5. Initialize Context and Start Application
    let api_key = env::var("BGG_API_KEY").unwrap_or_else(|_| {
        if is_debug {
            println!("[DEBUG] BGG_API_KEY missing in .env");
        }
        "".to_string()
    });

    let ctx = AppContext::new(pool, &api_key);

    println!("Launching Shellf Interface...");
    if let Err(e) = crate::presentation::cli::router::run_main_loop(ctx).await {
        eprintln!("\n💥 Critical Error: {}", e);
    }

    Ok(())
}

/// Utility for printing SQL connection parameters during debug mode.
///
/// # Arguments
/// * `user` - The database username.
/// * `socket` - Path to the Unix socket file.
/// * `db` - Name of the target database.
fn print_sql_debug_info(user: &str, socket: &str, db: &str) {
    println!("\n\x1b[1;33m[DEBUG] SQL Connection Parameters:\x1b[0m");
    println!("  > User:     {}", user);
    println!("  > Socket:   {}", socket);
    println!("  > Database: {}", db);
    println!("  > Protocol: Unix Socket (Local)\n");
}
