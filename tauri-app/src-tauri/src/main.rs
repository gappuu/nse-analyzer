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

        // Start new backend process
        match Command::new(backend_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
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
    // Get the backend binary path from resources
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;
    
    // Your backend binary name from nse-analyzer/backend/target/release/nse-analyzer
    // let backend_path = resource_dir.join("nse-analyzer");
    let backend_path = resource_dir.join("resource/nse-analyzer");
    
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
    
    // Wait a moment for the backend to start
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    // Check if backend is responding
    // match check_backend_health().await {
    //     Ok(_) => Ok("Backend started successfully".to_string()),
    //     Err(e) => {
    //         backend_process.stop();
    //         Err(format!("Backend started but not responding: {}", e))
    //     }
    // }
    Ok("Backend started successfully".to_string())
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