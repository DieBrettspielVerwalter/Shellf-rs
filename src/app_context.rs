use sqlx::MySqlPool;
use std::sync::Arc;

// Infrastructure - Repositories
use crate::infrastructure::repositories::sql::{
    game_copy_repository::SqlGameCopyRepository, game_night_repository::SqlGameNightRepository,
    game_repository::SqlGameRepository, game_session_repository::SqlGameSessionRepository,
    player_repository::SqlPlayerRepository,
};

// Application - Use Cases
use crate::application::use_cases::{
    capture_game::CaptureGameUseCase,
    create_player::{CreatePlayerUseCase, EditPlayerUseCase},
    lend_game::LendGameUseCase,
    manage_game, // Contains DeleteGameUseCase and EditGameUseCase
    manage_game_copy::{CreateGameCopyUseCase, DeleteGameCopyUseCase, EditGameCopyUseCase},
    plan_game_night::PlanGameNightUseCase,
    record_game_session::RecordGameSessionUseCase,
    return_game::ReturnGameUseCase,
};

// Infrastructure - API Clients
use crate::infrastructure::api::bgg_client::BggClient;

/// The central Dependency Injection (DI) container for the application.
///
/// `AppContext` manages the lifecycle of all long-lived components, including
/// database repositories, external API clients, and application use cases.
/// It is initialized once during startup and shared across the presentation layer.
pub struct AppContext {
    // Repositories wrapped in Arc to allow shared ownership across multiple Use Cases.
    pub game_repo: Arc<SqlGameRepository>,
    pub player_repo: Arc<SqlPlayerRepository>,
    pub copy_repo: Arc<SqlGameCopyRepository>,
    pub night_repo: Arc<SqlGameNightRepository>,
    pub session_repo: Arc<SqlGameSessionRepository>,

    /// Client for the BoardGameGeek XML API2.
    pub bgg_client: BggClient,

    // Orchestrated Use Cases
    pub capture_uc: CaptureGameUseCase,
    pub create_player_uc: CreatePlayerUseCase,
    pub edit_player_uc: EditPlayerUseCase,
    pub create_copy_uc: CreateGameCopyUseCase,
    pub edit_copy_uc: EditGameCopyUseCase,
    pub delete_copy_uc: DeleteGameCopyUseCase,
    pub lend_game_uc: LendGameUseCase,
    pub return_game_uc: ReturnGameUseCase,
    pub plan_night_uc: PlanGameNightUseCase,
    pub record_session_uc: RecordGameSessionUseCase,
    pub delete_game_uc: manage_game::DeleteGameUseCase,
    pub edit_game_uc: manage_game::EditGameUseCase,
}

impl AppContext {
    /// Initializes all components of the application.
    ///
    /// This constructor performs the "wiring" of the application:
    /// 1. Instantiates concrete SQL repositories using the provided pool.
    /// 2. Sets up the BGG API client.
    /// 3. Injects the repositories into their respective Use Case interactors.
    ///
    /// # Arguments
    ///
    /// * `pool` - The MariaDB connection pool (`sqlx::MySqlPool`).
    /// * `bgg_api_key` - The API key/token for BoardGameGeek access.
    pub fn new(pool: MySqlPool, bgg_api_key: &str) -> Self {
        // 1. Initialize Repositories
        let game_repo = Arc::new(SqlGameRepository::new(pool.clone()));
        let player_repo = Arc::new(SqlPlayerRepository::new(pool.clone()));
        let copy_repo = Arc::new(SqlGameCopyRepository::new(pool.clone()));
        let night_repo = Arc::new(SqlGameNightRepository::new(pool.clone()));
        let session_repo = Arc::new(SqlGameSessionRepository::new(pool.clone()));

        // 2. Initialize Clients
        let bgg_client = BggClient::new(bgg_api_key);

        // 3. Assemble Use Cases with their required dependencies
        Self {
            capture_uc: CaptureGameUseCase {
                game_repo: game_repo.clone(),
            },
            create_player_uc: CreatePlayerUseCase {
                player_repo: player_repo.clone(),
            },
            edit_player_uc: EditPlayerUseCase {
                player_repo: player_repo.clone(),
            },
            create_copy_uc: CreateGameCopyUseCase {
                copy_repo: copy_repo.clone(),
            },
            edit_copy_uc: EditGameCopyUseCase {
                copy_repo: copy_repo.clone(),
            },
            delete_copy_uc: DeleteGameCopyUseCase {
                copy_repo: copy_repo.clone(),
            },
            lend_game_uc: LendGameUseCase {
                copy_repo: copy_repo.clone(),
                _player_repo: player_repo.clone(),
            },
            return_game_uc: ReturnGameUseCase {
                copy_repo: copy_repo.clone(),
            },
            plan_night_uc: PlanGameNightUseCase {
                game_night_repo: night_repo.clone(),
            },
            record_session_uc: RecordGameSessionUseCase {
                session_repo: session_repo.clone(),
            },
            delete_game_uc: manage_game::DeleteGameUseCase {
                game_repo: game_repo.clone(),
            },
            edit_game_uc: manage_game::EditGameUseCase {
                game_repo: game_repo.clone(),
            },

            // Expose repositories for direct access (e.g., listing all entities)
            game_repo,
            player_repo,
            copy_repo,
            night_repo,
            session_repo,
            bgg_client,
        }
    }
}
