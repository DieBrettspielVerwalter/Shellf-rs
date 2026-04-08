use anyhow::Result;
use chrono::NaiveDate;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};

/// Provides a universal date entry dialog with a default fallback value.
///
/// This helper standardizes date input across the CLI using the `YYYY-MM-DD` format.
///
/// # Arguments
///
/// * `prompt` - The descriptive text shown to the user.
/// * `default` - The `NaiveDate` that will be used if the user submits an empty input.
///
/// # Returns
///
/// A `Result` containing the parsed `NaiveDate`.
pub fn input_date(prompt: &str, default: NaiveDate) -> Result<NaiveDate> {
    let date_str: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} (YYYY-MM-DD)", prompt))
        .default(default.format("%Y-%m-%d").to_string())
        .interact_text()?;

    Ok(NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?)
}

/// A standard Yes/No confirmation dialog for destructive operations.
///
/// # Arguments
///
/// * `label` - A description of the item to be deleted (e.g., "this player").
///
/// # Returns
///
/// `Ok(true)` if the user confirmed, `Ok(false)` otherwise.
pub fn confirm_delete(label: &str) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Do you really want to delete {}?", label))
        .interact()?)
}

/// Displays the initial login dialog to capture database credentials.
///
/// This function gathers the email (DB user) and password required to
/// initialize the connection pool.
///
/// # Returns
///
/// A `Result` containing a tuple of `(email, password)`.
pub fn run_login_dialog() -> Result<(String, String)> {
    let theme = ColorfulTheme::default();
    println!("\n=== SHELLF LOGIN ===");

    let email = Input::with_theme(&theme)
        .with_prompt("E-Mail / DB-User")
        .interact_text()?;

    let password = Password::with_theme(&theme)
        .with_prompt("Password (leave empty for none)")
        .allow_empty_password(true)
        .interact()?;

    Ok((email, password))
}

/// Maps technical SQL errors to user-friendly localized messages.
///
/// This utility acts as a translator for MariaDB error codes, ensuring that
/// users receive actionable feedback (e.g., "Access denied") instead of
/// raw stack traces or internal query details.
///
/// # Arguments
///
/// * `err` - The `sqlx::Error` encountered during an operation.
#[allow(dead_code)]
pub fn translate_db_error(err: &sqlx::Error) -> String {
    if let Some(db_err) = err.as_database_error() {
        // Mapping specific MariaDB error codes to human-readable text
        match db_err.code() {
            Some(code) if code == "1045" => {
                "Access denied: Username or password incorrect.".to_string()
            }
            Some(code) if code == "1044" => {
                "Permission denied: User does not have rights for this database.".to_string()
            }
            Some(code) if code == "2002" || code == "1049" => {
                "Database not found or the server is offline.".to_string()
            }
            Some(code) if code == "1062" => "This entry already exists (Duplicate).".to_string(),
            _ => format!("A database error occurred: {}", db_err.message()),
        }
    } else {
        // Handling non-DB errors (e.g., network timeouts or protocol errors)
        format!("Connection error: {}", err)
    }
}
