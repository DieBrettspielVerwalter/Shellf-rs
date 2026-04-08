pub mod game_copy_repository;
pub mod game_night_repository;
/// Abstract data access interfaces for the domain layer.
///
/// This module defines the repository traits required to persist and retrieve
/// domain entities. By using these abstractions, the application layer remains
/// agnostic of the underlying storage technology.
pub mod game_repository;
pub mod game_session_repository;
pub mod player_repository;

pub use game_copy_repository::GameCopyRepository;
pub use game_night_repository::GameNightRepository;
pub use game_repository::GameRepository;
pub use game_session_repository::GameSessionRepository;
pub use player_repository::PlayerRepository;

/// Automated mock implementations for testing purposes.
///
/// These mocks are generated using `mockall` and allow use cases to be tested
/// in isolation without requiring a running database.
#[cfg(test)]
pub mod mockups {
    use super::*;
    pub use game_copy_repository::MockGameCopyRepository;
    pub use game_night_repository::MockGameNightRepository;
    pub use game_repository::MockGameRepository;
    pub use game_session_repository::MockGameSessionRepository;
    pub use player_repository::MockPlayerRepository;
}

/// Generic contract tests for ensuring repository consistency.
///
/// These tests define the expected behavior of any `Repository` implementation.
/// They can be run against both the real database adapters and in-memory
/// implementations to ensure they adhere to the same business rules.
#[cfg(test)]
pub mod tests {
    use super::*;
    pub use game_copy_repository::tests::GameCopyRepositoryContractTests;
    pub use game_night_repository::tests::GameNightRepositoryContractTests;
    pub use game_repository::tests::GameRepositoryContractTests;
    pub use game_session_repository::tests::GameSessionRepositoryContractTests;
    pub use player_repository::tests::PlayerRepositoryContractTests;
}
