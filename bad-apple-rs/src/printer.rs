use std::time::{Instant, Duration};
use std::thread;
use std::io::{Write, stdout, StdoutLock, BufWriter};

pub struct Timer {
    clk_ns: u64,
    start: Option<Instant>,
    bg_start: Option<Instant>,
}

impl Timer {
    pub fn new(clk_us: u64) -> Self {
        Self {
            clk_ns: clk_us * 1000,
            start: None,
            bg_start: None,
        }
    }

    pub fn bg(&mut self) {
        let now = Instant::now();
        self.bg_start = Some(now);
        self.start = Some(now);
    }

    /// Returns the number of frames that should have passed since start.
    /// Used for frame skipping to maintain sync with audio.
    pub fn get_target_frame(&self) -> u64 {
        if let Some(bg) = self.bg_start {
            let elapsed = bg.elapsed().as_nanos() as u64;
            elapsed / self.clk_ns
        } else {
            0
        }
    }

    pub fn wait(&mut self) {
        if let Some(start) = self.start {
            let target = Duration::from_nanos(self.clk_ns);
            let now = Instant::now();
            
            if now < start + target {
                thread::sleep((start + target).duration_since(now));
            }
            
            // Sync start to current frame slot
            self.start = Some(start + target);
        }
    }
}

pub struct Printer {
    timer: Timer,
    not_clear: bool,
    writer: BufWriter<StdoutLock<'static>>,
}

impl Printer {
    pub fn new(clk_us: u64, not_clear: bool) -> Self {
        let mut timer = Timer::new(clk_us);
        
        let stdout_obj = Box::leak(Box::new(stdout()));
        let mut lock = stdout_obj.lock();
        
        if not_clear {
            writeln!(lock).unwrap();
        } else {
            write!(lock, "\x1b[?25l\x1b[2J\x1b[H").unwrap(); // Hide cursor, clear screen, move home
        }
        lock.flush().unwrap();
        
        timer.bg();
        
        Self {
            timer,
            not_clear,
            writer: BufWriter::with_capacity(1024 * 1024, lock), // 1MB Buffer
        }
    }

    pub fn print_a_frame(&mut self, buffer: &[u8]) {
        if !self.not_clear {
            let _ = write!(self.writer, "\x1b[H"); // Move home
        }
        let _ = self.writer.write_all(buffer);
        let _ = self.writer.flush();
        
        self.timer.wait();
    }

    pub fn get_target_frame(&self) -> u64 {
        self.timer.get_target_frame()
    }
}

use std::fs::File;

pub struct Preloader {
    fp: File,
}

impl Preloader {
    pub fn new(output: &str, x: i32, y: i32, clk: u64) -> Result<Self, String> {
        let mut fp = File::create(output).map_err(|e| format!("Open output file failed: {}", e))?;
        
        // Write header: "width (height/2) clk\n"
        write!(fp, "{} {} {}\n", x, y / 2, clk).map_err(|e| e.to_string())?;
        
        Ok(Self { fp })
    }

    pub fn print_a_frame(&mut self, buffer: &[u8]) {
        let _ = self.fp.write_all(buffer);
    }
}
