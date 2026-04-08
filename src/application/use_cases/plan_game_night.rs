use crate::domain::*;
use anyhow::Result;
use chrono::NaiveDate;
use mantra_rust_macros::req;
use std::sync::Arc;

/// A use case responsible for planning and persisting a game night event.
///
/// This struct orchestrates the creation of a `GameNight` domain entity,
/// including input transformation and validation, and delegates persistence
/// to the repository layer.
pub struct PlanGameNightUseCase {
    /// The repository used to persist and manage game night data.
    pub game_night_repo: Arc<dyn GameNightRepository>,
}

impl PlanGameNightUseCase {
    /// Executes the business logic to plan a new game night.
    ///
    /// The process involves:
    /// - Converting `suggested_copies` from `Vec<String>` to `Vec<i32>`.
    /// - Constructing a `GameNight` domain entity.
    /// - Persisting the entity using the repository.
    ///
    /// # Arguments
    ///
    /// * `date` - The scheduled date of the game night.
    /// * `notes` - Optional notes or additional information (e.g., "Bring snacks").
    /// * `participants` - A list of participant identifiers (e.g., email addresses).
    /// * `suggested_copies` - A list of game copy IDs as strings (parsed internally).
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if:
    /// - Any entry in `suggested_copies` cannot be parsed into a valid `i32`.
    /// - The repository fails to persist the `GameNight`
    ///   (e.g., database failure or connectivity issues).
    #[req("UC.6")]
    pub async fn execute(
        &self,
        date: NaiveDate,
        notes: Option<String>,
        participants: Vec<String>,
        suggested_copies: Vec<String>,
    ) -> Result<()> {
        // Wandelt Vec<String> in Vec<i32> um und bricht beim ersten Fehler ab
        let suggested_copies: Vec<i32> = suggested_copies
            .iter()
            .map(|c| {
                c.parse::<i32>()
                    .map_err(|_| anyhow::anyhow!("Invalid ID: {}", c))
            })
            .collect::<Result<Vec<_>>>()?; // Das Result wird hier mit '?' direkt entpackt

        let night = GameNight {
            id: 0, // Wird von der Datenbank (Auto-Increment) vergeben
            date,
            notes,
            participants,
            suggested_copies,
        };

        // Hier nutzen wir .await für die Datenbank
        self.game_night_repo.save(night).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::mockups::*;
    use anyhow::anyhow;
    use chrono::NaiveDate;
    use std::sync::Arc;
    use tokio;

    /// Verifies successful planning of a game night.
    ///
    /// This test ensures that:
    /// - A `GameNight` entity is constructed with correctly mapped fields.
    /// - The repository `save` method is called exactly once.
    /// - All collections (participants and suggested copies) are passed unchanged.
    /// - The use case returns `Ok(())` on success.
    ///
    /// This validates correct orchestration and boundary integrity.
    #[tokio::test]
    async fn plan_game_night_use_case_saves_game_night_successfully() {
        // --- 1. Prepare test data ---
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        let participants = vec!["maria@spiele.de".into(), "maria@spiele.de".into()];
        // let suggested_copies = vec!["Elden Ring".to_string(), "Cyberpunk 2077".to_string()];
        let suggested_copies = vec![1, 2];

        // --- 2. Prepare mock ---
        let mut repo = MockGameNightRepository::new();
        repo.expect_save()
            .withf({
                let participants = participants.clone();
                let suggested_copies = suggested_copies.clone();
                move |night| {
                    night.id == 0
                        && night.date == date
                        && night.notes.as_deref() == Some("Bring snacks")
                        && night.participants == participants
                        && night.suggested_copies == suggested_copies
                }
            })
            .once()
            .returning(|_| Ok(0));

        let use_case = PlanGameNightUseCase {
            game_night_repo: Arc::new(repo),
        };

        // --- 3. Execute ---
        let result = use_case
            .execute(
                date,
                Some("Bring snacks".to_string()),
                participants,
                suggested_copies
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>(),
            )
            .await;

        // --- 4. Assert ---
        assert!(result.is_ok());
    }

    /// Verifies that persistence failure is properly propagated.
    ///
    /// This test ensures that:
    /// - The `save` method is invoked exactly once.
    /// - If the repository returns an error, the use case propagates it.
    /// - No silent success occurs.
    ///
    /// This protects error semantics at the application boundary.
    #[tokio::test]
    async fn plan_game_night_use_case_propagates_save_failure() {
        // --- 1. Prepare test data ---
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

        // --- 2. Prepare mock ---
        let mut repo = MockGameNightRepository::new();
        repo.expect_save()
            .once()
            .returning(|_| Err(anyhow!("Database unavailable")));

        let use_case = PlanGameNightUseCase {
            game_night_repo: Arc::new(repo),
        };

        // --- 3. Execute ---
        let result = use_case.execute(date, None, vec![], vec![]).await;

        // --- 4. Assert ---
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Database unavailable"));
    }

    /// Verifies that empty collections are handled safely.
    ///
    /// This test ensures that:
    /// - The use case allows empty participants and suggested copies.
    /// - The repository is still called exactly once.
    /// - No implicit validation or mutation occurs.
    ///
    /// This guarantees predictable behavior when optional lists are empty.
    #[tokio::test]
    async fn plan_game_night_use_case_allows_empty_participants_and_copies() {
        // --- 1. Prepare test data ---
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

        // --- 2. Prepare mock ---
        let mut repo = MockGameNightRepository::new();
        repo.expect_save()
            .withf(|night| night.participants.is_empty() && night.suggested_copies.is_empty())
            .once()
            .returning(|_| Ok(1));

        let use_case = PlanGameNightUseCase {
            game_night_repo: Arc::new(repo),
        };

        // --- 3. Execute ---
        let result = use_case.execute(date, None, vec![], vec![]).await;

        // --- 4. Assert ---
        assert!(result.is_ok());
    }
}
