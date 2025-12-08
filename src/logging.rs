use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging to both console and file
/// Log files are created in ./logs directory with daily rotation
pub fn init_logging() {
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("./logs").expect("Failed to create logs directory");

    // File appender with daily rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "./logs",
        "nse-analyzer.log",
    );

    // Create layered subscriber
    tracing_subscriber::registry()
        .with(
            // Console output
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_ansi(true),
        )
        .with(
            // File output with JSON formatting
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_ansi(false)
                .json(),
        )
        .with(
            // Environment filter (set via RUST_LOG env var)
            // Default to info level if not set
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{error, info, warn};

    #[test]
    fn test_logging() {
        init_logging();
        
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");
        
        // Check if log file was created
        assert!(std::path::Path::new("./logs").exists());
    }
}