#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod selfupdate;
mod settings;
mod steam;
mod store;
mod vdf;

use serde::Serialize;
use tauri::http::Response;
use tauri::{AppHandle, Emitter};

fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> impl std::future::Future<Output = Result<T, String>> {
    async move { tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string()) }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryResponse {
    #[serde(flatten)]
    library: steam::Library,
    /// Set when a Web API key is configured but Steam couldn't be reached with it.
    api_error: Option<String>,
    complete: bool,
}

#[tauri::command]
async fn library(app: AppHandle) -> Result<LibraryResponse, String> {
    let mut lib = blocking(steam::library).await??;
    let key = settings::load(&app).web_api_key;
    let mut api_error = None;
    let mut complete = false;
    if !key.trim().is_empty() {
        complete = true;
        for a in lib.accounts.clone() {
            match store::owned_games(&key, &a.steam_id).await {
                Ok(games) if !games.is_empty() => steam::merge_owned(&mut lib, a.account_id, &games),
                Ok(_) => {
                    complete = false;
                    api_error.get_or_insert(format!("{}’s game details are private, so only games used on this PC are listed.", a.persona_name));
                }
                Err(e) => {
                    complete = false;
                    api_error = Some(e);
                    break;
                }
            }
        }
    }
    Ok(LibraryResponse { library: lib, api_error, complete })
}

#[tauri::command]
async fn status() -> Result<steam::Status, String> {
    blocking(steam::status).await
}

#[tauri::command]
async fn switch_account(app: AppHandle, account_name: String, launch: Option<u32>) -> Result<steam::SwitchResult, String> {
    let s = settings::load(&app);
    let opts = steam::SwitchOptions { show_window: s.show_steam_window, offline: s.offline_mode, launch };
    blocking(move || steam::switch(&account_name, opts, |stage| { let _ = app.emit("switch-progress", stage); })).await?
}

#[tauri::command]
async fn add_account(app: AppHandle) -> Result<(), String> {
    blocking(move || steam::add_account(|stage| { let _ = app.emit("switch-progress", stage); })).await?
}

#[tauri::command]
async fn details(ids: Vec<u32>) -> Result<Vec<store::Details>, String> {
    store::details(&ids).await
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    steam::open_url(&url)
}

#[tauri::command]
fn reveal(path: String) -> Result<(), String> {
    steam::reveal(&path)
}

#[tauri::command]
fn settings_get(app: AppHandle) -> settings::Settings {
    settings::load(&app)
}

#[tauri::command]
fn settings_set(app: AppHandle, value: settings::Settings) -> Result<(), String> {
    settings::save(&app, &value)
}

#[tauri::command]
async fn app_update_check(app: AppHandle) -> Result<selfupdate::Status, String> {
    selfupdate::check(&app).await
}

#[tauri::command]
async fn app_update_install(app: AppHandle) -> Result<(), String> {
    selfupdate::install(&app).await
}

fn mime(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn main() {
    selfupdate::cleanup();
    tauri::Builder::default()
        // Artwork and avatars straight from Steam's own caches, so the library works offline.
        .register_asynchronous_uri_scheme_protocol("art", |_ctx, request, responder| {
            let path = request.uri().path().to_string();
            tauri::async_runtime::spawn_blocking(move || {
                let found = steam::art_file(&path).and_then(|f| std::fs::read(&f).ok().map(|b| (f, b)));
                let res = match found {
                    Some((f, bytes)) => Response::builder()
                        .header("Content-Type", mime(&f))
                        .header("Cache-Control", "max-age=86400")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(bytes),
                    None => Response::builder().status(404).body(Vec::new()),
                };
                responder.respond(res.unwrap());
            });
        })
        .invoke_handler(tauri::generate_handler![
            library,
            status,
            switch_account,
            add_account,
            details,
            open_url,
            reveal,
            settings_get,
            settings_set,
            app_update_check,
            app_update_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hangar");
}
