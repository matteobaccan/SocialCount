# AGENTS.md — Developer & Agent Guidelines for SocialCount

Welcome! This document provides complete context, architectural guidelines, code standards, and operational workflows for SocialCount. If you are an AI agent, developer, or automated tool running on a fresh environment without prior memory, read and follow this document strictly.

---

## 1. Project Overview & Ownership

- **Project**: SocialCount
- **Description**: Real-time desktop YouTube channel subscriber and view count tracker with transparency, always-on-top, delta tracking, and compact widget modes.
- **Author**: Matteo Baccan
- **Repository**: [https://github.com/matteobaccan/SocialCount](https://github.com/matteobaccan/SocialCount)
- **Documentation / Website**: [https://matteobaccan.github.io/SocialCount/](https://matteobaccan.github.io/SocialCount/)
- **License**: MIT License ([LICENSE](LICENSE))

### Mandatory File Header
Every Rust source file (`src/**/*.rs`, `examples/**/*.rs`) **MUST** start with the following header:

```rust
// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount
```

### Strict Editorial Constraints
- **NO AI or Assistant References**: Never mention AI, LLMs, prompts, or automated assistants in commits, PR descriptions, issues, documentation, code comments, or tool outputs. All changes must read as professional human contributions by or on behalf of Matteo Baccan.
- **Language**: All code comments, documentation, commit messages, and PRs must be written in **English**.
- **No Textual Emojis**: Do not use textual Unicode emojis (such as 🚀, ⚙️, 🌓, ⚠️, 🗑️) in markdown documentation, web pages, or UI labels. Use scalable SVG vector icons (`assets/icons/`) or custom vector drawing routines (`src/icons.rs`).

---

## 2. Technology Stack & Architecture

- **Language**: Rust (2021 edition)
- **GUI Framework**: `eframe` / `egui` (native desktop windowing and immediate-mode GUI)
- **HTTP Client**: `reqwest` (with `blocking` feature and system native-tls)
- **Data Serialization**: `serde`, `toml`
- **Asset Rendering**: `ab_glyph`, `image`

### Directory Layout

```
.
├── .github/
│   └── workflows/
│       ├── build.yml         # CI check: formats, clippy, unit tests across OS targets
│       ├── pages.yml         # Continuous deployment for docs/ to GitHub Pages
│       └── release.yml       # Automated multi-platform release on tag push (v*)
├── assets/
│   ├── icons/                # Vector SVG icons used across documentation and web pages
│   └── ...                   # Application logos, banners, and preview screenshots
├── docs/                     # Static site for GitHub Pages (https://matteobaccan.github.io/SocialCount/)
│   ├── index.html            # Main English documentation site
│   └── assets/               # Synchronized copies of web icons and preview assets
├── examples/
│   └── generate_assets.rs    # Standalone generator to render official logos, banners, screenshots
├── src/
│   ├── main.rs               # Application entry point, CLI arguments parser (--compact, --channels)
│   ├── app.rs                # Main egui application: UI rendering, window states, delta indicators
│   ├── channels.rs           # Channel configuration parser (channels.toml) and data structures
│   ├── icons.rs              # Pure vector icon renderers (moon icon, gear icon, badge markers)
│   └── youtube.rs            # YouTube public page scraper and stats parser with retry logic
├── channels.toml             # Default channel list template for the application
├── Cargo.toml                # Package metadata, dependencies, and edition settings
└── .imgbotconfig             # Configuration excluding ImgBot ("schedule": "never", "ignoredFiles": ["*"])
```

---

## 3. Core Behavioral Specifications

1. **Stat Delta Tracking**:
   - The application records the initial subscriber and view counts read at launch time.
   - Any variation occurring during the runtime is displayed with a badge/delta indicator (`+X` or `-X`) and a visual marker next to the statistic.
2. **Error & Retry Handling**:
   - If a channel statistics fetch fails or returns `N/D`, the scraper automatically retries fetching that channel after **1 minute** instead of waiting for the full standard refresh cycle.
3. **Window Modes**:
   - **Normal Mode**: Detailed multi-channel list with title bar, settings gear, and transparency slider.
   - **Compact Widget Mode** (`--compact` or shortcut): Frameless, minimal overlay with always-on-top support and draggable surface.

---

## 4. Development & Quality Assurance

Before submitting any commit or PR, verify that all quality gates pass:

```powershell
# 1. Run unit tests
cargo test

# 2. Strict linter check (zero warnings allowed)
cargo clippy --all-targets -- -D warnings

# 3. Code formatting
cargo fmt --check
```

### Regenerating Media Assets
When modifying visual UI, themes, or layouts, regenerate image assets using:
```powershell
cargo run --example generate_assets
```
Ensure newly generated images in `assets/` are synchronized with `docs/assets/` if they are referenced on the website.

---

## 5. Release Workflow (Automated)

Releases are fully automated via GitHub Actions ([`.github/workflows/release.yml`](.github/workflows/release.yml)). The workflow builds cross-platform release binaries for:
- Windows x64 (`socialcount-windows-x64-vX.Y.Z.zip`)
- Linux x64 (`socialcount-linux-x64-vX.Y.Z.tar.gz`)
- macOS Apple Silicon ARM64 (`socialcount-macos-arm64-vX.Y.Z.tar.gz`)
- macOS Intel x64 (`socialcount-macos-x64-vX.Y.Z.tar.gz`)

### Step-by-Step Release Procedure

To perform a new release (e.g., `1.1.0`):

1. **Update version in `Cargo.toml`**:
   ```toml
   [package]
   name = "socialcount"
   version = "1.1.0"
   ```
2. **Verify tests and linter**:
   ```powershell
   cargo test
   cargo clippy --all-targets -- -D warnings
   ```
3. **Commit the version update**:
   ```powershell
   git commit -am "Release 1.1.0"
   ```
4. **Create a matching annotated Git tag**:
   ```powershell
   git tag -a v1.1.0 -m "Release v1.1.0"
   ```
   *Note: The tag must start with `v` and match the exact version in `Cargo.toml`.*
5. **Push commit and tags to remote**:
   ```powershell
   git push origin main --tags
   ```
6. **Verify the workflow execution**:
   ```powershell
   gh run list --workflow Release --limit 1
   ```
   GitHub Actions will automatically verify version matching, compile on all runners, compute SHA-256 checksums, and publish the release with assets attached.

---

## 6. GitHub Pages Deployment

The documentation site in `docs/` is automatically deployed to [matteobaccan.github.io/SocialCount](https://matteobaccan.github.io/SocialCount/) via [`.github/workflows/pages.yml`](.github/workflows/pages.yml) on every push to the `main` branch.
- Keep `docs/index.html` updated with new features, shortcuts, and configuration options.
- Maintain TOML syntax highlighting using `<pre class="code-block"><code>...</code></pre>` with `white-space: pre;`.
