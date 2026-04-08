use crate::domain::*;
use anyhow::Result;
use mantra_rust_macros::req;
use std::sync::Arc;

/// A use case responsible for capturing and persisting a new board game.
///
/// This struct orchestrates the creation of a `Game` domain entity and
/// ensures its persistence through the provided repository.
pub struct CaptureGameUseCase {
    /// The repository used to persist game data.
    pub game_repo: Arc<dyn GameRepository>,
}

impl CaptureGameUseCase {
    /// Executes the business logic to capture a game with the provided metadata.
    ///
    /// This method constructs a `Game` entity from the input arguments and
    /// attempts to save it to the underlying storage.
    ///
    /// # Arguments
    ///
    /// * `name` - The title of the game.
    /// * `year` - The release year of the game.
    /// * `players` - A description of the player count (e.g., "1-4").
    /// * `duration` - The average playtime in minutes.
    /// * `age` - The minimum recommended age for players.
    /// * `category` - The genre or category of the game.
    /// * `publisher_name` - The name of the publisher. If empty, the publisher is set to `None`.
    /// * `author_names` - A list of authors who designed the game.
    /// * `rating` - The numerical rating assigned to the game.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result::Err` if the persistence layer (`game_repo`)
    /// fails to save the game.
    #[allow(clippy::too_many_arguments)]
    #[req("UC.1")]
    pub async fn execute(
        &self,
        name: String,
        year: i32,
        players: String,
        duration: i32,
        age: i32,
        category: String,
        publisher_name: String,
        author_names: Vec<String>,
        rating: f32,
    ) -> Result<()> {
        let game = Game {
            name,
            year,
            players,
            duration,
            age,
            category,
            publisher: if publisher_name.is_empty() {
                None
            } else {
                Some(publisher_name)
            },
            authors: author_names,
            rating,
        };

        self.game_repo.save(game).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::game_repository::MockGameRepository;
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn capture_game_use_case_executes_and_saves_game() {
        let mut game_repo = MockGameRepository::new();
        game_repo
            .expect_save()
            .withf(|game| {
                game.name == "Elden Ring"
                    && game.year == 2022
                    && game.players == "1-4"
                    && game.duration == 120
                    && game.age == 16
                    && game.category == "Action RPG"
                    && game.publisher.as_deref() == Some("Epic Publisher")
                    && game.authors == vec!["jane.doe@bgg-import.de"]
                    && game.rating == 4.5
            })
            .once()
            .returning(|_| Ok(()));

        let use_case = CaptureGameUseCase {
            game_repo: Arc::new(game_repo),
        };

        let result = use_case
            .execute(
                "Elden Ring".to_string(),
                2022,
                "1-4".to_string(),
                120,
                16,
                "Action RPG".to_string(),
                "Epic Publisher".to_string(),
                vec!["jane.doe@bgg-import.de".to_string()],
                4.5,
            )
            .await;

        assert!(result.is_ok());
    }
}
