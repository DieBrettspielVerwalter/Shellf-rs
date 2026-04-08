//! Infrastructure layer providing concrete implementations for external systems.
//!
//! This module houses the "Adapters" in Hexagonal Architecture, bridging the
//! gap between the abstract Domain traits and real-world technologies like
//! MariaDB (SQL) and the BoardGameGeek API.

/// Data persistence implementations (Repositories).
///
/// Contains the SQL-based logic for storing games, players, and sessions
/// using `sqlx`.
pub mod repositories;

/// External API clients and data providers.
///
/// Contains the `BggClient` for fetching and mapping board game
/// metadata from the BoardGameGeek XML API2.
pub mod api;
