# Shiroyaku

A MedlinePlus-powered symptom search engine that uses vector embeddings to help you find relevant medical conditions based on your symptoms.

## Disclaimer

**This tool is NOT a diagnosis tool.** Always consult a medical professional for proper diagnosis and treatment. This program is for informational purposes only.

## Description

Shiroyaku is a minimalist program that fetches the latest medical information from MedlinePlus, embeds it using Gemma 300M, and allows you to search for conditions by describing your symptoms. It uses semantic search with LanceDB across three categories:
- Descriptions
- Etiology (causes)
- Manifestations (symptoms)

## Requirements

### All Platforms
- [Rust](https://rustup.rs/) (latest stable version)
- ~10GB free disk space

## Installation

### 1. Install Rust

**Windows:**
```powershell
# Download and run rustup-init.exe from https://rustup.rs/
# Or via PowerShell:
irm https://rustup.rs | iex
```

**MacOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone and Build

```bash
# Clone the repository
git clone https://github.com/Snowflake-Amiya/Shiroyaku.git
cd Shiroyaku

# Build the project
cargo build --release
```

## Usage

### First Run (Requires Internet)

```bash
# Run the program (will download latest medical data on first run)
cargo run
```

The first run will:
1. Fetch latest MedlinePlus data (~5-10 minutes)
2. Embed all conditions (~10-30 minutes depending on hardware)
3. Show the search interface

### Subsequent Runs (Works Offline)

```bash
# Skip fetching, use existing embeddings
cargo run -- --no-update
```

### Customize Search Results

```bash
# Change number of top results to consider
cargo run -- --top-k 30
```

## How to Use

1. **Launch the program**
2. **Read the disclaimer** - This is important!
3. **Enter your symptoms** - Describe what you're feeling (e.g., "I have a headache and fever")
4. **View results** - The program shows top 5 likely conditions with:
   - Match scores
   - Description snippets
   - Etiology (causes) snippets
   - Manifestation (symptom) snippets
5. **Search again** or type `q` to quit

## Features

- Fetches latest medical data from MedlinePlus
- Uses EmbeddingGemma 300M for semantic search
- Cross-references description, etiology, and manifestations
- Weighted scoring (manifestations weighted highest)
- Works offline after first run
- Progress bars for long operations

## Troubleshooting

### "No cached data available"
Run without `--no-update` flag to fetch fresh data:
```bash
cargo run
```

### Out of memory
The embedding model requires significant RAM. Close other applications or use a machine with more memory.

### Slow performance
The first run is slow because it needs to download and embed all medical data. Subsequent runs are much faster.

## License

MIT License
