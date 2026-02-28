use std::io;

/// Get user input for their symptoms/issues
pub fn get_user_input() -> String {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("              DESCRIBE YOUR SYMPTOMS OR CONCERNS");
    println!("═══════════════════════════════════════════════════════════════");
    println!("Enter what you're feeling or experiencing:\n");
    
    let mut input = String::new();
    
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");
    
    input.trim().to_string()
}

/// Display welcome message
pub fn display_welcome() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                        Shiroyaku                              ║");
    println!("║            MedlinePlus Symptom Search Engine                  ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  This tool helps find relevant medical conditions based on    ║");
    println!("║  your symptoms. It uses vector embeddings to search through   ║");
    println!("║  medical information from MedlinePlus.                        ║");
    println!("║                                                               ║");
    println!("║          WARNING: This is NOT a diagnosis tool.               ║");
    println!("║     Always consult a medical professional for proper          ║");
    println!("║     diagnosis and treatment.                                  ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
}

/// Display loading message for data fetching
pub fn display_fetching_message() {
    println!("Fetching latest MedlinePlus data...");
}

/// Display loading message for embedding
pub fn display_embedding_message() {
    println!("Processing embeddings (this may take a while on first run)...");
}

/// Display when skipping update
pub fn display_skipping_update() {
    println!("Skipping data update (using existing embeddings)");
}

/// Display initialization message
pub fn display_initializing() {
    print!("Initializing embedding database... ");
}

/// Ask user if they want to search again
pub fn ask_search_again() -> bool {
    println!("\nWould you like to search for another symptom? (y/n)");
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read user input");
    
    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}
