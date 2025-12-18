
mod app_config;
use nse_analyzer::nse::nse_commands;
use nse_analyzer::nse::config;
use app_config::{AppConfig, Colorize};
use nse_commands::NSECommands;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize configuration
    let app_config = AppConfig::from_env();
    app_config.validate()?;
    app_config.log_ci_config();

    // Execute the appropriate command based on mode
    execute_command(&app_config).await
}

/// Execute the appropriate command based on the configuration mode
async fn execute_command(config: &AppConfig) -> Result<()> {
    match config.mode.as_str() {
        "server" => {
            if NSECommands::handle_ci_mode_override(&config.mode) {
                NSECommands::run_batch().await
            } else {
                NSECommands::run_server(config.port).await
            }
        }
        "batch" => NSECommands::run_batch().await,
        _ => {
            if config::is_ci_environment() {
                println!("{} GitHub Actions only supports batch mode, switching to batch", "ℹ".blue());
                NSECommands::run_batch().await
            } else {
                handle_invalid_mode(&config.mode)
            }
        }
    }
}

/// Handle invalid mode by showing usage and exiting
fn handle_invalid_mode(mode: &str) -> Result<()> {
    eprintln!("Invalid mode '{}'. Use 'batch' or 'server'", mode);
    NSECommands::print_usage();
    std::process::exit(1);
}