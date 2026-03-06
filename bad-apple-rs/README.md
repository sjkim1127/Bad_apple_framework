# 🍎 Bad Apple!! Rust (bad-apple-rs)

> **"A high-performance, feature-rich ASCII player reborn in Rust."**

This project is a modern migration and significant enhancement of the classic "Bad Apple!!" terminal player. Built with performance in mind using Rust, it extends beyond simple grayscale to support **24-bit True Color**, direct **YouTube streaming**, and an **All-in-One Standalone** experience.

![Play Demo](play.gif)

## ✨ Key Features

* **⚡ High-Performance Rust Engine**: Optimized frame decoding and rendering ensures buttery smooth playback even at high resolutions.
* **🌈 True Color (24-bit RGB)**: Use the `--color` flag to transform simple ASCII into vibrant, full-color terminal art using ANSI 24-bit escape codes.
* **📺 YouTube Direct Streaming**: No need to download videos. Just paste a YouTube URL and play in real-time. (Automatic `yt-dlp` management included).
* **📦 100% Standalone (Portable)**: `ffmpeg` and `ffprobe` binaries are embedded directly into the executable using `include_bytes!`. No external dependencies required on the host system.
* **🔊 Native Audio & A/V Sync**: Direct audio output using `rodio` with a custom frame-skipping logic to ensure the video stays perfectly synced with the music.
* **📏 Automatic Scaling**: Intelligently detects terminal dimensions and scales the output to fit your screen while maintaining aspect ratio.

## 🚀 Getting Started

### Build from Source

Ensure you have the Rust toolchain installed.

```powershell
# Clone the repository and enter the directory
cd bad-apple-rs

# Build the project in release mode
cargo build --release

# Run (Default looks for badapple.mp4)
./target/release/bad-apple-rs.exe
```

### Usage Examples

#### 1. Stream from YouTube (Color Mode)

Stream directly from the web with full-color ASCII rendering.

```powershell
cargo run --release -- -i "https://www.youtube.com/watch?v=FtutLA63Cp8" --color
```

#### 2. Play Local File

Specify any video file and adjust playback settings.

```powershell
cargo run --release -- -i my_video.mp4 --rate 1.5 --contrast
```

#### 3. Preload (Pre-encoding)

For low-end systems, pre-encode video into a highly optimized `.badapple` ASCII data format.

```powershell
# Encode to binary data format
cargo run --release -- -i badapple.mp4 --preload

# Play the pre-encoded file (Instant startup, extremely low CPU)
cargo run --release -- -i badapple.mp4.badapple
```

## 🛠 Command Line Arguments

| Argument | Description | Default |
| :--- | :--- | :--- |
| `-i, --input` | Path to video file or YouTube URL | `badapple.mp4` |
| `--color` | **Enable 24-bit True Color mode** | `false` |
| `-r, --rate` | Playback speed (e.g., 2.0 = 2x speed) | `1.0` |
| `-s, --scale` | Global rendering scale | `1.0` |
| `--font` | Select ASCII font set | `default` |
| `--contrast` | Enhance grayscale contrast | `false` |
| `--preload` | Pre-encode to `.badapple` format instead of playing | `false` |
| `--not-clear`| Append output instead of clearing screen | `false` |

## 📐 System Requirements

* **OS**: Windows 10/11 (Optimized for Windows environment)
* **Terminal**: Windows Terminal (Recommended for best True Color experience), PowerShell, or CMD.

## 🤝 Credits

This project inherits the spirit of the original C++ implementation while leveraging Rust's safety and speed.

* **FFmpeg**: Media decoding engine
* **yt-dlp**: YouTube stream extraction
* **Rodio**: Rust native audio playback

---
Developed with ❤️ by Antigravity
