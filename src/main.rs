mod fetch;
mod embedding;
mod search;
mod ui;

use anyhow::Result;
use clap::Parser;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about = "MedlinePlus Symptom Search Engine with LanceDB embeddings")]
struct Cli {
    /// Skip fetching latest data from MedlinePlus
    #[arg(long, default_value_t = false)]
    no_update: bool,
    
    /// Number of top results to consider from each embedding table
    #[arg(long, default_value_t = 20)]
    top_k: usize,
}

fn needs_fetch(no_update: bool) -> bool {
    if no_update {
        return false;
    }
    
    let xml_path = Path::new("data/mplus_topics_latest.xml");
    if !xml_path.exists() {
        return true;
    }
    
    let metadata_path = Path::new("data/conditions_metadata.json");
    if !metadata_path.exists() {
        return true;
    }
    
    if let Ok(metadata) = std::fs::metadata(&metadata_path) {
        if let Ok(modified) = metadata.modified() {
            let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
            let now = chrono::Utc::now();
            let days_since_update = (now - modified_time).num_days();
            
            if days_since_update < 10 {
                return false;
            } else {
                println!("Data is {} days old (more than 10 days). Updating...", days_since_update);
            }
        }
    }
    
    true
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    ui::display_welcome();
    
    let has_embeddings = embedding::has_embeddings().await;
    
    let needs_fresh_data = needs_fetch(cli.no_update);
    
    if needs_fresh_data {
        ui::display_fetching_message();
        
        // Fetch conditions from MedlinePlus
        let conditions = match fetch::fetch_conditions(cli.no_update).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error fetching conditions: {}", e);
                // Try to load from cache if fetch fails
                match fetch::load_conditions() {
                    Ok(c) => c,
                    Err(_) => {
                        eprintln!("No cached data available. Exiting.");
                        return Ok(());
                    }
                }
            }
        };
        
        if !conditions.is_empty() {
            ui::display_embedding_message();
            
            println!("Loading embedding model (gemma-300m)...");
            let mut model = TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::EmbeddingGemma300M),
            )?;
            
            embedding::embed_conditions(conditions, &mut model).await?;
        }
    } else if has_embeddings {
        ui::display_skipping_update();
    } else {
        println!("    No embeddings found and --no-update set.");
        println!("    Attempting to load cached data...");
        
        if let Ok(conditions) = fetch::load_conditions() {
            if !conditions.is_empty() {
                ui::display_embedding_message();
                println!("Loading embedding model (gemma-300m)...");
                let mut model = TextEmbedding::try_new(
                    InitOptions::new(EmbeddingModel::EmbeddingGemma300M),
                )?;
                embedding::embed_conditions(conditions, &mut model).await?;
            }
        } else {
            println!("No cached data available. Run without --no-update to fetch data.");
            return Ok(());
        }
    }
    
    ui::display_initializing();
    
    if !embedding::has_embeddings().await {
        println!("No embeddings found in database!");
        return Ok(());
    }
    println!("Database ready");
    
    loop {
        let user_input = ui::get_user_input();
        
        if user_input.to_lowercase() == "q" || user_input.is_empty() {
            println!("\nGoodbye! Take care!");
            break;
        }
        
        println!("Embedding your input...");
        let mut model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::EmbeddingGemma300M),
        )?;
        
        let query_embedding = model.embed(vec![user_input], None)?[0].clone();
        
        let results = search::cross_reference_search(
            query_embedding,
            cli.top_k,
        ).await?;
        
        search::display_results(&results);
        
        if !ui::ask_search_again() {
            println!("\nGoodbye! Take care!");
            break;
        }
    }
    
    Ok(())
}
