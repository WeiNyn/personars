# Personars

A personal developer toolbox built with Rust and [egui](https://github.com/emilk/egui/). Runs natively on desktop and in the browser as a PWA via WebAssembly.

## Features

Personars has three main modes, accessible from the top bar:

### 🔧 Tools

A collection of developer utilities, each running as a draggable window on desktop or as a full-screen view on mobile:

| Tool | Description |
|------|-------------|
| **Format Converter** | Convert between JSON, YAML, and TOML |
| **Epoch Converter** | Convert between Unix timestamps and human-readable dates |
| **Base64 Converter** | Encode and decode Base64 strings |
| **JWT Debugger** | Decode and inspect JWT tokens |
| **UUID Generator** | Generate UUIDs (v4, v7) |
| **Hash Generator** | Compute MD5, SHA-1, and SHA-256 hashes |
| **Password Generator** | Generate secure random passwords |
| **Regex Tester** | Test regular expressions with live matching |
| **QR Code Generator** | Generate QR codes from text |
| **Diff Viewer** | Side-by-side text diff comparison |
| **Todo List** | Simple task manager |

### 📒 Note Down

A Markdown notes editor with:
- Create, edit, and delete notes
- Markdown preview via `egui_commonmark`
- Full-text search across note titles
- Persistent storage (eframe persistence on native, IndexedDB on web)

### 💰 Finance

A personal finance tracker with:
- Multi-account transaction management
- Date/time picker and time-range filtering
- Transactions grouped by day
- Statistics dashboard with monthly summaries and category breakdowns (pie charts & trend lines via `egui_plot`)
- Persistent storage (eframe persistence on native, IndexedDB on web)

## Tech Stack

- **Language:** Rust (Edition 2024)
- **GUI Framework:** [egui](https://github.com/emilk/egui/) 0.33 / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) 0.33
- **Icons:** [egui-phosphor](https://crates.io/crates/egui-phosphor)
- **Web Storage:** [rexie](https://crates.io/crates/rexie) (IndexedDB wrapper for WASM)
- **Build (Web):** [Trunk](https://trunkrs.dev/)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.88+)

On Linux, install the following system dependencies:

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

On Fedora:

```bash
dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel
```

### Run Natively

```bash
cargo run --release
```

### Run on Web (WASM)

1. Install the WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Install [Trunk](https://trunkrs.dev/):
   ```bash
   cargo install --locked trunk
   ```
3. Serve locally:
   ```bash
   trunk serve
   ```
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser.

> **Note:** Appending `#dev` skips the service worker cache, ensuring you always load the latest build during development.

### Deploy to Web

```bash
trunk build --release
```

This generates a `dist/` directory containing a static site ready to be hosted on any static file server, including [GitHub Pages](https://pages.github.com/).

A GitHub Actions workflow (`.github/workflows/pages.yml`) is included for automatic deployment to GitHub Pages on push.

## Responsive Design

Personars adapts to screen size:

- **Desktop (≥ 500px):** Tools open as draggable/resizable windows within a central panel. A collapsible sidebar provides quick access.
- **Mobile (< 500px):** Tools open full-screen with a back button. Navigation collapses into a hamburger menu.

## Project Structure

```
src/
├── main.rs              # Entry point (native + WASM)
├── lib.rs               # Crate root
├── app.rs               # Main application, routing, layout
└── tools/
    ├── mod.rs            # Tool trait & ToolKind enum
    ├── format_converter.rs
    ├── epoch_converter.rs
    ├── base64_converter.rs
    ├── jwt_debugger.rs
    ├── uuid_generator.rs
    ├── hash_generator.rs
    ├── password_generator.rs
    ├── regex_tester.rs
    ├── qr_code_generator.rs
    ├── diff_viewer.rs
    ├── todo_list.rs
    ├── markdown_notes.rs
    ├── finance_tracker.rs
    ├── finance_stats.rs
    └── idb_storage.rs   # IndexedDB bridge (wasm32 only)
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
