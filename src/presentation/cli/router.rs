use crate::app_context::AppContext;
use crate::presentation::cli;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};
use mantra_rust_macros::reqcov;

use crate::domain::repositories::game_copy_repository::GameCopyRepository;
use crate::domain::repositories::game_night_repository::GameNightRepository;
use crate::domain::repositories::game_repository::GameRepository;
use crate::domain::repositories::game_session_repository::GameSessionRepository;
use crate::domain::repositories::player_repository::PlayerRepository;

/// Central execution loop for the CLI application.
///
/// This function serves as the main entry point for user interaction,
/// dispatching requests to specialized sub-menus and maintaining the
/// application's lifecycle until the user chooses to exit.
///
/// # Arguments
/// * `ctx` - The initialized [`AppContext`] containing all repositories and use cases.
pub async fn run_main_loop(ctx: AppContext) -> Result<()> {
    let theme = ColorfulTheme::default();
    loop {
        let main_menu = vec![
            "Manage Games & Shelf",
            "Players & Lending",
            "Planning & Sessions",
            "Display Help",
            "Exit",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("Main Menu")
            .items(&main_menu)
            .default(0)
            .interact_opt()?;

        match selection {
            Some(0) => run_game_menu(&ctx).await?,
            Some(1) => run_player_menu(&ctx).await?,
            Some(2) => run_planning_menu(&ctx).await?,
            Some(3) => {
                cli::display_help_menu();
            }
            None | Some(4) => break,
            _ => {}
        }
    }
    Ok(())
}

/// Sub-menu for managing master data (Games) and physical inventory (Copies).
///
/// Handles the orchestration of game metadata capture (via BGG),
/// shelf management, and CRUD operations for game entities.
async fn run_game_menu(ctx: &AppContext) -> Result<()> {
    let theme = ColorfulTheme::default();
    loop {
        let sub = vec![
            "Capture New Game",
            "Show Game List",
            "Edit Game",
            "Delete Game",
            "Add Game Copy",
            "Edit Game Copy",
            "Delete Game Copy",
            "Show Shelf",
            "<< Back",
        ];

        let sel = Select::with_theme(&theme)
            .with_prompt("Shelf Management")
            .items(&sub)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => {
                if let Some(data) = cli::run_interactive_capture(&ctx.bgg_client).await? {
                    ctx.capture_uc
                        .execute(
                            data.title,
                            data.year,
                            data.players,
                            data.duration,
                            data.age,
                            data.category,
                            data.publisher,
                            data.authors,
                            data.rating,
                        )
                        .await?;
                    println!("\n✅ Game captured successfully.");
                }
            }
            Some(1) => {
                let games = ctx.game_repo.all().await;
                cli::display_game_list(&games);
            }
            Some(2) => {
                let games = ctx.game_repo.all().await;
                if let Some(game) = cli::select_game(&games) {
                    if let Some(updated) = cli::run_interactive_game_edit(game.clone())? {
                        ctx.edit_game_uc
                            .execute(game.name, game.year, updated)
                            .await?;
                        println!("\n✅ Game metadata updated.");
                    }
                }
            }
            Some(3) => {
                let games = ctx.game_repo.all().await;
                if let Some(game) = cli::select_game(&games) {
                    if cli::confirm_delete(&format!("Game '{}'", game.name))? {
                        ctx.delete_game_uc.execute(game.name, game.year).await?;
                        println!("\n🗑️ Game deleted.");
                    }
                }
            }
            Some(4) => {
                let games = ctx.game_repo.all().await;
                reqcov!("UC.1.REQ.056");
                reqcov!("REQ.056");
                if let Ok(Some((name, year))) = cli::run_interactive_copy_creation(&games) {
                    match ctx.create_copy_uc.execute(name, year).await {
                        Ok(new_id) => println!("\n✅ Copy with ID {} registered on shelf.", new_id),
                        Err(e) => println!("\n❌ Error saving copy: {}", e),
                    }
                }
            }
            Some(5) => {
                let copies = ctx.copy_repo.all().await;
                let players = ctx.player_repo.all().await;
                if let Some(copy) = cli::select_game_copy_from_list(&copies) {
                    let old_id = copy.id;
                    if let Some(updated) = cli::run_interactive_copy_edit(copy, &players)? {
                        ctx.edit_copy_uc.execute(old_id, updated).await?;
                        println!("\n✅ Update successful.");
                    }
                }
            }
            Some(6) => {
                let copies = ctx.copy_repo.all().await;
                reqcov!("UC.2.REQ.055");
                reqcov!("REQ.055");
                if let Ok(Some(c_id)) = cli::select_game_copy(&copies) {
                    let id = c_id.parse::<i32>().unwrap_or(0);
                    if cli::confirm_delete(&format!("Copy ID {}", id))? {
                        ctx.delete_copy_uc.execute(id).await?;
                        println!("\n🗑️ Copy removed.");
                    }
                }
            }
            Some(7) => {
                let copies = ctx.copy_repo.all().await;
                cli::display_copy_list(&copies);
            }
            None | Some(8) => break,
            _ => {}
        }
    }
    Ok(())
}

