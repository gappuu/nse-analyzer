mod app_config;
use nse_analyzer::nse::nse_commands;
use nse_analyzer::mcx::mcx_commands;
use nse_analyzer::nse::config as nse_config;
// use nse_analyzer::mcx::config as mcx_config;
use app_config::{AppConfig, Colorize};
use nse_commands::NSECommands;
use mcx_commands::MCXCommands;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize configuration
    let app_config = AppConfig::from_env();
    app_config.validate()?;
    app_config.log_ci_config();

    // Execute the appropriate command based on mode and exchange
    execute_command(&app_config).await
}

/// Execute the appropriate command based on the configuration mode and exchange
async fn execute_command(config: &AppConfig) -> Result<()> {
    match config.mode.as_str() {
        "server" => {
            // Handle CI mode override
            if handle_ci_mode_override(&config.mode, &config.exchange) {
                run_batch_mode(config).await
            } else {
                run_server_mode(config).await
            }
        }
        "batch" => run_batch_mode(config).await,
        _ => {
            if nse_config::is_ci_environment() {
                println!("{} GitHub Actions only supports batch mode, switching to batch", "ℹ".blue());
                run_batch_mode(config).await
            } else {
                handle_invalid_mode(&config.mode)
            }
        }
    }
}

/// Run server mode for the specified exchange (NSE or MCX only - no combined)
async fn run_server_mode(config: &AppConfig) -> Result<()> {
    match config.exchange.as_str() {
        "nse" => {
            println!("{}", "Starting NSE API Server...".green());
            println!("{} Port: {}", "→".cyan(), config.port);
            println!("{} Exchange: NSE Only", "→".cyan());
            NSECommands::run_server(config.port).await
        }
        "mcx" => {
            println!("{}", "Starting MCX API Server...".green());
            println!("{} Port: {}", "→".cyan(), config.port);
            println!("{} Exchange: MCX Only", "→".cyan());
            MCXCommands::run_server(config.port).await
        }
        "both" => {
            println!("{}", "❌ Combined server not supported in separate implementation".red());
            println!("{}", "Run NSE and MCX servers separately:".yellow());
            println!("  NSE Server: MODE=server EXCHANGE=nse PORT=3001 cargo run");
            println!("  MCX Server: MODE=server EXCHANGE=mcx PORT=3002 cargo run");
            println!();
            print_orchestration_help();
            std::process::exit(1);
        }
        _ => {
            eprintln!("Invalid exchange '{}'. Use 'nse' or 'mcx'", config.exchange);
            print_usage();
            std::process::exit(1);
        }
    }
}

/// Run batch mode for the specified exchange(s)
async fn run_batch_mode(config: &AppConfig) -> Result<()> {
    match config.exchange.as_str() {
        "nse" => {
            println!("{}", "Running NSE batch analysis...".green());
            NSECommands::run_batch().await?;
            
            // Split batch file into individual ticker files
            split_batch_if_exists("batch_processed.json").await?;
            
            Ok(())
        }
        "mcx" => {
            println!("{}", "Running MCX batch analysis...".green());
            MCXCommands::run_batch().await?;
            
            // Split batch file into individual ticker files
            split_batch_if_exists("batch_processed.json").await?;
            
            Ok(())
        }
        "both" => {
            println!("{}", "Running batch analysis for both NSE and MCX...".green());
            
            // Run NSE batch first
            println!("\n{}", "Starting NSE batch analysis...".cyan());
            if let Err(e) = NSECommands::run_batch().await {
                eprintln!("{} NSE batch failed: {}", "✗".red(), e);
            } else {
                // Split NSE batch file
                if let Err(e) = split_batch_if_exists("batch_processed.json").await {
                    eprintln!("{} NSE batch split failed: {}", "✗".red(), e);
                }
            }
            
            // Then run MCX batch
            println!("\n{}", "Starting MCX batch analysis...".cyan());
            if let Err(e) = MCXCommands::run_batch().await {
                eprintln!("{} MCX batch failed: {}", "✗".red(), e);
            } else {
                // Split MCX batch file
                if let Err(e) = split_batch_if_exists("batch_processed.json").await {
                    eprintln!("{} MCX batch split failed: {}", "✗".red(), e);
                }
            }
            
            println!("\n{}", "Both analyses completed!".green());
            Ok(())
        }
        _ => {
            eprintln!("Invalid exchange '{}'. Use 'nse', 'mcx', or 'both'", config.exchange);
            print_usage();
            std::process::exit(1);
        }
    }
}

