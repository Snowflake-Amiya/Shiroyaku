# Development status

Currently this project is still in development, I would recommend waiting for a release
until actually used. The release won't take too long from now, around a 1-2 week*s or so.

# Shiroyaku - Medical Symptom Search Engine

A medical symptom search application that helps users find relevant medical conditions based on their symptoms. The application uses semantic embeddings to match user-described symptoms with conditions from MedlinePlus.

## Features

- **Symptom Search**: Enter your symptoms and get top 5 matching medical conditions
- **Cross-Reference Search**: Searches across descriptions, etiology, and manifestations
- **Relevance Scoring**: Conditions are ranked by relevance score
- **Detailed Information**: Expand any result to see full descriptions, causes, and symptoms
- **Professional Medical Context**: Designed for healthcare professionals

## Installation

### Prerequisites

- Rust (1.70+)
- Node.js (for building Tauri)
- System dependencies for Tauri (see below)

### System Dependencies

For Linux (Ubuntu/Debian):
```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf
```

For Fedora:
```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

### Build Steps

1. Install Tauri CLI:
```bash
npm install -g @tauri-apps/cli
```

2. Build the Tauri app:
```bash
cd src-tauri
cargo tauri build
```

3. Run the app:
```bash
cargo tauri dev
```

## Project Structure

```
.
├── src/                    # Original terminal application
│   ├── main.rs            # Entry point
│   ├── fetch/             # MedlinePlus data fetching
│   ├── embedding/         # LanceDB embeddings
│   └── search/            # Cross-reference search
├── src-tauri/             # Tauri desktop application
│   ├── src/
│   │   ├── lib.rs        # Tauri commands
│   │   ├── main.rs       # Tauri entry
│   │   ├── embedding.rs  # Embedding logic
│   │   ├── fetch.rs      # Data fetching
│   │   └── search.rs     # Search logic
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
├── dist/                  # Frontend assets
│   └── index.html        # Web UI
└── index.html            # Original UI reference
```

## Usage

1. Launch the application
2. On first run, the database will initialize (this may take a few minutes)
3. Enter your symptoms in the search box (e.g., "chest pain shortness of breath")
4. View the top 5 matching conditions
5. Click on any result to expand and see detailed information

## GUI Features

The GUI matches the style of the original index.html with:

- **Calm Color Palette**: Sage green accent (#6B8E8E) on warm off-white (#FAF9F6)
- **Search Engine Layout**: Centered search bar with instant filtering
- **Keyboard Navigation**: Arrow keys + Enter support
- **Filter Tabs**: All/Recent/Critical/Follow-up (visual only in Tauri version)
- **Clickable Results**: Expandable cards showing detailed condition information
- **Accessibility**: Semantic HTML, clear visual hierarchy, reduced motion support

## Medical Disclaimer

This application is for informational purposes only. It is NOT a diagnosis tool. Always consult a medical professional for proper evaluation and treatment.

## License

MIT License - See LICENSE file for details
