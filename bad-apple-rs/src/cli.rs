use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "badapple")]
#[command(version = "v2.1.4-rs")]
#[command(about = "Play the video in the console as ASCII art.", long_about = None)]
pub struct Cli {
    /// video file
    #[arg(short = 'i', long = "input", default_value = "./badapple.mp4")]
    pub input: String,

    /// [preload] output file
    #[arg(short = 'o', long = "output", default_value = "")]
    pub output: String,

    /// font data file
    #[arg(short = 'f', long = "font", default_value = "")]
    pub font: String,

    /// audio file
    #[arg(short = 'a', long = "audio", default_value = "")]
    pub audio: String,

    /// player [ffmpeg mpv mpg123 cmus]
    #[arg(short = 'p', long = "player", default_value = "")]
    pub player: String,

    /// width:height (0 means auto)
    #[arg(short = 's', long = "scale", default_value = "0:0")]
    pub scale: String,

    /// frame rate
    #[arg(short = 'r', long = "rate", default_value_t = 1024.0)]
    pub rate: f64,

    /// not clear screen (with ANSI) before each frame
    #[arg(long = "not_clear")]
    pub not_clear: bool,

    /// contrast enhancement
    #[arg(long = "contrast")]
    pub contrast: bool,

    /// color true-color rendering
    #[arg(long = "color")]
    pub color: bool,

    /// preload video (not play)
    #[arg(long = "preload")]
    pub preload: bool,

    /// [debug]
    #[arg(long = "debug")]
    pub debug: bool,

    /// ytdlp binary path (optional, will download if not found when URL used)
    #[arg(long = "ytdlp_path")]
    pub ytdlp_path: Option<String>,
}

impl Cli {
    pub fn is_url(&self) -> bool {
        self.input.starts_with("http://") || self.input.starts_with("https://")
    }
}
