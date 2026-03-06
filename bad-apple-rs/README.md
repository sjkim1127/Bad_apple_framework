# 🍎 Bad Apple!! Rust (bad-apple-rs)

> **"A ultra-high-performance, feature-rich ASCII player reborn in Rust."**

This project is a modern migration and significant enhancement of the classic "Bad Apple!!" terminal player. Built with performance in mind using Rust, it extends beyond simple grayscale to support **24-bit True Color**, **ASCII Shaders**, **YouTube streaming**, and **Half-Block High-Resolution** rendering.

![Play Demo](play.gif)

## ✨ Key Features

* **⚡ Parallel Rendering Engine (Rayon)**: Leverages multi-core processing to convert video frames into ASCII in parallel, ensuring smooth 60fps playback even at ultra-high resolutions.
* **▀ Half-Block High-Resolution**: Doubles vertical resolution using ANSI half-block (`▀`) characters, providing near-video quality clarity within a text terminal.
* **🎨 24-bit True Color RGB**: Full 24-bit color support. Use the `--color` flag for vibrant, stunning terminal art.
* **✨ ASCII Shaders & Post-processing**:
  * **Scanlines**: Authentic CRT-style monitor effects.
  * **Noise/Glitch**: Cyberpunk-style grain and texture.
  * **Bloom**: Elegant glow effects around bright areas using background color bleed.
* **📺 YouTube Direct Streaming**: Stream directly from the web. Just paste a YouTube URL and play in real-time. (Automatic `yt-dlp` management included).
* **📦 100% Standalone (Portable)**: `ffmpeg` and `ffprobe` binaries are embedded directly into the executable. No external dependencies required.
* **🔊 Native A/V Sync**: Direct audio output using `rodio` with a custom frame-skipping logic to ensure zero-lag synchronization.

## 🚀 Performance Optimizations

This player is engineered for maximum speed:

* **Zero-allocation Formatting**: Replaced `format!` with high-speed integer-to-string conversion, eliminating thousands of heap allocations per frame.
* **Persistent I/O Locking**: Locks `stdout` once per session and uses a massive `BufWriter` (1MB) to flood the terminal with data efficiently.
* **Memory Reuse**: Dedicated row buffers are pre-allocated and reused, effectively reaching constant memory usage during playback.

## 🚀 Getting Started

### Build from Source

Ensure you have the Rust toolchain installed.

> [!IMPORTANT]
> To build the standalone version:
>
> 1. Create a `bin` folder in the project root.
> 2. Copy `ffmpeg.exe` and `ffprobe.exe` into the `bin` folder.
> 3. Run: `cargo build --release`

```powershell
# Build and run with all the bells and whistles
cargo run --release -- -i "https://youtu.be/7gxkOp7R6jc" --color --bloom --scanlines
```

## 🛠 Command Line Arguments

| Argument | Description | Default |
| :--- | :--- | :--- |
| `-i, --input` | Path to video file or YouTube URL | `badapple.mp4` |
| `--color` | **Enable 24-bit True Color mode** | `false` |
| `--scanlines` | Enable CRT-style scanline effect | `false` |
| `--noise` | Enable grain/noise effect | `false` |
| `--bloom` | Enable glow/light bleed effect | `false` |
| `-r, --rate` | Playback speed (e.g., 2.0 = 2x speed) | `1.0` |
| `--contrast` | Enhance grayscale contrast | `false` |
| `--preload` | Pre-encode to `.badapple` format | `false` |
| `--not-clear` | Append output instead of clearing screen | `false` |

## 📐 System Requirements

* **OS**: Windows 10/11 (Optimized for Windows environment)
* **Terminal**: **Windows Terminal** (Highly recommended for True Color & HQ), PowerShell, or CMD.

## 🤝 Credits

* **Rayon**: Parallel processing engine.
* **FFmpeg**: Media decoding engine.
* **yt-dlp**: YouTube stream extraction.
* **Rodio**: Rust native audio playback.

---
Developed with ❤️ by Antigravity (Advanced Agentic Coding @ DeepMind)
