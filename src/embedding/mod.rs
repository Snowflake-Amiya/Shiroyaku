use anyhow::Result;
use arrow_array::cast::AsArray;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray};
use futures::TryStreamExt;
use lancedb::connect;
use lancedb::query::{ExecutableQuery};
use std::sync::Arc;

use crate::fetch::ConditionData;

const DB_PATH: &str = "data/lancedb";

/// Search result from a single table
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub condition_name: String,
    pub text: String,
    pub embedding_type: String,
}

/// Embed and store condition data in LanceDB
pub async fn embed_conditions(
    conditions: Vec<ConditionData>,
    model: &mut fastembed::TextEmbedding,
) -> Result<()> {
    if conditions.is_empty() {
        println!("No conditions to embed");
        return Ok(());
    }
    
    let total = conditions.len() * 3;
    println!("Embedding {} conditions ({} total embeddings)...", conditions.len(), total);
    
    use indicatif::{ProgressBar, ProgressStyle};
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Embedding conditions...");
    
    let mut description_data: Vec<(&str, &str, Vec<f32>)> = Vec::new();
    let mut etiology_data: Vec<(&str, &str, Vec<f32>)> = Vec::new();
    let mut manifestation_data: Vec<(&str, &str, Vec<f32>)> = Vec::new();
    
    for condition in &conditions {
        if !condition.description.is_empty() && condition.description != "No summary available" {
            let emb = model.embed(vec![condition.description.clone()], None)?;
            description_data.push((condition.name.as_str(), condition.description.as_str(), emb[0].clone()));
            pb.inc(1);
        }
        
        if !condition.etiology.is_empty() && condition.etiology != "N/A" {
            let emb = model.embed(vec![condition.etiology.clone()], None)?;
            etiology_data.push((condition.name.as_str(), condition.etiology.as_str(), emb[0].clone()));
            pb.inc(1);
        }
        
        if !condition.manifestations.is_empty() && condition.manifestations != "N/A" {
            let emb = model.embed(vec![condition.manifestations.clone()], None)?;
            manifestation_data.push((condition.name.as_str(), condition.manifestations.as_str(), emb[0].clone()));
            pb.inc(1);
        }
    }
    
    pb.finish_with_message("Embedding complete!");
    
    let db = connect(DB_PATH).execute().await?;
    
    if !description_data.is_empty() {
        println!("Storing {} description embeddings...", description_data.len());
        create_and_insert_embeddings(&db, "description_embeddings", description_data).await?;
    }
    
    if !etiology_data.is_empty() {
        println!("Storing {} etiology embeddings...", etiology_data.len());
        create_and_insert_embeddings(&db, "etiology_embeddings", etiology_data).await?;
    }
    
    if !manifestation_data.is_empty() {
        println!("Storing {} manifestation embeddings...", manifestation_data.len());
        create_and_insert_embeddings(&db, "manifestation_embeddings", manifestation_data).await?;
    }
    
    println!("All embeddings stored!");
    Ok(())
}

/// Create table and insert embeddings using simple flat vector storage
async fn create_and_insert_embeddings(
    db: &lancedb::Connection,
    table_name: &str,
    data: Vec<(&str, &str, Vec<f32>)>,
) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    
    let embedding_dim = data[0].2.len();
    println!("   Embedding dimension: {}", embedding_dim);
    
    let mut condition_names: Vec<String> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    let mut vectors: Vec<String> = Vec::new();  // Store as JSON strings
    
    for (name, text, vec) in data {
        condition_names.push(name.to_string());
        texts.push(text.to_string());
        let vec_json = serde_json::to_string(&vec).unwrap_or_default();
        vectors.push(vec_json);
    }
    
    let batch = RecordBatch::try_new(
        Arc::new(arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("condition_name", arrow_schema::DataType::Utf8, false),
            arrow_schema::Field::new("text", arrow_schema::DataType::Utf8, false),
            arrow_schema::Field::new("vector", arrow_schema::DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(condition_names)),
            Arc::new(StringArray::from(texts)),
            Arc::new(StringArray::from(vectors)),
        ],
    )?;
    
    match db.open_table(table_name).execute().await {
        Ok(table) => {
            let iter = RecordBatchIterator::new(std::iter::once(Ok::<_, arrow::error::ArrowError>(batch.clone())), batch.schema());
            table.add(iter).execute().await?;
        }
        Err(_) => {
            let iter = RecordBatchIterator::new(std::iter::once(Ok::<_, arrow::error::ArrowError>(batch.clone())), batch.schema());
            db.create_table(table_name, iter).execute().await?;
        }
    }
    
    Ok(())
}

/// Search for similar embeddings - compute cosine similarity manually
pub async fn search_table(
    table: &lancedb::Table,
    query_embedding: Vec<f32>,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let stream = table.query().execute().await?;
    
    let results: Vec<RecordBatch> = stream.try_collect::<Vec<_>>().await?;
    
    let mut scored_results: Vec<(String, String, f32)> = Vec::new();
    
    for batch in results.iter() {
        if let Some(name_col) = batch.column_by_name("condition_name") {
            if let Some(text_col) = batch.column_by_name("text") {
                if let Some(vector_col) = batch.column_by_name("vector") {
                    let name_array = name_col.as_string::<i32>();
                    let text_array = text_col.as_string::<i32>();
                    let vector_array = vector_col.as_string::<i32>();
                    
                    for row_idx in 0..batch.num_rows() {
                        let name = name_array.value(row_idx).to_string();
                        let text = text_array.value(row_idx).to_string();
                        let vector_str = vector_array.value(row_idx);
                        
                        if let Ok(target_vec) = serde_json::from_str::<Vec<f32>>(vector_str) {
                            let similarity = cosine_similarity(&query_embedding, &target_vec);
                            scored_results.push((name, text, similarity));
                        }
                    }
                }
            }
        }
    }
    
    scored_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    
    let top_results: Vec<SearchResult> = scored_results
        .into_iter()
        .take(limit)
        .map(|(name, text, _)| SearchResult {
            condition_name: name,
            text,
            embedding_type: "".to_string(),
        })
        .collect();
    
    Ok(top_results)
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

/// Check if embeddings exist in the database
pub async fn has_embeddings() -> bool {
    match connect(DB_PATH).execute().await {
        Ok(db) => {
            db.open_table("description_embeddings").execute().await.is_ok()
        }
        Err(_) => false,
    }
}

/// Get a table from the database
pub async fn get_table(table_name: &str) -> Result<lancedb::Table> {
    let db = connect(DB_PATH).execute().await?;
    Ok(db.open_table(table_name).execute().await?)
}
