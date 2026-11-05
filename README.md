<div align="center">

<img src="assets/socialcount-banner.png" alt="SocialCount Banner" width="100%" />

# <img src="assets/icons/youtube.svg" width="32" height="32" valign="middle" alt="SocialCount" /> SocialCount

**A modern, lightweight, transparent, always-on-top desktop widget developed in Rust to monitor YouTube channel statistics in real time.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Website](https://img.shields.io/badge/Website-matteobaccan.github.io%2FSocialCount-blueviolet.svg)](https://matteobaccan.github.io/SocialCount/)
[![Repository](https://img.shields.io/badge/GitHub-matteobaccan%2FSocialCount-red.svg)](https://github.com/matteobaccan/SocialCount)

</div>

---

## <img src="assets/icons/camera.svg" width="22" height="22" valign="middle" alt="Screenshots" /> Screenshots

<div align="center">
  <table>
    <tr>
      <td align="center"><b>Normal View (Glassmorphism)</b></td>
      <td align="center"><b>Ultra-Compact View (Double-Click)</b></td>
    </tr>
    <tr>
      <td align="center"><img src="assets/screenshot-normal.png" alt="SocialCount Normal View" width="420" /></td>
      <td align="center"><img src="assets/screenshot-compact.png" alt="SocialCount Compact View" width="360" /></td>
    </tr>
  </table>
</div>

---

## <img src="assets/icons/sparkles.svg" width="22" height="22" valign="middle" alt="Key Features" /> Key Features

- **Transparent & Borderless Window**: Clean system chrome-free window with native transparency that blends seamlessly onto your desktop.
- **Always on Top**: Stays floating above all active applications during work or gaming sessions.
- **High-Impact Glassmorphism UI**:
  - Frosted glass container with ambient drop shadow and subtle reflection border.
  - Floating channel cards with dark acrylic styling.
  - Real-time animated pulsing emerald live indicator dot.
  - Crisp vector YouTube badge logo integrated into the header.
- **Circular Channel Avatars**:
  - Channel profile pictures downloaded in the background, clipped to a perfect circle with anti-aliased edge masking and subtle highlight ring.
- **4 Live Counter Statistics (Zero Emojis)**:
  - **Emerald Pill (Subscribers)**: Vector user silhouette icon + clean formatted count + real-time delta tracking (`+N`).
  - **Sapphire Pill (Views)**: Vector eye icon with pupil highlight + formatted view count + delta since launch.
  - **Purple Pill (Videos)**: Vector video camera icon + total uploaded video count + variation indicator.
  - **Amber Pill (Comments)**: Vector speech bubble icon + cumulative comments sum tracked across recent uploads via SQLite.
  - Equal width and responsive layout matching window width.
  - **100% solid & opaque counter pills** regardless of chosen background transparency, ensuring maximum contrast and legibility.
- **Detailed Channel View & Recent Videos**:
  - Click on any channel's avatar or the **Details** button to open a dedicated modal displaying full metrics and recent uploads (with individual views, likes, comments, and direct links to YouTube).
- **Real-Time Change Detection**:
  - Automatically highlights counter pills with animated markers and border accents whenever subscriber, view, video, or comment counts update.
- **Automatic Sorting by Subscribers**: Channels are dynamically ordered by subscriber count in descending order.
- **Continuous Opacity Slider**:
  - Adjust background opacity in real time from **0% (completely transparent)** to **100% opaque**. At 0% opacity, all cards and backgrounds vanish, leaving only the floating circular avatars and counter pills visible.
- **Ultra-Compact Mode (Double-Click or `--compact`)**:
  - **Double-click anywhere on the title bar** or run with `--compact` to shrink into an ultra-compact widget: displays the circular avatar on the left and a balanced 2x2 grid with all 4 counter pills on the right.
  - Double-click again to restore normal view.
- **Dual Fetch Engine with SQLite Caching**:
  - Optional **YouTube Data API v3** support for enriched video and comment stats with embedded SQLite tiered caching (`socialcount.db`).
  - Automatic fallback to **public HTML scraping** when no API key is specified or when quota is exceeded (zero configuration required).
- **Smooth Multi-Screen Dragging**:
  - Custom coordinate-based dragging with direct position updates and `SWP_NOSIZE`, preventing Aero Snap dockings or unwanted resizing/ballooning when dragged across monitors with different DPIs or resolutions.
- **Full CRUD Profile Manager (Right-Click or Settings)**:
  - **Right-click anywhere** or click the settings header button to open the channel manager.
  - **Create**: Add new YouTube channels (auto-formatted with `@`).
  - **Read**: View list with live avatars and handles.
  - **Update**: Inline handle editing, reorder channels up/down.
  - **Delete**: Remove channels from monitoring.
  - Automatically persists changes to `channels.toml` and spawns immediate background fetch workers.
- **Sticky Footer with Timestamp**:
  - Real-time pulsing live dot with the exact timestamp of the last successful refresh (e.g. `04/11/2026 21:05:12`).
- **Simple TOML Configuration (`channels.toml`)**:
  - Reads channels and settings from a lightweight configuration file editable on the fly without recompilation.
- **Direct Web Links**:
  - Click on any channel's name or avatar to open its YouTube page directly in your default browser.

---

## <img src="assets/icons/rocket.svg" width="22" height="22" valign="middle" alt="Getting Started" /> Getting Started

### Prerequisites
- **Rust** (with Cargo): Ensure Rust 1.75+ is installed (`rustc --version`).

### Build and Run
```bash
# Clone the repository
git clone https://github.com/matteobaccan/SocialCount.git
cd SocialCount

# Run directly via Cargo
cargo run --release
```

Or build the standalone release binary:
```bash
cargo build --release
.\target\release\socialcount.exe
```

---

## <img src="assets/icons/settings.svg" width="22" height="22" valign="middle" alt="Configuration" /> Configuration (`channels.toml`)

On startup, SocialCount reads [`channels.toml`](channels.toml). If not present, a default configuration is generated:

```toml
# SocialCount configuration file
# List of YouTube channels to monitor (handles with or without @)
channels = [
    "@matteobaccan"
]

# Optional YouTube Data API v3 key (leave commented to use public HTML scraping)
# youtube_api_key = "YOUR_YOUTUBE_DATA_API_KEY"

# Automatic refresh interval in seconds (default: 300 = 5 minutes)
refresh_interval_secs = 300

# Background opacity (0 = fully transparent, 255 = opaque, default: 190)
card_opacity = 190

# Compact mode (true = compact widget, false = full view)
compact_mode = false

# Window dimensions (width and height in logical pixels)
window_width = 360.0
window_height = 440.0
```

---

## <img src="assets/icons/shield.svg" width="22" height="22" valign="middle" alt="YouTube Data API Guide" /> YouTube Data API Key Guide

SocialCount supports the official **YouTube Data API v3** for rich analytics, per-video statistics (views, likes, comments), and aggregated comment tracking stored in a local SQLite cache. When no key is configured, SocialCount automatically falls back to public HTML scraping.

### Step 1: Create a Google Cloud Project

1. Open the [Google Cloud Console](https://console.cloud.google.com/).
2. Sign in with your Google account.
3. In the top navigation bar, click the project dropdown and select **New Project**.
4. Name your project (e.g., `SocialCount`) and click **Create**.

### Step 2: Enable the YouTube Data API v3

1. Make sure your newly created project is selected in the top bar.
2. Open the navigation menu and go to **APIs & Services** > **Library**.
3. Search for **YouTube Data API v3** and select it from the results list.
4. Click **Enable**.

### Step 3: Generate and Restrict Your API Key

1. Navigate to **APIs & Services** > **Credentials**.
2. Click **+ Create Credentials** at the top of the page and select **API key**.
3. Copy the generated key string (starting with `AIzaSy...`).
4. *(Recommended)* Click **Edit API key**:
   - Under **API restrictions**, select **Restrict key**.
   - In the dropdown, choose **YouTube Data API v3**.
   - Click **Save**.

### Step 4: Configure Your API Key in SocialCount

SocialCount automatically detects and loads your key using the following priority resolution:

#### Method 1: Local `secrets.toml` (Recommended for Development)
Create a file named `secrets.toml` in the project root:

```toml
# SocialCount - Local Secrets (Ignored by Git)
youtube_api_key = "AIzaSyYourGeneratedApiKeyHere"
```

> [!NOTE]
> `secrets.toml` is included in `.gitignore` by default. Your personal key will never be committed or published to Git repositories.

#### Method 2: Application Configuration (`channels.toml`)
You can also place the key directly in `channels.toml`:

```toml
# Optional YouTube Data API v3 key
youtube_api_key = "AIzaSyYourGeneratedApiKeyHere"
```

#### Method 3: Environment Variable
Alternatively, export the key in your terminal or shell profile:

```bash
# Linux / macOS
export YOUTUBE_API_KEY="AIzaSyYourGeneratedApiKeyHere"

# Windows PowerShell
$env:YOUTUBE_API_KEY="AIzaSyYourGeneratedApiKeyHere"
```

---

## <img src="assets/icons/folder.svg" width="22" height="22" valign="middle" alt="Project Structure" /> Project Structure

```
SocialCount/
├── Cargo.toml          # Rust package configuration and dependencies (eframe, egui, reqwest, rusqlite, serde, toml)
├── channels.toml       # Monitored YouTube channels and widget settings
├── secrets.toml        # Optional local secret credentials (ignored by Git)
├── socialcount.db      # Local SQLite database caching channel metrics and video statistics
├── LICENSE             # MIT License
├── README.md           # Project documentation
├── assets/             # Logo, social banner, vector SVG icons, and documentation screenshots
│   ├── icons/          # Vector SVG icons
│   ├── logo.png
│   ├── socialcount-banner.png
│   ├── screenshot-normal.png
│   └── screenshot-compact.png
├── examples/
│   └── generate_assets.rs # Vector and raster graphic assets generator
└── src/
    ├── main.rs         # Application entry point, CLI parser, and viewport configuration
    ├── app.rs          # egui GUI implementation, 4-counter views, details view, Win32 drag
    ├── channels.rs     # Channel data structures and delta calculations
    ├── config.rs       # TOML configuration and secrets resolution
    ├── db.rs           # Embedded SQLite engine, video caching, and tiered polling queries
    ├── icons.rs        # Pure vector rendering (YouTube badge, user, eye, video, comment, moon)
    └── youtube.rs      # YouTube API v3 client, public HTML scraper fallback, and batch syncer
```

---

## <img src="assets/icons/file-text.svg" width="22" height="22" valign="middle" alt="License" /> License

Distributed under the **MIT** License. See [`LICENSE`](LICENSE) for more information.

---

## <img src="assets/icons/user.svg" width="22" height="22" valign="middle" alt="Author" /> Author

**Matteo Baccan**
- GitHub: [@matteobaccan](https://github.com/matteobaccan)
- Repository: [https://github.com/matteobaccan/SocialCount](https://github.com/matteobaccan/SocialCount)
