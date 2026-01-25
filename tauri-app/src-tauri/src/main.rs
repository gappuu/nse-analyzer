// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

#[derive(Clone, serde::Serialize)]
struct BackendConfig {
    exchange: String,
    port: String,
    health_path: String,
}

struct BackendManager {
    processes: Arc<Mutex<Vec<Child>>>,
    configs: Arc<Mutex<Vec<BackendConfig>>>,
}

impl BackendManager {
    fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(Vec::new())),
            configs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn start_all(&self, backend_path: &str, app_handle: &AppHandle) -> Result<Vec<BackendConfig>, String> {
        let mut guard = self.processes.lock().unwrap();
        let mut configs_guard = self.configs.lock().unwrap();

        // Kill existing
        for mut child in guard.drain(..) {
            let _ = child.kill();
            let _ = child.wait();
        }
        configs_guard.clear();

        // Define backend configurations
        let backend_defs = vec![
            ("nse", "/nse_health"),
            ("mcx", "/mcx_health"),
        ];

        let mut started_configs = Vec::new();
        let mut used_ports = Vec::new(); // Track used ports

        for (exchange, health_path) in backend_defs {
            // Find an available port that hasn't been used yet
            let port = find_available_port_excluding(&used_ports)
                .ok_or_else(|| format!("No available port found for {} backend", exchange))?;
            
            used_ports.push(port.clone()); // Mark port as used

            let child = Command::new(backend_path)
                .env("MODE", "server")
                .env("EXCHANGE", exchange)
                .env("PORT", &port)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    format!(
                        "Failed to start {} backend on port {}: {}",
                        exchange, port, e
                    )
                })?;

            println!("✅ Started {} backend on port {}", exchange, port);

            let config = BackendConfig {
                exchange: exchange.to_string(),
                port: port.clone(),
                health_path: health_path.to_string(),
            };

            started_configs.push(config.clone());
            configs_guard.push(config);
            guard.push(child);
        }

        // Write ports to .env file for frontend
        if let Err(e) = write_env_file(&started_configs, app_handle) {
            eprintln!("Warning: Failed to write .env file: {}", e);
        }

        Ok(started_configs)
    }

    fn stop_all(&self) {
        let mut guard = self.processes.lock().unwrap();
        for mut child in guard.drain(..) {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn is_any_running(&self) -> bool {
        let mut guard = self.processes.lock().unwrap();
        guard.retain_mut(|child| match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        });
        !guard.is_empty()
    }

    fn get_configs(&self) -> Vec<BackendConfig> {
        self.configs.lock().unwrap().clone()
    }
}

// Find an available port excluding already used ports
fn find_available_port_excluding(used_ports: &[String]) -> Option<String> {
    // Try ports in range 3001-3100
    for port in 3001..3100 {
        let port_str = port.to_string();
        
        // Skip if this port is already used in this session
        if used_ports.contains(&port_str) {
            continue;
        }
        
        // Check if port is actually available
        if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
            drop(listener);
            return Some(port_str);
        }
    }
    None
}

