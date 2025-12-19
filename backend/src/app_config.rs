use nse_analyzer::nse::config as nse_config;
// use nse_analyzer::mcx::config as mcx_config;
use anyhow::Result;

/// Application configuration handler
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mode: String,
    pub port: u16,
    pub exchange: String,  // "nse", "mcx", or "both"
}

impl AppConfig {
    /// Create new configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            mode: Self::get_execution_mode(),
            port: Self::get_port(),
            exchange: Self::get_exchange(),
        }
    }

    /// Log configuration details for CI environments
    pub fn log_ci_config(&self) {
        if nse_config::is_ci_environment() {
            println!("{}", "Running in CI environment (GitHub Actions)".blue().bold());
            println!("{} Mode: {}", "→".cyan(), self.mode.yellow());
            println!("{} Exchange: {}", "→".cyan(), self.exchange.yellow());
            
            if self.mode == "server" {
                println!("{} Server mode not supported in CI - switching to batch", "⚠".yellow());
            }
            println!();
        }
    }

    /// Get execution mode from environment or default to batch
    fn get_execution_mode() -> String {
        std::env::var("MODE")
            .or_else(|_| std::env::var("NSE_MODE"))
            .or_else(|_| std::env::var("MCX_MODE"))
            .unwrap_or_else(|_| "batch".to_string())
    }

    /// Get exchange from environment or default to both
    fn get_exchange() -> String {
        std::env::var("EXCHANGE").unwrap_or_else(|_| "both".to_string())
    }

    /// Get port from environment or default
    fn get_port() -> u16 {
        std::env::var("PORT")
            .or_else(|_| std::env::var("NSE_PORT"))
            .or_else(|_| std::env::var("MCX_PORT"))
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .unwrap_or(3001)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate exchange
        match self.exchange.as_str() {
            "nse" | "mcx" => Ok(()),
            "both" => {
                // Both is only allowed for batch mode
                if self.mode == "batch" {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "EXCHANGE=both is only supported in batch mode. For servers, run separately:\n  \
                         NSE: MODE=server EXCHANGE=nse PORT=3001\n  \
                         MCX: MODE=server EXCHANGE=mcx PORT=3002"
                    ))
                }
            }
            _ => Err(anyhow::anyhow!("Invalid exchange '{}'. Use 'nse', 'mcx', or 'both' (batch only)", self.exchange)),
        }
    }
}

// Re-export colored for use in main.rs
pub use colored::Colorize;