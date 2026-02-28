use chrono::{Duration, Local};
use roxmltree::{Document, ParsingOptions};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio::task;

/// Topic information extracted from MedlinePlus XML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub title: String,
    pub medline_url: String,
    pub full_summary: String,
    pub groups: Vec<String>,
}

/// Condition data with separated sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionData {
    pub name: String,
    pub medline_url: String,
    pub groups: Vec<String>,
    pub description: String,
    pub etiology: String,
    pub manifestations: String,
    pub treatments: String,
}

/// Fetch and parse MedlinePlus data (async wrapper)
pub async fn fetch_conditions(no_update: bool) -> Result<Vec<ConditionData>, Box<dyn Error + Send + Sync>> {
    if no_update {
        println!("Skipping data fetch (--no-update flag)");
        return Ok(Vec::new());
    }

    let result = task::spawn_blocking(move || {
        fetch_conditions_sync(no_update)
    }).await?;

    result
}

/// Synchronous fetch logic
fn fetch_conditions_sync(no_update: bool) -> Result<Vec<ConditionData>, Box<dyn Error + Send + Sync>> {
    if no_update {
        println!("Skipping data fetch (--no-update flag)");
        return Ok(Vec::new());
    }

    println!("Finding latest MedlinePlus XML...");
    let client = reqwest::blocking::Client::builder()
        .user_agent("TakeUrMeds/1.0 (+https://github.com/yourname/take_ur_meds)")
        .build()?;

    let latest_xml_url = find_latest_xml_url(&client)?;
    println!("Downloading: {}", latest_xml_url);

    let xml_text = client.get(&latest_xml_url).send()?.text()?;
    
    let xml_path = Path::new("data").join("mplus_topics_latest.xml");
    fs::create_dir_all("data")?;
    fs::write(&xml_path, &xml_text)?;
    println!("XML saved to {}", xml_path.display());

    println!("Parsing XML...");
    let doc = Document::parse_with_options(
        &xml_text,
        ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let root = doc.root_element();
    let mut all_topics: Vec<TopicInfo> = Vec::new();

    for node in root.descendants().filter(|n| n.has_tag_name("health-topic")) {
        if let Some(title) = node.attribute("title") {
            if let Some(lang) = node.attribute("language") {
                if lang != "English" && lang != "en" {
                    continue;
                }
            }

            let medline_url = node.attribute("url").unwrap_or("").to_string();

            let full_summary = if let Some(summary_node) = node.children().find(|n| n.has_tag_name("full-summary")) {
                summary_node
                    .descendants()
                    .filter_map(|n| n.text())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace("\n\n\n", "\n\n")
                    .trim()
                    .to_string()
            } else {
                String::new()
            };

            let mut groups = Vec::new();
            for g in node.children().filter(|n| n.has_tag_name("group")) {
                if let Some(txt) = g.text() {
                    groups.push(txt.trim().to_string());
                }
            }

            all_topics.push(TopicInfo {
                title: title.to_string(),
                medline_url,
                full_summary,
                groups,
            });
        }
    }

    println!("Parsed {} English topics.", all_topics.len());

    // Filter to diseases, disorders, conditions
    let topics_to_process: Vec<_> = all_topics
        .into_iter()
        .filter(|t| {
            let title_lower = t.title.to_lowercase();
            let groups_lower: Vec<String> = t.groups.iter().map(|g| g.to_lowercase()).collect();
            title_lower.contains("disease")
                || title_lower.contains("disorder")
                || title_lower.contains("syndrome")
                || title_lower.contains("cancer")
                || title_lower.contains("infection")
                || title_lower.contains("tumor")
                || title_lower.contains("arthritis")
                || title_lower.contains("diabetes")
                || groups_lower.iter().any(|g| {
                    g.contains("disorder")
                        || g.contains("cancer")
                        || g.contains("infection")
                        || g.contains("injury")
                        || g.contains("mental health")
                })
        })
        .collect();

    println!(
        "Filtered to {} diseases, disorders & conditions.",
        topics_to_process.len()
    );

    // Extract sections for each condition
    let conditions: Vec<ConditionData> = topics_to_process
        .into_iter()
        .map(|topic| {
            let (description, etiology, manifestations, treatments) =
                extract_sections(&topic.full_summary);

            ConditionData {
                name: topic.title,
                medline_url: topic.medline_url,
                groups: topic.groups,
                description,
                etiology,
                manifestations,
                treatments,
            }
        })
        .collect();

    // Save metadata
    let metadata_path = Path::new("data").join("conditions_metadata.json");
    let metadata_json = serde_json::to_string_pretty(&conditions)?;
    fs::write(&metadata_path, metadata_json)?;
    println!("Metadata saved to {}", metadata_path.display());

    Ok(conditions)
}

/// Extract sections from full summary
fn extract_sections(summary: &str) -> (String, String, String, String) {
    if summary.trim().is_empty() {
        return (
            "No summary available".to_string(),
            "N/A".to_string(),
            "N/A".to_string(),
            "N/A".to_string(),
        );
    }

    let lower = summary.to_lowercase();
    let first_part = summary.lines().take(20).collect::<Vec<_>>().join("\n");

    let etiology = extract_section(&lower, summary, &["cause", "caused by", "etiology", "risk factor"]);
    let manifestations =
        extract_section(&lower, summary, &["symptom", "sign", "manifestation", "present with"]);
    let treatments =
        extract_section(&lower, summary, &["treat", "therapy", "treatment", "medication", "surgery"]);

    (first_part, etiology, manifestations, treatments)
}

fn extract_section(lower: &str, original: &str, keywords: &[&str]) -> String {
    for &kw in keywords {
        if let Some(pos) = lower.find(kw) {
            let start = if pos > 100 { pos - 100 } else { 0 };
            let slice = &original[start..];
            if let Some(end) = slice.find("\n\n") {
                return slice[..end].trim().to_string();
            } else {
                return slice.lines().take(15).collect::<Vec<_>>().join("\n");
            }
        }
    }
    "Details integrated in the description above.".to_string()
}

fn find_latest_xml_url(client: &reqwest::blocking::Client) -> Result<String, Box<dyn Error + Send + Sync>> {
    let today = Local::now().date_naive();
    for i in 0..7 {
        let date = today - Duration::days(i);
        let candidate = format!("https://medlineplus.gov/xml/mplus_topics_{}.xml", date.format("%Y-%m-%d"));
        if client.head(&candidate).send().is_ok() {
            return Ok(candidate);
        }
    }
    // Fallback
    Ok("https://medlineplus.gov/xml/mplus_topics_2026-02-25.xml".to_string())
}

/// Load conditions from saved metadata
pub fn load_conditions() -> Result<Vec<ConditionData>, Box<dyn Error + Send + Sync>> {
    let metadata_path = Path::new("data").join("conditions_metadata.json");
    if metadata_path.exists() {
        let content = fs::read_to_string(&metadata_path)?;
        let conditions: Vec<ConditionData> = serde_json::from_str(&content)?;
        println!("Loaded {} conditions from cache", conditions.len());
        Ok(conditions)
    } else {
        Err("No cached data found".into())
    }
}
