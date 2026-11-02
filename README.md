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
- **Solid Stat Counter Pills (Zero Emojis)**:
  - **Emerald Pill (Users / Subscribers)**: Vector user silhouette icon + clean formatted number.
  - **Sapphire Pill (Views)**: Vector eye icon with pupil and catchlight highlight + clean formatted number.
  - Equal width and responsive sizing matching window width.
  - **100% solid & opaque counter pills** regardless of chosen background transparency, ensuring maximum contrast and legibility.
- **Real-Time Change Detection**:
  - Automatically highlights counter pills with a red border whenever a channel's subscriber or view count changes.
- **Automatic Sorting by Subscribers**: Channels are dynamically ordered by subscriber count in descending order.
- **Continuous Opacity Slider**:
  - Adjust background opacity in real time from **0% (completely transparent)** to **100% opaque**. At 0% opacity, all cards and backgrounds vanish, leaving only the floating circular avatars and counter pills visible.
- **Ultra-Compact Mode (Double-Click)**:
  - **Double-click anywhere on the title bar** to instantly shrink into ultra-compact mode: header controls and footer fade away, showing only the centered avatar flanked by the two counter pills.
  - Double-click again to restore normal view.
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
  - Real-time pulsing live dot with the exact timestamp of the last successful refresh (e.g. `02/11/2026 10:45:12`).
- **Simple TOML Configuration (`channels.toml`)**:
  - Reads channels and settings from a lightweight configuration file editable on the fly without recompilation.
- **Zero API Keys Required**:
  - Public data extraction directly from YouTube via background worker threads.
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

## <img src="assets/icons/folder.svg" width="22" height="22" valign="middle" alt="Project Structure" /> Project Structure

```
SocialCount/
├── Cargo.toml          # Rust package configuration and dependencies (eframe, egui, reqwest, serde, toml, image)
├── channels.toml       # Monitored YouTube channels and widget settings
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
    ├── main.rs         # Application entry point and transparent borderless viewport configuration
    ├── app.rs          # egui GUI implementation, responsive layout, CRUD profile manager, Win32 drag
    ├── config.rs       # TOML configuration loading, normalization, and persistence
    ├── icons.rs        # Pure vector rendering (YouTube badge, user silhouette, eye, live dot)
    └── youtube.rs      # Background HTTP client, HTML parser, and subscriber counter extractor
```

---

## <img src="assets/icons/file-text.svg" width="22" height="22" valign="middle" alt="License" /> License

Distributed under the **MIT** License. See [`LICENSE`](LICENSE) for more information.

---

## <img src="assets/icons/user.svg" width="22" height="22" valign="middle" alt="Author" /> Author

**Matteo Baccan**
- GitHub: [@matteobaccan](https://github.com/matteobaccan)
- Repository: [https://github.com/matteobaccan/SocialCount](https://github.com/matteobaccan/SocialCount)
