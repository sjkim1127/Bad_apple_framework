use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rodio::{Decoder, OutputStream, Sink};

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
}

impl AudioPlayer {
    pub fn new(video: &str, mut audio: String, _player: String, preload: bool, ffmpeg_path: &str) -> Self {
        if preload {
            return Self { _stream: None, sink: None };
        }

        let is_url = audio.starts_with("http://") || audio.starts_with("https://");

        if audio.is_empty() {
            // Try to look for .wav if available
            let wav_path = "badapple.wav";
            if Path::new(wav_path).exists() {
                audio = wav_path.to_string();
            } else {
                audio = video.to_string();
            }
        } else if !is_url {
            if !Path::new(&audio).exists() {
                audio = video.to_string();
            }
        }

        if audio.is_empty() {
            return Self { _stream: None, sink: None };
        }

        // Try to initialize Rodio
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (s, h),
            Err(e) => {
                eprintln!("Failed to initialize audio device: {}", e);
                return Self { _stream: None, sink: None };
            }
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create audio sink: {}", e);
                return Self { _stream: None, sink: None };
            }
        };

        if is_url {
            use std::process::Command;
            let temp_audio = "temp_youtube_audio.wav";
            let _ = Command::new(ffmpeg_path)
                .arg("-v").arg("quiet")
                .arg("-y")
                .arg("-i").arg(&audio)
                .arg("-vn")
                .arg("-acodec").arg("pcm_s16le")
                .arg("-ar").arg("44100")
                .arg("-ac").arg("2")
                .arg(temp_audio)
                .status();
            
            if let Ok(file) = File::open(temp_audio) {
                let buf_reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(buf_reader) {
                    sink.append(source);
                    sink.play();
                    
                    return Self {
                        _stream: Some(_stream),
                        sink: Some(sink),
                    };
                }
            }
        } else if let Ok(file) = File::open(&audio) {
            let buf_reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(buf_reader) {
                // To avoid issues with certain sample rates causing playback errors
                sink.append(source);
                sink.play();
                
                return Self {
                    _stream: Some(_stream),
                    sink: Some(sink),
                };
            } else {
                eprintln!("Failed to decode audio from: {}", audio);
            }
        } else {
            eprintln!("Failed to open audio file: {}", audio);
        }

        Self { _stream: None, sink: None }
    }

    pub fn terminate(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }
}

