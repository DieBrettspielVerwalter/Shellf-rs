Shellf 📖🎲

Shellf ist ein digitaler Manager für deine Brettspielsammlung. Es ermöglicht die Verwaltung von Spieldaten (inklusive BoardGameGeek-Anbindung), das Tracking physischer Spielkopien, die Organisation von Spieleabenden und das Protokollieren von Spielergebnissen.

Das Projekt folgt strikt den Prinzipien der Clean Architecture (Hexagonale Architektur), um eine hohe Testbarkeit und Unabhängigkeit von externen Technologien zu gewährleisten.
🏗️ Architektur & Schichten

Das Projekt ist in vier Schichten unterteilt:

    Domain Layer (src/domain): Enthält die Business-Logik, Entitäten (Game, Player, GameCopy etc.) und Repository-Traits. Diese Schicht ist komplett unabhängig von anderen Modulen.

    Application Layer (src/application): Beinhaltet die Use-Cases (Interaktoren), die den Datenfluss zwischen der Domain und der Außenwelt orchestrieren (z.B. CaptureGameUseCase).

    Infrastructure Layer (src/infrastructure): Die technische Implementierung. Hier liegen die SQLx-Repositories für MariaDB und der API-Client für BoardGameGeek.

    Presentation Layer (src/presentation): Die Schnittstelle zum Nutzer. In diesem Fall ein interaktives Command Line Interface (CLI) basierend auf dialoguer.

✨ Features

    Spiel-Katalog: Suche und Import von Metadaten direkt über die BoardGameGeek XML API2.

    Regal-Verwaltung: Erfasse deine physischen Spielkopien und weise sie Besitzern zu.

    Spieler-Management: Registriere Mitspieler inklusive automatischer Erstellung von MariaDB-Datenbank-Benutzern für feingranulare Berechtigungen.

    Verleih-System: Tracke, wer welches Spiel aus deinem Regal ausgeliehen hat und behalte Rückgabefristen im Blick.

    Partien-Log: Dokumentiere Spielergebnisse, Teilnehmer und Gewinner deiner Spielrunden.

    Event-Planung: Plane zukünftige Spieleabende mit Teilnehmerlisten und Spielevorschlägen.

🛠️ Tech Stack

    Sprache: Rust 🦀

    Datenbank: MariaDB / MySQL

    SQL-Framework: sqlx (mit Compile-time Query Check)

    CLI-Toolkit: dialoguer

    API: reqwest & quick-xml für BoardGameGeek Integration

    Asynchronität: tokio

🚀 Setup
Voraussetzungen

    Eine laufende MariaDB-Instanz.

    Eine .env Datei im Projektwurzelverzeichnis (siehe unten).

    (Optional) Ein BGG API-Key, falls die Authentifizierung für BoardGameGeek genutzt werden soll.

1. Umgebungsvariablen konfigurieren

Erstelle eine .env Datei:
Code-Snippet

# Die Standard-DB für den initialen Verbindungsaufbau (wird zur Laufzeit durch User-Login ergänzt)
DATABASE_URL=mysql://localhost/SpieleDB
BGG_API_KEY=dein_api_key_hier

2. Datenbank-Schema

Initialisiere deine Datenbank mit der Shellf.sql (oder dem entsprechenden Schema-File), um Tabellen, Stored Procedures (z.B. Spieler_EigeneKopie_Einfuegen) und die benötigten Rollen (spieler_role, dba_role) anzulegen.
3. Starten
   Bash

# Normaler Start
cargo run

# Debug-Modus für detaillierte SQL-Verbindungsinfos
cargo run -- --debug

🧪 Testing

Das Projekt nutzt isolierte Integrationstests mit einem automatisierten Datenbank-Lifecycle (TestDbGuard). Jeder Testlauf erstellt eine temporäre, eindeutige Datenbank.
Bash

cargo test

📂 Projektstruktur (Auszug)
Plaintext

src/  
├── domain/            # Entitäten & Repository-Definitionen  
├── application/       # Use Cases (Business Orchestration)  
├── infrastructure/    # SQL-Implementierungen & BGG-Client  
├── presentation/      # CLI Menüs & Dialoge  
├── app_context.rs     # Dependency Injection Container  
└── main.rs            # Entry Point & DB-Login-Loop  

Dieses Projekt wurde mit Fokus auf Clean Code und strikte Trennung von Belangen entwickelt.
