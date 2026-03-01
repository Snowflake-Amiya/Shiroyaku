#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod embedding;
pub mod fetch;
pub mod search;
pub mod ui;

use serde::{Deserialize, Serialize};
use tauri::Manager;

/// Search result for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub rank: usize,
    pub name: String,
    pub score: f32,
    pub description_matches: usize,
    pub etiology_matches: usize,
    pub manifestation_matches: usize,
    pub description_text: Option<String>,
    pub etiology_text: Option<String>,
    pub manifestation_text: Option<String>,
}

/// Check if database is ready
#[tauri::command]
async fn check_database() -> Result<bool, String> {
    let has_embeddings = embedding::has_embeddings().await;
    Ok(has_embeddings)
}

/// Initialize database (fetch and embed if needed)
#[tauri::command]
async fn initialize_database(no_update: bool) -> Result<String, String> {
    let needs_fresh_data = !no_update && needs_fetch();
    
    if needs_fresh_data {
        let conditions = fetch::fetch_conditions(no_update)
            .await
            .map_err(|e| format!("Error fetching conditions: {}", e))?;
        
        if !conditions.is_empty() {
            let mut model = fastembed::TextEmbedding::try_new(
                fastembed::InitOptions::new(fastembed::EmbeddingModel::EmbeddingGemma300M),
            ).map_err(|e| format!("Error loading model: {}", e))?;
            
            embedding::embed_conditions(conditions, &mut model)
                .await
                .map_err(|e| format!("Error embedding: {}", e))?;
        }
    }
    
    Ok("Database initialized".to_string())
}

/// Perform a symptom search
#[tauri::command]
async fn search_symptoms(symptoms: String, top_k: usize) -> Result<Vec<SearchResult>, String> {
    if symptoms.trim().is_empty() {
        return Err("Please enter your symptoms".to_string());
    }
    
    if !embedding::has_embeddings().await {
        return Err("Database not initialized. Please run initialization first.".to_string());
    }
    
    let mut model = fastembed::TextEmbedding::try_new(
        fastembed::InitOptions::new(fastembed::EmbeddingModel::EmbeddingGemma300M),
    ).map_err(|e| format!("Error loading model: {}", e))?;
    
    let query_embedding = model
        .embed(vec![symptoms], None)
        .map_err(|e| format!("Error embedding query: {}", e))?[0]
        .clone();
    
    let results = search::cross_reference_search(query_embedding, top_k)
        .await
        .map_err(|e| format!("Search error: {}", e))?;
    
    let search_results: Vec<SearchResult> = results
        .into_iter()
        .enumerate()
        .map(|(i, r)| SearchResult {
            rank: i + 1,
            name: r.name,
            score: r.score,
            description_matches: r.description_matches,
            etiology_matches: r.etiology_matches,
            manifestation_matches: r.manifestation_matches,
            description_text: r.description_text,
            etiology_text: r.etiology_text,
            manifestation_text: r.manifestation_text,
        })
        .collect();
    
    Ok(search_results)
}

fn needs_fetch() -> bool {
    let xml_path = std::path::Path::new("data/mplus_topics_latest.xml");
    if !xml_path.exists() {
        return true;
    }
    
    let metadata_path = std::path::Path::new("data/conditions_metadata.json");
    if !metadata_path.exists() {
        return true;
    }
    
    if let Ok(metadata) = std::fs::metadata(&metadata_path) {
        if let Ok(modified) = metadata.modified() {
            let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
            let now = chrono::Utc::now();
            let days_since_update = (now - modified_time).num_days();
            
            if days_since_update >= 10 {
                return true;
            }
        }
    }
    
    false
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_database,
            initialize_database,
            search_symptoms,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("Shiroyaku - Medical Symptom Search").ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
