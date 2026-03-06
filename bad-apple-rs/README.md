# 🍎 Bad-Apple-RS: Ultimate ASCII Video Engine

![Bad Apple React Theme](https://raw.githubusercontent.com/sjkim1127/Bad_apple_framework/main/bad_apple_react_ui_mockup.png)

A high-performance, feature-rich **Bad Apple!!** animation renderer written in Rust. This project transforms any video (or YouTube URL) into stunning ASCII art across multiple platforms: Terminal, React Web Components, and Real-time Web Streams.

---

## ✨ Key Features

### 1. 🚀 High-Performance Rendering

- **Pure Rust Core**: Lightning-fast video decoding and ASCII transformation using `rayon` for parallel processing.
- **24-bit True Color**: Supports full RGB ASCII rendering for colorful videos.
- **A/V Sync**: Perfectly synchronized audio playback with frame-skipping logic to prevent lag.

### 2. 🎮 Real-time Web Streamer (Rust-to-Web Bridge)

- **WebSocket Streaming**: Stream live ASCII frames from Rust to your browser in real-time.
- **Canvas-based Viewer**: High-performance rendering with **Emoji Mode (🍎/🍏)** and CRT filters.
- **Interactive Controls**: Toggle scanlines, change color filters (Amber, Green, Cyan), and more via a live web dashboard.

### 3. ⚛️ React Component Exporter

- **Zero-Dependency .tsx**: Generate a single standalone React file containing all frame data and audio (Base64).
- **Cyberpunk Dashboard UI**: Includes a built-in retro interface with zoom, volume, and visual effects.
- **Frame-Audio Sync**: Re-syncs animation frames to the audio `currentTime` for millisecond precision.

### 4. 📺 YouTube Streaming

- Direct playback from YouTube URLs using `yt-dlp` integration. No need to download videos manually.

---

## 🛠️ Installation & Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [FFmpeg](https://ffmpeg.org/download.html) (Ensure it's in your PATH)

### Quick Start

```powershell
# Clone the repository
git clone https://github.com/sjkim1127/Bad_apple_framework.git
cd Bad_apple_framework/bad-apple-rs

# Run in terminal (Classic)
cargo run --release -- -i badapple.mp4 -s 64:32

# Run as Real-time Web Streamer (Emoji Mode ready!)
cargo run --release -- -i badapple.mp4 --web
```

---

## 📖 CLI Usage Options

| Flag | Description |
| :--- | :--- |
| `-i, --input` | Video file path or YouTube URL. |
| `-a, --audio` | Audio file path for sync. |
| `-s, --scale` | Output resolution (e.g., `80:40`). |
| `--color` | Enable 24-bit True Color rendering. |
| `--web` | Start the real-time WebSocket web streamer. |
| `--react` | Export as a standalone React (.tsx) component. |
| `--react-output` | Specify the path for the React component. |

---

## 🎨 Visual Effects

- **Scanlines**: Emulates retro CRT monitor scanlines.
- **Bloom & Noise**: Adds organic texture and glow to the ASCII characters.
- **CRT Filter**: Choose between Classic Green, Amber, or Cyan themes.

---

## 🤝 Contribution

Feel free to open issues or submit pull requests. Let's make the best ASCII engine together!

**License**: MIT
**Author**: sjkim1127
