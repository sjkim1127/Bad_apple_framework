mod cli;
mod decoder;
mod font;
mod encoder;
mod audio;
mod printer;

use clap::Parser;
use std::process;
use std::path::Path;

use cli::Cli;
use encoder::{Encoder, EncoderRT, EncoderRe};
use audio::AudioPlayer;
use printer::{Printer, Preloader};
use std::process::{Command, Stdio};


use crate::decoder::YoutubeFetcher;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Check if input is a URL
    let mut video_input = cli.input.clone();
    let mut audio_input = cli.audio.clone();
    if cli.is_url() {
        // Find or Download yt-dlp
        let ytdlp_exe = match find_or_download_ytdlp(cli.ytdlp_path.clone().as_deref()).await {
             Ok(p) => p,
             Err(e) => {
                  eprintln!("Error getting yt-dlp: {}", e);
                  process::exit(1);
             }
        };
        
        println!("Fetching stream URL from YouTube/URL...");
        match YoutubeFetcher::fetch_stream_url(&cli.input, Some(ytdlp_exe)).await {
            Ok((v_url, a_url)) => {
                video_input = v_url;
                audio_input = a_url;
            },
            Err(e) => {
                eprintln!("YouTube error: {}", e);
                process::exit(1);
            }
        }
    } else if !Path::new(&cli.input).exists() {
        eprintln!("Open video file failed.");
        process::exit(1);
    }

    let mut preload = cli.preload;
    let mut output = cli.output.clone();
    
    if !output.is_empty() {
        preload = true;
    } else if preload {
        output = format!("{}.badapple", cli.input);
    }

    let (ffmpeg_path, ffprobe_path) = find_ffmpeg_tools();

    // Deferred initialization
    let mut clplayer: Option<AudioPlayer> = None;
    let mut outer_printer: Option<Printer> = None;
    let mut outer_preload: Option<Preloader> = None;

    let is_badapple_ext = cli.input.ends_with(".badapple");
    
    let mut enc: Box<dyn Encoder> = if is_badapple_ext {
        if preload {
            eprintln!("Video file is already preloaded.");
            process::exit(1);
        }
        match EncoderRe::new(&video_input, "badapple", cli.debug) {
            Ok(e) => Box::new(e),
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    } else {
        match EncoderRT::new(
            video_input.clone(),
            cli.font.clone(),
            cli.scale.clone(),
            cli.rate,
            cli.contrast,
            cli.color,
            cli.scanlines,
            cli.noise,
            cli.bloom,
            ffmpeg_path.clone(),
            ffprobe_path.clone(),
            "FFmpeg",
            cli.debug,
        ) {
            Ok(e) => Box::new(e),
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    };
    
    // Core playback remains essentially the same.
    // ...

    let clk = enc.clk();
    let enc_x = enc.x();
    let enc_y = enc.y();

    let mo = enc.mo();

    let mut i = 0;
    let mut frame_sent = 0;
    loop {
        if enc.read_a_frame().is_err() {
            if i == 0 {
                eprintln!("The first frame is empty.");
                process::exit(1);
            }
            break;
        }

        // Initialize Audio and Timer on the very first frame to ensure perfect sync
        if i == 0 {
            if !preload {
                clplayer = Some(AudioPlayer::new(&video_input, audio_input.clone(), cli.player.clone(), false, &ffmpeg_path));
                outer_printer = Some(Printer::new(clk, cli.not_clear));
            } else {
                outer_preload = Some(Preloader::new(&output, enc_x, enc_y, clk).unwrap());
            }
        }

        if i % mo != 0 {
            i += 1;
            continue;
        }
        
        // A/V Sync: If video is lagging behind the timer, skip rendering/printing
        if let Some(ref p) = outer_printer {
            let target_frame = p.get_target_frame();
            if target_frame > frame_sent + 1 {
                frame_sent += 1;
                i += 1;
                continue;
            }
        }

        enc.refresh_buffer();
        
        let buf_size = enc.buffer_size();
        let buf = enc.buffer();

        if let Some(ref mut p) = outer_preload {
            p.print_a_frame(&buf[0..buf_size]);
        } else if let Some(ref mut p) = outer_printer {
            p.print_a_frame(&buf[0..buf_size]);
        }
        
        frame_sent += 1;
        i += 1;
    }

    if let Some(ref mut _p) = outer_printer {
         // Show cursor before exit
         print!("\x1b[?25h");
    }

    enc.cls();
    if let Some(mut p) = clplayer {
        p.terminate();
    }
}


const BUNDLED_FFMPEG: &[u8] = include_bytes!("..\\bin\\ffmpeg.exe");
const BUNDLED_FFPROBE: &[u8] = include_bytes!("..\\bin\\ffprobe.exe");

async fn find_or_download_ytdlp(path_hint: Option<&str>) -> Result<String, String> {
    if let Some(p) = path_hint {
        if Path::new(p).exists() {
            return Ok(p.to_string());
        }
    }

    if Command::new("yt-dlp").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok() {
        return Ok("yt-dlp".to_string());
    }

    let mut exe_path = dirs::cache_dir().unwrap_or_else(|| Path::new(".").to_path_buf());
    exe_path.push("yt-dlp.exe");

    if exe_path.exists() {
        return Ok(exe_path.to_string_lossy().to_string());
    }

    println!("yt-dlp not found. Downloading to {:?}...", exe_path);
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
    let response = reqwest::get(url).await.map_err(|e| format!("Download error: {}", e))?;
    let content = response.bytes().await.map_err(|e| format!("Content error: {}", e))?;
    std::fs::write(&exe_path, &content).map_err(|e| format!("Write error: {}", e))?;
    Ok(exe_path.to_string_lossy().to_string())
}

fn find_ffmpeg_tools() -> (String, String) {
    // 1. Check system path first
    if Command::new("ffmpeg").arg("-version").stdout(Stdio::null()).status().is_ok() {
        if Command::new("ffprobe").arg("-version").stdout(Stdio::null()).status().is_ok() {
            return ("ffmpeg".to_string(), "ffprobe".to_string());
        }
    }

    // 2. Check current/cache folder
    let base_path = dirs::cache_dir().unwrap_or_else(|| Path::new(".").to_path_buf());
    
    let ffmpeg_exe = base_path.join("ffmpeg.exe");
    let ffprobe_exe = base_path.join("ffprobe.exe");

    if !ffmpeg_exe.exists() || !ffprobe_exe.exists() {
        println!("Extracting bundled FFmpeg tools to {:?}...", base_path);
        let _ = std::fs::write(&ffmpeg_exe, BUNDLED_FFMPEG);
        let _ = std::fs::write(&ffprobe_exe, BUNDLED_FFPROBE);
    }

    if ffmpeg_exe.exists() && ffprobe_exe.exists() {
        return (ffmpeg_exe.to_string_lossy().to_string(), ffprobe_exe.to_string_lossy().to_string());
    }

    // 3. Fallback to existing logic for sibling folders if extraction fails
    let sibling_ffmpeg = "..\\Bad-Apple\\win_dep_bin\\ffmpeg\\ffmpeg.exe";
    let sibling_ffprobe = "..\\Bad-Apple\\win_dep_bin\\ffmpeg\\ffprobe.exe";
    if Path::new(sibling_ffmpeg).exists() && Path::new(sibling_ffprobe).exists() {
        return (sibling_ffmpeg.to_string(), sibling_ffprobe.to_string());
    }

    ("ffmpeg".to_string(), "ffprobe".to_string())
}
