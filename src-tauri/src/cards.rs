use serde::Serialize;
use tauri::AppHandle;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CustomCardRow {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CustomCard {
    pub id: String,
    pub title: String,
    pub rows: Vec<CustomCardRow>,
    pub refresh_s: Option<u32>,
    pub error: Option<String>,
}

fn list(_app: &AppHandle) -> Vec<CustomCard> {
    Vec::new()
}

fn data(_app: &AppHandle, _id: &str) -> Result<serde_json::Value, String> {
    Err("This card has no data source.".into())
}

fn open_folder(_app: &AppHandle) -> Result<(), String> {
    Err("Custom cards are not supported on this device.".into())
}

#[tauri::command]
pub async fn get_custom_cards(app: AppHandle) -> Result<Vec<CustomCard>, String> {
    tauri::async_runtime::spawn_blocking(move || list(&app))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_custom_card_data(id: String, app: AppHandle) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || data(&app, &id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn open_custom_cards_folder(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || open_folder(&app))
        .await
        .map_err(|e| e.to_string())?
}
