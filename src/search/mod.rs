use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::embedding::{get_table, search_table};

/// Final ranked condition result
#[derive(Debug, Clone)]
pub struct RankedCondition {
    pub name: String,
    pub score: f32,
    pub description_matches: usize,
    pub etiology_matches: usize,
    pub manifestation_matches: usize,
    pub description_text: Option<String>,
    pub etiology_text: Option<String>,
    pub manifestation_text: Option<String>,
}

/// Cross-reference search across all three embedding tables
pub async fn cross_reference_search(
    query_embedding: Vec<f32>,
    top_k_per_table: usize,
) -> Result<Vec<RankedCondition>> {
    println!("Searching for similar conditions...");
    
    let description_table = get_table("description_embeddings").await?;
    let etiology_table = get_table("etiology_embeddings").await?;
    let manifestation_table = get_table("manifestation_embeddings").await?;
    
    println!("  - Searching description embeddings...");
    let mut description_results = search_table(&description_table, query_embedding.clone(), top_k_per_table).await?;
    for r in description_results.iter_mut() {
        r.embedding_type = "description".to_string();
    }
    
    println!("  - Searching etiology embeddings...");
    let mut etiology_results = search_table(&etiology_table, query_embedding.clone(), top_k_per_table).await?;
    for r in etiology_results.iter_mut() {
        r.embedding_type = "etiology".to_string();
    }
    
    println!("  - Searching manifestation embeddings...");
    let mut manifestation_results = search_table(&manifestation_table, query_embedding.clone(), top_k_per_table).await?;
    for r in manifestation_results.iter_mut() {
        r.embedding_type = "manifestation".to_string();
    }
    
    let mut all_conditions: HashSet<String> = HashSet::new();
    for r in &description_results {
        all_conditions.insert(r.condition_name.clone());
    }
    for r in &etiology_results {
        all_conditions.insert(r.condition_name.clone());
    }
    for r in &manifestation_results {
        all_conditions.insert(r.condition_name.clone());
    }
    
    println!("  - Cross-referencing {} conditions...", all_conditions.len());
    
    let mut condition_texts: HashMap<String, (Option<String>, Option<String>, Option<String>)> = HashMap::new();
    
    for r in &description_results {
        let entry = condition_texts.entry(r.condition_name.clone()).or_insert((None, None, None));
        entry.0 = Some(r.text.clone());
    }
    for r in &etiology_results {
        let entry = condition_texts.entry(r.condition_name.clone()).or_insert((None, None, None));
        entry.1 = Some(r.text.clone());
    }
    for r in &manifestation_results {
        let entry = condition_texts.entry(r.condition_name.clone()).or_insert((None, None, None));
        entry.2 = Some(r.text.clone());
    }
    
    let description_set: HashSet<String> = description_results.iter().map(|r| r.condition_name.clone()).collect();
    let etiology_set: HashSet<String> = etiology_results.iter().map(|r| r.condition_name.clone()).collect();
    let manifestation_set: HashSet<String> = manifestation_results.iter().map(|r| r.condition_name.clone()).collect();
    
    let mut ranked_conditions: Vec<RankedCondition> = Vec::new();
    
    for condition_name in all_conditions {
        let desc_match = if description_set.contains(&condition_name) { 1 } else { 0 };
        let etio_match = if etiology_set.contains(&condition_name) { 1 } else { 0 };
        let manif_match = if manifestation_set.contains(&condition_name) { 1 } else { 0 };
        
        let score = (desc_match as f32 * 1.0) + (etio_match as f32 * 1.2) + (manif_match as f32 * 1.5);
        
        let texts = condition_texts.get(&condition_name).cloned().unwrap_or((None, None, None));
        
        ranked_conditions.push(RankedCondition {
            name: condition_name,
            score,
            description_matches: desc_match,
            etiology_matches: etio_match,
            manifestation_matches: manif_match,
            description_text: texts.0,
            etiology_text: texts.1,
            manifestation_text: texts.2,
        });
    }
    
    ranked_conditions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    
    let top_5: Vec<RankedCondition> = ranked_conditions.into_iter().take(5).collect();
    
    println!("Found top {} conditions", top_5.len());
    
    Ok(top_5)
}

/// Display search results to user
pub fn display_results(results: &[RankedCondition]) {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                    TOP 5 LIKELY CONDITIONS");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    for (i, condition) in results.iter().enumerate() {
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ #{}. {} ", i + 1, condition.name);
        println!("│    Score: {:.2}", condition.score);
        println!("│    Matches: {} desc, {} etiology, {} manifestations",
            condition.description_matches,
            condition.etiology_matches,
            condition.manifestation_matches
        );
        println!("└─────────────────────────────────────────────────────────────┘");
        
        if let Some(ref desc) = condition.description_text {
            if !desc.is_empty() {
                println!("   Description: {}...", &desc[..desc.len().min(100)]);
            }
        }
        if let Some(ref etio) = condition.etiology_text {
            if !etio.is_empty() {
                println!("   Etiology: {}...", &etio[..etio.len().min(100)]);
            }
        }
        if let Some(ref manif) = condition.manifestation_text {
            if !manif.is_empty() {
                println!("   Manifestations: {}...", &manif[..manif.len().min(100)]);
            }
        }
        println!();
    }
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("    This is NOT a diagnosis. Consult a medical professional.");
    println!("═══════════════════════════════════════════════════════════════\n");
}
