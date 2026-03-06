use std::io::Read;
use std::process::{Command, Stdio, ChildStdout};

#[derive(Debug, Clone)]
pub struct VideoProperties {
    pub width: i32,
    pub height: i32,
    pub _nb_frames: i32,
    pub rate: f64,
    pub duration: f64,
}

pub struct YoutubeFetcher;

impl YoutubeFetcher {
    pub async fn fetch_stream_url(input: &str, ytdlp_path: Option<String>) -> Result<(String, String), String> {
        let exe = ytdlp_path.unwrap_or_else(|| "yt-dlp".to_string());
        
        let output = Command::new(&exe)
            .arg("-g")
            .arg("-f")
            .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best")
            .arg(input)
            .output()
            .map_err(|e| format!("Failed to run {}: {}", exe, e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("yt-dlp failed: {}", err));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.trim().lines().collect();
        
        if lines.is_empty() {
             return Err("yt-dlp returned no stream URL".to_string());
        }
        
        let video_url = lines[0].to_string();
        let audio_url = if lines.len() > 1 {
            lines[1].to_string()
        } else {
            video_url.clone()
        };
        
        Ok((video_url, audio_url))
    }
}

pub struct DecoderFFmpeg {
    video: String,
    x: i32,
    y: i32,
    xy: usize,
    stdout: Option<ChildStdout>,
    ffmpeg_path: String,
    ffprobe_path: String,
}

impl DecoderFFmpeg {
    pub fn new(video: String, ffmpeg_path: String, ffprobe_path: String) -> Self {
        Self {
            video,
            x: 0,
            y: 0,
            xy: 0,
            stdout: None,
            ffmpeg_path,
            ffprobe_path,
        }
    }

    pub fn analysis(&self) -> Result<VideoProperties, String> {
        let output = Command::new(&self.ffprobe_path)
            .arg("-v")
            .arg("quiet")
            .arg("-show_streams")
            .arg("-select_streams")
            .arg("v")
            .arg(&self.video)
            .output()
            .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

        if !output.status.success() {
            return Err("ffprobe failed".to_string());
        }

        let result_s = String::from_utf8_lossy(&output.stdout);

        let width = Self::extract_int(&result_s, "width=").ok_or("No width found")?;
        let height = Self::extract_int(&result_s, "height=").ok_or("No height found")?;
        let nb_frames = Self::extract_int(&result_s, "nb_frames=").unwrap_or(0);
        let duration = Self::extract_f64(&result_s, "duration=").unwrap_or(0.0);
        
        let rate_str = Self::extract_str(&result_s, "r_frame_rate=").ok_or("No frame rate found")?;
        let rate_parts: Vec<&str> = rate_str.split('/').collect();
        if rate_parts.len() != 2 {
            return Err("Invalid frame rate format".to_string());
        }
        let rate_l: f64 = rate_parts[0].parse().unwrap_or(0.0);
        let rate_r: f64 = rate_parts[1].parse().unwrap_or(1.0);
        if rate_r < 1.0 {
            return Err("Invalid frame rate denominator".to_string());
        }
        let rate = rate_l / rate_r;

        Ok(VideoProperties {
            width,
            height,
            _nb_frames: nb_frames,
            rate,
            duration,
        })
    }

    pub fn ready_to_read(&mut self, x: i32, y: i32, color: bool) -> Result<(), String> {
        self.x = x;
        self.y = y;
        self.xy = (x * y) as usize;

        let scale_arg = format!("scale={}:{}", x, y);
        let pix_fmt = if color { "rgb24" } else { "gray" };

        let mut child = Command::new(&self.ffmpeg_path)
            .arg("-v").arg("quiet")
            .arg("-i").arg(&self.video)
            .arg("-vf").arg(&scale_arg)
            .arg("-c:v").arg("rawvideo")
            .arg("-pix_fmt").arg(pix_fmt)
            .arg("-f").arg("rawvideo")
            .arg("-")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to build pipe: {}", e))?;

        self.stdout = child.stdout.take();
        Ok(())
    }

    pub fn read_a_frame(&mut self, buffer: &mut [u8]) -> std::io::Result<()> {
        if let Some(stdout) = &mut self.stdout {
            stdout.read_exact(buffer)?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Pipe not ready"))
        }
    }

    pub fn cls(&mut self) {
        self.stdout = None;
    }

    fn extract_str<'a>(text: &'a str, key: &str) -> Option<&'a str> {
        let p = text.find(key)?;
        let start = p + key.len();
        let end = text[start..].find('\n').map(|i| start + i)?;
        Some(text[start..end].trim())
    }

    fn extract_int(text: &str, key: &str) -> Option<i32> {
        Self::extract_str(text, key)?.parse().ok()
    }

    fn extract_f64(text: &str, key: &str) -> Option<f64> {
        Self::extract_str(text, key)?.parse().ok()
    }
}
