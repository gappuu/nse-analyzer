// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

#[derive(Clone)]
struct BackendConfig {
    exchange: &'static str,
    port: &'static str,
    health_path: &'static str,
}

const BACKENDS: &[BackendConfig] = &[
    BackendConfig {
        exchange: "nse",
        port: "3001",
        health_path: "/nse_health",
    },
    BackendConfig {
        exchange: "mcx",
        port: "3002",
        health_path: "/mcx_health",
    },
];

struct BackendManager {
    processes: Arc<Mutex<Vec<Child>>>,
}

impl BackendManager {
    fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn start_all(&self, backend_path: &str) -> Result<(), String> {
        let mut guard = self.processes.lock().unwrap();

        // Kill existing
        for mut child in guard.drain(..) {
            let _ = child.kill();
            let _ = child.wait();
        }

        for backend in BACKENDS {
            let child = Command::new(backend_path)
                .env("MODE", "server")
                .env("EXCHANGE", backend.exchange)
                .env("PORT", backend.port)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    format!(
                        "Failed to start {} backend on port {}: {}",
                        backend.exchange, backend.port, e
                    )
                })?;

            println!(
                "✅ Started {} backend on port {}",
                backend.exchange, backend.port
            );
            guard.push(child);
        }

        Ok(())
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

    let backend_path = [
        resource_dir.join("nse-analyzer"),
        resource_dir.join("resource/nse-analyzer"),
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

    backend_manager.start_all(&backend_path.to_string_lossy())?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    for backend in BACKENDS {
        check_health(backend.port, backend.health_path)
            .await
            .map_err(|e| {
                format!(
                    "{} backend on port {} failed health check: {}",
                    backend.exchange, backend.port, e
                )
            })?;
    }

    Ok("Both backends started successfully".to_string())
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
async fn get_backend_urls() -> Result<Vec<String>, String> {
    Ok(vec![
        "http://localhost:3001".to_string(),
        "http://localhost:3002".to_string(),
    ])
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
        .manage(BackendManager::new())
        .invoke_handler(tauri::generate_handler![
            start_backends,
            stop_backends,
            backend_status,
            get_backend_urls
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let manager: State<BackendManager> = handle.state();
                if start_backends(manager, handle.clone()).await.is_ok() {
                    let _ = handle.emit("backend-status", true);
                } else {
                    let _ = handle.emit("backend-status", false);
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