// Write backend configuration to .env file
fn write_env_file(configs: &[BackendConfig], app_handle: &AppHandle) -> Result<(), String> {
    // Get the app's resource directory
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;

    // Look for .env file in multiple locations
    let possible_env_paths = vec![
        resource_dir.join(".env.local"),
        resource_dir.join("..").join(".env.local"),
        resource_dir.join("..").join("..").join(".env.local"),
        resource_dir.join("..").join("..").join("..").join(".env.local"),
    ];

    // Find the first existing .env.local or create in resource_dir
    let env_path = possible_env_paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| resource_dir.join(".env.local"));

    println!("📝 Writing backend configuration to: {}", env_path.display());

    // Read existing .env content
    let existing_content = if env_path.exists() {
        fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Remove old backend port entries
    let filtered_lines: Vec<&str> = existing_content
        .lines()
        .filter(|line| {
            !line.starts_with("NEXT_PUBLIC_NSE_API_PORT=")
                && !line.starts_with("NEXT_PUBLIC_MCX_API_PORT=")
                && !line.starts_with("NEXT_PUBLIC_IS_TAURI=")
        })
        .collect();

    let mut new_content = filtered_lines.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n');
    }

    // Add new backend ports
    new_content.push_str("NEXT_PUBLIC_IS_TAURI=true\n");
    
    for config in configs {
        let env_var = match config.exchange.as_str() {
            "nse" => "NEXT_PUBLIC_NSE_API_PORT",
            "mcx" => "NEXT_PUBLIC_MCX_API_PORT",
            _ => continue,
        };
        new_content.push_str(&format!("{}={}\n", env_var, config.port));
    }

    // Write to file
    let mut file = fs::File::create(&env_path)
        .map_err(|e| format!("Failed to create .env file: {}", e))?;
    
    file.write_all(new_content.as_bytes())
        .map_err(|e| format!("Failed to write to .env file: {}", e))?;

    println!("✅ Backend configuration written successfully");
    Ok(())
}

#[tauri::command]
async fn start_backends(
    backend_manager: State<'_, BackendManager>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|_| "Failed to locate resource directory")?;

    // Platform-specific binary names
    #[cfg(target_os = "windows")]
    let binary_name = "nse-analyzer.exe";
    
    #[cfg(not(target_os = "windows"))]
    let binary_name = "nse-analyzer";

    let backend_path = [
        resource_dir.join(binary_name),
        resource_dir.join(format!("resource/{}", binary_name)),
    ]
    .into_iter()
    .find(|p| p.exists())
    .ok_or("Backend binary not found")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&backend_path)
            .map_err(|_| "Failed to read backend metadata")?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&backend_path, perms).ok();
    }

    let configs = backend_manager.start_all(&backend_path.to_string_lossy(), &app_handle)?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    // Health check all backends
    for config in &configs {
        check_health(&config.port, &config.health_path)
            .await
            .map_err(|e| {
                format!(
                    "{} backend on port {} failed health check: {}",
                    config.exchange, config.port, e
                )
            })?;
    }

    Ok("All backends started successfully".to_string())
}

#[tauri::command]
async fn stop_backends(manager: State<'_, BackendManager>) -> Result<String, String> {
    manager.stop_all();
    Ok("All backends stopped".to_string())
}

#[tauri::command]
async fn backend_status(manager: State<'_, BackendManager>) -> Result<bool, String> {
    Ok(manager.is_any_running())
}

#[tauri::command]
async fn get_backend_urls(manager: State<'_, BackendManager>) -> Result<Vec<String>, String> {
    let configs = manager.get_configs();
    let urls = configs
        .iter()
        .map(|config| format!("http://localhost:{}", config.port))
        .collect();
    Ok(urls)
}

#[tauri::command]
async fn get_backend_configs(manager: State<'_, BackendManager>) -> Result<Vec<BackendConfig>, String> {
    Ok(manager.get_configs())
}

#[derive(serde::Serialize)]
struct PlatformInfo {
    is_tauri: bool,
    platform: String,
}

#[tauri::command]
fn get_platform_info() -> PlatformInfo {
    PlatformInfo {
        is_tauri: true,
        platform: std::env::consts::OS.to_string(),
    }
}

async fn check_health(port: &str, path: &str) -> Result<(), reqwest::Error> {
    reqwest::Client::new()
        .get(format!("http://localhost:{}{}", port, path))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    Ok(())
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(BackendManager::new())
        .invoke_handler(tauri::generate_handler![
            start_backends,
            stop_backends,
            backend_status,
            get_backend_urls,
            get_backend_configs,
            get_platform_info
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let manager: State<BackendManager> = handle.state();
                match start_backends(manager, handle.clone()).await {
                    Ok(_) => {
                        println!("✅ All backends started successfully");
                        let _ = handle.emit("backend-status", true);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start backends: {}", e);
                        let _ = handle.emit("backend-status", false);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let manager: State<BackendManager> = window.state();
                manager.stop_all();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}