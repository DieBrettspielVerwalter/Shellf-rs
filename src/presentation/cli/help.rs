use dialoguer::Input;

/// Renders the interactive help menu to the console.
///
/// This function acts as a static documentation provider within the CLI.
/// It explains the core concepts of the application:
/// 1. **Collection Management**: Metadata (via BGG) and physical inventory.
/// 2. **Social & Lending**: Player profiles and tracking of physical copy locations.
/// 3. **Session Tracking**: Scheduling events and recording game results.
///
/// It also provides a brief guide on CLI navigation (Arrows, Enter, Esc).
pub fn display_help_menu() {
    println!("\n==================================================");
    println!("            📖 SHELLF - HELP SYSTEM");
    println!("==================================================");

    println!("\nShellf is your digital manager for board games.");
    println!("Here is an overview of the main sections:\n");

    println!("📂 1. GAMES & SHELF MANAGEMENT");
    println!("   Maintain your master data. You can search for games");
    println!("   directly via BoardGameGeek (BGG) and add them to your");
    println!("   'Shelf' (your physical copies).\n");

    println!("👥 2. PLAYERS & LENDING");
    println!("   Manage your gaming group and keep track of who has");
    println!("   borrowed which game. The system automatically");
    println!("   calculates deadlines and status.\n");

    println!("🎲 3. PLANNING & SESSIONS");
    println!("   Plan future game nights or record results from");
    println!("   sessions already played (who won, who participated?).\n");

    println!("⌨️  CONTROLS");
    println!("   • Navigate through menus using arrow keys.");
    println!("   • Confirm your selection with 'Enter'.");
    println!("   • 'Esc' or 'Back' takes you up one level.");
    println!("==================================================\n");

    // Prevents the help screen from closing immediately by waiting for user acknowledgement.
    let _: String = Input::new()
        .with_prompt("Press Enter to return to the main menu")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();
}
