/// The core domain layer of the application.
///
/// This module contains the fundamental business logic, including:
/// - **Entities**: Pure data structures and their associated logic.
/// - **Repositories**: Traits defining the interface for data persistence.
/// - **Errors**: Domain-specific error types for business rule violations.
///
/// The domain layer is strictly decoupled from infrastructure and application
/// orchestration, serving as the "Source of Truth" for the system's rules.
pub mod entities;

/// Abstract interfaces for data storage and retrieval.
pub mod repositories;

/// Domain-specific error types and result aliases.
pub mod error;

pub use entities::*;
pub use repositories::*;