/// Sub-menu for player profiles and lending logistics.
///
/// Coordinates player registration and tracks the physical location of
/// game copies (Lending/Returning).
async fn run_player_menu(ctx: &AppContext) -> Result<()> {
    let theme = ColorfulTheme::default();
    loop {
        let sub = vec![
            "New Player",
            "Show Player List",
            "Edit Player",
            "Lend Game",
            "Return Game",
            "Show Lending Overview",
            "<< Back",
        ];
        let sel = Select::with_theme(&theme)
            .items(&sub)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => {
                if let Ok(Some(data)) = cli::run_interactive_player_creation() {
                    match ctx
                        .create_player_uc
                        .execute(
                            data.nickname,
                            data.first_name,
                            data.last_name,
                            data.email,
                            data.details,
                            data.password,
                        )
                        .await
                    {
                        Ok(p) => println!("\n✅ Player '{}' created.", p.nickname),
                        Err(e) => println!("\n❌ Error: {}", e),
                    }
                }
            }
            Some(1) => {
                let players = ctx.player_repo.all().await;
                cli::display_player_list(&players);
            }
            Some(2) => {
                let players = ctx.player_repo.all().await;
                if let Some(player) = cli::select_player(&players) {
                    let old_email = player.email.clone();
                    if let Some(updated) = cli::run_interactive_player_edit(player)? {
                        ctx.edit_player_uc.execute(&old_email, updated).await?;
                        println!("\n✅ Player updated.");
                    }
                }
            }
            Some(3) => {
                let copies = ctx.copy_repo.all().await;
                let players = ctx.player_repo.all().await;
                reqcov!("UC.2.REQ.010");
                reqcov!("UC.4.REQ.019");
                reqcov!("UC.2.REQ.020");
                reqcov!("UC.4.REQ.021");
                reqcov!("REQ.010");
                reqcov!("REQ.019");
                reqcov!("REQ.020");
                reqcov!("REQ.021");
                if let Ok(Some(c_id)) = cli::select_game_copy(&copies) {
                    if let Ok(Some(p_email)) = cli::select_borrower(&players) {
                        let start =
                            cli::input_date("Start Date", chrono::Local::now().date_naive())?;
                        let default_due = start + chrono::Duration::days(30);
                        let due = cli::input_date("Due Date", default_due).ok();

                        match ctx.lend_game_uc.execute(&c_id, p_email, start, due).await {
                            Ok(_) => println!("\n✅ Lent successfully."),
                            Err(e) => println!("\n❌ Error: {}", e),
                        }
                    }
                }
            }
            Some(4) => {
                let copies = ctx.copy_repo.all().await;
                reqcov!("UC.2.REQ.012");
                reqcov!("UC.2.REQ.024");
                reqcov!("REQ.012");
                reqcov!("REQ.024");
                if let Ok(Some(c_id)) = cli::select_lent_copy(&copies) {
                    match ctx.return_game_uc.execute(&c_id).await {
                        Ok(_) => println!("\n✅ Returned successfully."),
                        Err(e) => println!("\n❌ Error: {}", e),
                    }
                }
            }
            Some(5) => {
                if let Ok(lent) = ctx.copy_repo.all_lent().await {
                    cli::display_lent_copy_list(&lent);
                }
            }
            None | Some(6) => break,
            _ => {}
        }
    }
    Ok(())
}

