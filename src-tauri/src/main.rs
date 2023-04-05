// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod errors;
mod json;
mod utils;

use errors::Error;
use json::{VersionJson, LauncherJson};
use tauri::{State, Manager, AppHandle, async_runtime};

const VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

struct LauncherState {
    pub client: reqwest::Client,
}

impl LauncherState {
    fn new() -> LauncherState {
        let client = reqwest::Client::new();
        LauncherState { client }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(LauncherState::new())
        .invoke_handler(tauri::generate_handler![launch_mc, get_versions_manifest, get_version_json, get_assets])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_versions_manifest(state: State<'_, LauncherState>) -> Result<LauncherJson, Error> {
    let req = state.client.get(VERSION_MANIFEST).send().await.map_err(Error::Request)?;
    req.json().await.map_err(Error::Request)
}

#[tauri::command]
async fn get_version_json(state: State<'_, LauncherState>, url: &str) -> Result<VersionJson, Error> {
    let res = state.client.get(url).send().await.map_err(Error::Request)?;
    res.json().await.map_err(Error::Request)
}

#[tauri::command]
async fn get_assets(app: AppHandle, state: State<'_, LauncherState>, index_url: String) -> Result<(), Error> {
    let window = app.get_window("main").unwrap();
    async_runtime::block_on(utils::write_assets(&state.client, &index_url, window))
}

#[tauri::command]
fn launch_mc(classpath: &str, version_json: VersionJson) -> Result<(), Error> {
    std::process::Command::new("java")
        .arg("-cp")
        .arg(classpath)
        .arg(&version_json.main_class)
        .arg("--accessToken")
        .arg("null")
        .arg("--version")
        .arg(&version_json.id)
        .arg("--assetIndex")
        .arg(&version_json.asset_index.id)
        .arg("./assets")
        .spawn().map_err(Error::Io)?;

    Ok(())
}