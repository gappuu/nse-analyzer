use nse_analyzer::nse::config;
use anyhow::Result;

/// Application configuration handler
pub struct AppConfig {
    pub mode: String,
    pub port: u16,
}

impl AppConfig {
    /// Create new configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            mode: config::get_execution_mode(),
            port: Self::get_port(),
        }
    }

    /// Log configuration details for CI environments
    pub fn log_ci_config(&self) {
        if config::is_ci_environment() {
            println!("{}", "Running in CI environment (GitHub Actions)".blue().bold());
            println!("{} Mode: {}", "→".cyan(), self.mode.yellow());
            
            if self.mode == "server" {
                println!("{} Server mode not supported in CI - switching to batch", "⚠".yellow());
            }
            println!();
        }
    }

    /// Get port from environment or default
    fn get_port() -> u16 {
        std::env::var("NSE_PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .unwrap_or(3001)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Add any validation logic here if needed
        Ok(())
    }
}

// Re-export colored for use in main.rs
pub use colored::Colorize;