/// Sub-menu for event planning and session recording.
///
/// Orchestrates the scheduling of game nights and the logging of actual
/// game sessions (matches) including participants and results.
async fn run_planning_menu(ctx: &AppContext) -> Result<()> {
    let theme = ColorfulTheme::default();
    loop {
        let sub = vec![
            "Scheduled Game Nights",
            "Plan New Game Night",
            "Show My Shelf",
            "Record Session",
            "Edit Session",
            "Delete Session",
            "Show Session History",
            "<< Back",
        ];

        let sel = Select::with_theme(&theme)
            .items(&sub)
            .default(0)
            .interact_opt()?;

        match sel {
            Some(0) => {
                let nights = ctx.night_repo.all().await;
                cli::display_game_night_list(&nights);
            }
            Some(1) => {
                let players = ctx.player_repo.all().await;
                let copies = ctx.copy_repo.all().await;
                if let Ok(Some(d)) = cli::run_interactive_game_night_planning(&players, &copies) {
                    ctx.plan_night_uc
                        .execute(d.date, d.notes, d.participants, d.suggested_copies)
                        .await?;
                    println!("\n✅ Game night scheduled.");
                }
            }
            Some(2) => {
                let copies = ctx.copy_repo.all().await;
                cli::display_copy_list(&copies);
            }
            Some(3) => {
                let copies = ctx.copy_repo.all().await;
                let players = ctx.player_repo.all().await;
                let nights = ctx.night_repo.all().await;

                if let Ok(Some(d)) =
                    cli::run_interactive_game_session_capture(&copies, &players, &nights)
                {
                    let request = crate::application::use_cases::record_game_session::RecordGameSessionRequest {
                        game_copy_id: d.game_copy_id.parse::<i32>()?,
                        date: d.date,
                        participants: d.participants,
                        results: if d._results.is_empty() { None } else { Some(d._results) },
                    };

                    match ctx.record_session_uc.execute(request).await {
                        Ok(id) => println!("\n✅ Session saved! ID: {}", id),
                        Err(e) => println!("\n❌ Error: {}", e),
                    }
                }
            }
            Some(4) => {
                let sessions = ctx.session_repo.all().await;
                if let Some(session) = cli::select_game_session(&sessions) {
                    let partie_id = session.id.unwrap();
                    let participants = ctx.session_repo.get_participants(partie_id).await?;
                    let copies = ctx.copy_repo.all().await;
                    let players = ctx.player_repo.all().await;

                    if let Ok(Some((updated_s, updated_p))) = cli::run_interactive_game_session_edit(
                        session,
                        participants,
                        &copies,
                        &players,
                    ) {
                        ctx.session_repo
                            .update(partie_id, updated_s, updated_p)
                            .await?;
                        println!("\n✅ Session updated.");
                    }
                }
            }
            Some(5) => {
                let sessions = ctx.session_repo.all().await;
                if let Some(session) = cli::select_game_session(&sessions) {
                    let partie_id = session.id.unwrap();
                    if cli::confirm_delete(&format!("Session ID {}", partie_id))? {
                        ctx.session_repo.delete(partie_id).await?;
                        println!("\n🗑️ Session deleted.");
                    }
                }
            }
            Some(6) => {
                let sessions = ctx.session_repo.all().await;
                cli::display_game_session_list(&sessions);
            }
            None | Some(7) => break,
            _ => {}
        }
    }
    Ok(())
}
