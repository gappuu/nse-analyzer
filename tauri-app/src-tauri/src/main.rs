// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

// Backend process manager
struct BackendProcess {
    process: Arc<Mutex<Option<Child>>>,
}

impl BackendProcess {
    fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }

    fn start(&self, backend_path: &str) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();
        
        // Kill existing process if any
        if let Some(mut child) = process_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Start new backend process with required environment variables
        match Command::new(backend_path)
            .env("NSE_MODE", "server")           // Set server mode
            .env("NSE_PORT", "3001")             // Set port
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                println!("✅ Backend started with NSE_MODE=server NSE_PORT=3001");
                *process_guard = Some(child);
                Ok(())
            }
            Err(e) => Err(format!("Failed to start backend: {}", e)),
        }
    }

    fn stop(&self) {
        let mut process_guard = self.process.lock().unwrap();
        if let Some(mut child) = process_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn is_running(&self) -> bool {
        let mut process_guard = self.process.lock().unwrap();
        if let Some(child) = process_guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has terminated
                    *process_guard = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking process, assume it's dead
                    *process_guard = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

// Tauri commands
#[tauri::command]
async fn start_backend(
    backend_process: State<'_, BackendProcess>,
    app_handle: AppHandle,
) -> Result<String, String> {
    // Try to get resource directory, but handle failure gracefully
    let backend_path = match app_handle.path().resource_dir() {
        Ok(resource_dir) => {
            println!("✅ Resource directory found: {:?}", resource_dir);
            
            // List contents for debugging
            if let Ok(entries) = std::fs::read_dir(&resource_dir) {
                println!("Contents of resource directory:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  📁 {:?}", entry.path());
                    }
                }
            }
            
            // Try multiple possible paths
            let possible_paths = [
                resource_dir.join("nse-analyzer"),
                resource_dir.join("resource/nse-analyzer"),
                resource_dir.join("resource").join("nse-analyzer"),
            ];
            
            let mut found_path = None;
            for path in &possible_paths {
                println!("🔍 Checking: {:?} - exists: {}", path, path.exists());
                if path.exists() {
                    found_path = Some(path.clone());
                    break;
                }
            }
            
            if let Some(path) = found_path {
                path
            } else {
                return Err("Backend binary not found in any expected resource location".to_string());
            }
        },
        Err(_) => {
            println!("❌ Resource directory failed, trying alternative approaches...");
            
            // Fallback 1: Try to construct path from app bundle structure
            if let Ok(app_config_dir) = app_handle.path().app_config_dir() {
                println!("🔄 Trying app config dir approach: {:?}", app_config_dir);
                
                // Navigate to Resources from app bundle
                if let Some(contents) = app_config_dir.parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent()) {
                    
                    let resources_dir = contents.join("Resources");
                    println!("🔄 Constructed Resources path: {:?}", resources_dir);
                    
                    let possible_paths = [
                        resources_dir.join("nse-analyzer"),
                        resources_dir.join("resource/nse-analyzer"),
                        resources_dir.join("resource").join("nse-analyzer"),
                    ];
                    
                    let mut found_path = None;
                    for path in &possible_paths {
                        println!("🔍 Checking constructed: {:?} - exists: {}", path, path.exists());
                        if path.exists() {
                            found_path = Some(path.clone());
                            break;
                        }
                    }
                    
                    if let Some(path) = found_path {
                        path
                    } else {
                        return Err("Backend binary not found via constructed path".to_string());
                    }
                } else {
                    return Err("Could not construct path to Resources directory".to_string());
                }
            } else {
                // Fallback 2: For testing, use absolute path to your backend
                let absolute_backend = std::path::PathBuf::from("/Users/adityachaudhary/Desktop/nse-analyzer/backend/target/release/nse-analyzer");
                println!("🔄 Using absolute path for testing: {:?}", absolute_backend);
                
                if absolute_backend.exists() {
                    absolute_backend
                } else {
                    return Err("Backend binary not found at absolute path".to_string());
                }
            }
        }
    };
    
    println!("✅ Using backend at: {:?}", backend_path);
    println!("🚀 Starting backend with NSE_MODE=server NSE_PORT=3001");
    
    // Make sure the binary is executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&backend_path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = std::fs::set_permissions(&backend_path, permissions);
        }
    }

    backend_process.start(&backend_path.to_string_lossy())?;
    
    // Wait longer for the backend to start (backend might need time to initialize)
    println!("⏳ Waiting for backend to start...");
    tokio::time::sleep(Duration::from_millis(5000)).await;
    
    // Check if backend is responding
    match check_backend_health().await {
        Ok(_) => {
            println!("✅ Backend health check passed!");
            Ok("Backend started successfully".to_string())
        },
        Err(e) => {
            println!("❌ Backend health check failed: {}", e);
            
            // Don't stop the backend immediately - it might still be starting
            println!("🔄 Backend might still be starting, checking port...");
            
            // Check if something is listening on port 3001
            match tokio::process::Command::new("lsof")
                .args(["-i", ":3001"])
                .output()
                .await 
            {
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        println!("✅ Something is listening on port 3001");
                        Ok("Backend started (port active, health check failed)".to_string())
                    } else {
                        println!("❌ Nothing listening on port 3001");
                        backend_process.stop();
                        Err(format!("Backend started but not responding: {}", e))
                    }
                },
                Err(_) => {
                    println!("❌ Could not check port status");
                    backend_process.stop();
                    Err(format!("Backend started but not responding: {}", e))
                }
            }
        }
    }
}

#[tauri::command]
async fn stop_backend(backend_process: State<'_, BackendProcess>) -> Result<String, String> {
    backend_process.stop();
    Ok("Backend stopped".to_string())
}

#[tauri::command]
async fn check_backend_status(backend_process: State<'_, BackendProcess>) -> Result<bool, String> {
    if backend_process.is_running() {
        // Double-check by making a health request
        match check_backend_health().await {
            Ok(_) => Ok(true),
            Err(_) => {
                // Backend process exists but not responding, stop it
                backend_process.stop();
                Ok(false)
            }
        }
    } else {
        Ok(false)
    }
}

async fn check_backend_health() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get("http://localhost:3001/health")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    Ok(())
}

#[tauri::command]
async fn get_backend_url() -> Result<String, String> {
    Ok("http://localhost:3001".to_string())
}

fn main() {
    env_logger::init();

    let backend_process = BackendProcess::new();

    tauri::Builder::default()
        .manage(backend_process)
        .invoke_handler(tauri::generate_handler![
            start_backend,
            stop_backend,
            check_backend_status,
            get_backend_url
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Start backend automatically when app starts
            tauri::async_runtime::spawn(async move {
                let backend_process: State<BackendProcess> = app_handle.state();
                
                // Try to start backend
                match start_backend(backend_process, app_handle.clone()).await {
                    Ok(msg) => {
                        log::info!("Backend startup: {}", msg);
                        let _ = app_handle.emit("backend-status", true);
                    }
                    Err(e) => {
                        log::error!("Failed to start backend: {}", e);
                        let _ = app_handle.emit("backend-status", false);
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let backend_process: State<BackendProcess> = window.state();
                backend_process.stop();
                log::info!("Backend stopped on app close");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}