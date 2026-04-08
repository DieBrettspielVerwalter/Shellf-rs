use thiserror::Error;

/// Specific error types representing business rule violations in the domain layer.
///
/// This enumeration provides a structured way to handle validation failures
/// and logic errors that occur during entity construction or domain service execution.
#[derive(Error, Debug)]
pub enum DomainError {
    /// Indicates that a provided email string does not meet the required format.
    ///
    /// # Arguments
    /// * `0` - The invalid email string that caused the error.
    #[error("Ungültige E-Mail Adresse: {0}")]
    InvalidEmail(String),

    /// Indicates that a player's nickname was either empty or consisted only of whitespace.
    #[error("Nickname darf nicht leer sein")]
    EmptyNickname,

    /// Indicates that mandatory name fields (first or last name) were missing or invalid.
    #[error("Vor- und Nachname müssen ausgefüllt sein")]
    InvalidName,
}

/// A specialized `Result` type for domain-layer operations.
///
/// This alias simplifies function signatures by defaulting the error type
/// to [`DomainError`].
pub type DomainResult<T> = Result<T, DomainError>;
