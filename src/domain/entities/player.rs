use crate::domain::error::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// A domain entity representing a player within the system.
///
/// This struct models the core identity and profile data of a player.
/// It includes validation logic to ensure domain invariants are upheld
/// when creating new instances.
///
/// Note: Fields remain public to support ORM mapping (e.g., SQLx),
/// but construction should go through the `new` method to enforce validation.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Player {
    /// The unique email address used to identify the player.
    pub email: String, // Felder bleiben pub für SQLx Mapper,

    /// The player's display name or nickname.
    pub nickname: String, // aber wir nutzen die Logik im Use Case.

    /// The player's legal first name.
    pub first_name: String,

    /// The player's legal last name.
    pub last_name: String,

    /// Optional additional information or profile details.
    pub details: Option<String>,
}

impl Player {
    /// Creates a new `Player` instance with validated input.
    ///
    /// This constructor enforces domain rules to ensure that:
    /// - The email has a valid format.
    /// - The nickname is not empty.
    /// - First and last names are not empty.
    ///
    /// # Arguments
    ///
    /// * `email` - The player's email address (must contain '@').
    /// * `nickname` - The player's display name (must not be empty).
    /// * `first_name` - The player's first name (must not be empty).
    /// * `last_name` - The player's last name (must not be empty).
    /// * `details` - Optional additional profile information.
    ///
    /// # Errors
    ///
    /// Returns a `DomainError` if:
    /// - The email is invalid.
    /// - The nickname is empty.
    /// - The first or last name is invalid.
    pub fn new(
        email: String,
        nickname: String,
        first_name: String,
        last_name: String,
        details: Option<String>,
    ) -> DomainResult<Self> {
        // VALIDIERUNG
        if !email.contains('@') {
            return Err(DomainError::InvalidEmail(email));
        }
        if nickname.trim().is_empty() {
            return Err(DomainError::EmptyNickname);
        }
        if first_name.trim().is_empty() || last_name.trim().is_empty() {
            return Err(DomainError::InvalidName);
        }

        Ok(Self {
            email,
            nickname,
            first_name,
            last_name,
            details,
        })
    }
}
