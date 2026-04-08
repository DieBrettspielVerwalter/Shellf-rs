/// The Command Line Interface (CLI) presentation layer.
///
/// This module organizes the user-facing interactions into subcommands
/// for each domain entity and coordinates navigation through a central router.
pub mod common;

/// CLI commands for managing board game metadata.
pub mod game;

/// CLI commands for physical game copies and lending operations.
pub mod game_copy;

/// CLI commands for player registration and profile management.
pub mod player;

/// CLI commands for scheduling and managing game night events.
pub mod game_night;

/// CLI commands for recording and reviewing individual game sessions.
pub mod game_session;

/// The central execution engine that maps input strings to specific logic.
pub mod router;

/// User assistance and documentation for the CLI.
pub mod help;

// Re-exports to provide a flat API for the main execution loop.
pub use common::*;
pub use game::*;
pub use game_copy::*;
pub use game_night::*;
pub use game_session::*;
pub use help::*;
pub use player::*;