/// Split batch_processed.json into individual ticker files if it exists
async fn split_batch_if_exists(filename: &str) -> Result<()> {
    // Check if batch_processed.json exists in current directory or backend/
    let backend_path = format!("backend/{}", filename);
    let paths = [
        std::path::Path::new(filename),
        std::path::Path::new(&backend_path),
    ];
    
    for path in &paths {
        if path.exists() {
            println!("\n{} Found {}, splitting into individual files...", "→".cyan(), filename);
            return NSECommands::split_batch_file().await;
        }
    }
    
    println!("{} No {} found, skipping split", "ℹ".blue(), filename);
    Ok(())
}

/// Print server orchestration help
fn print_orchestration_help() {
    println!("{}", "🔧 Server Orchestration Options:".cyan().bold());
    println!();
    println!("{}", "1. Manual Orchestration (Development):".yellow());
    println!("   Terminal 1: MODE=server EXCHANGE=nse PORT=3001 cargo run");
    println!("   Terminal 2: MODE=server EXCHANGE=mcx PORT=3002 cargo run");
    println!();
    println!("{}", "2. Docker Compose Orchestration:".yellow());
    println!("   Use docker-compose.yml to run both services");
    println!();
    println!("{}", "3. Reverse Proxy (Production):".yellow());
    println!("   Nginx/Traefik to route /api/nse/* → NSE server");
    println!("   Nginx/Traefik to route /api/mcx/* → MCX server");
    println!();
    println!("{}", "4. Process Manager:".yellow());
    println!("   PM2, systemd, or supervisor to manage both processes");
}

/// Handle invalid mode by showing usage and exiting
fn handle_invalid_mode(mode: &str) -> Result<()> {
    eprintln!("Invalid mode '{}'. Use 'batch' or 'server'", mode);
    print_usage();
    std::process::exit(1);
}

/// Handle CI environment mode switching
fn handle_ci_mode_override(mode: &str, exchange: &str) -> bool {
    if nse_config::is_ci_environment() && (mode == "server") {
        match exchange {
            "nse" => NSECommands::handle_ci_mode_override(mode),
            "mcx" => MCXCommands::handle_ci_mode_override(mode),
            "both" => {
                println!("{} GitHub Actions only supports batch mode, running batch for both exchanges", "ℹ".blue());
                true
            }
            _ => true
        }
    } else {
        false
    }
}

/// Print usage instructions
fn print_usage() {
    eprintln!("Set environment variables to control execution:");
    eprintln!();
    eprintln!("Environment Variables:");
    eprintln!("  MODE or NSE_MODE or MCX_MODE  - Execution mode ('batch' or 'server')");
    eprintln!("  EXCHANGE                      - Exchange to use ('nse' or 'mcx')");
    eprintln!("  PORT or NSE_PORT or MCX_PORT  - Server port");
    eprintln!();
    eprintln!("Server Examples (Separate Services):");
    eprintln!("  MODE=server EXCHANGE=nse PORT=3001 cargo run      # NSE server on port 3001");
    eprintln!("  MODE=server EXCHANGE=mcx PORT=3002 cargo run      # MCX server on port 3002");
    eprintln!();
    eprintln!("Batch Examples:");
    eprintln!("  MODE=batch EXCHANGE=nse cargo run                 # NSE batch analysis");
    eprintln!("  MODE=batch EXCHANGE=mcx cargo run                 # MCX batch analysis");
    eprintln!("  MODE=batch EXCHANGE=both cargo run                # Both exchanges batch");
    eprintln!();
    eprintln!("GitHub Actions (CI):");
    eprintln!("  EXCHANGE=nse cargo run                            # Auto-switches to batch");
    eprintln!("  EXCHANGE=mcx cargo run                            # Auto-switches to batch");
    eprintln!("  EXCHANGE=both cargo run                           # Both in batch mode");
}