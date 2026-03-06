use std::fs::File;
use std::io::Read;
use crate::decoder::DecoderFFmpeg;
use crate::font::Font;

pub trait Encoder {
    fn read_a_frame(&mut self) -> Result<(), ()>;
    fn refresh_buffer(&mut self) {}
    fn buffer(&self) -> &[u8];
    fn buffer_size(&self) -> usize;
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn clk(&self) -> u64;
    fn mo(&self) -> u32;
    fn cls(&mut self) {}
}

pub struct EncoderRT {
    fnt: Font,
    decoder: DecoderFFmpeg,
    contrast: bool,
    color: bool,
    x: i32,
    y: i32,
    clk: u64,
    mo: u32,
    frame_buf: Vec<u8>,
    print_buf: Vec<u8>,
    print_size: usize,
}

impl EncoderRT {
    pub fn new(
        video: String,
        font: String,
        scale: String,
        fps: f64,
        contrast: bool,
        color: bool,
        ffmpeg_path: String,
        ffprobe_path: String,
        name: &str,
        debug: bool,
    ) -> Result<Self, String> {
        let fnt = Font::new(&font);
        let mut decoder = DecoderFFmpeg::new(video, ffmpeg_path, ffprobe_path);
        
        let vp = decoder.analysis()?;
        
        let mw = vp.width;
        let mh = vp.height;
        let mr = vp.rate;
        
        let mut mo = (0.5 + mr / fps) as u32;
        if mo == 0 {
            mo = 1;
        }
        let clk = (mo as f64 * 1.0e6 / mr) as u64; // clk in us

        let (max_x, max_y) = match crossterm::terminal::size() {
            Ok((w, h)) => (w as i32, (h as i32 - 1) * 2), // h is terminal rows, each row has 2 vertical pixels
            Err(_) => (120, 40),
        };

        let parts: Vec<&str> = scale.split(':').collect();
        let mut x = parts.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let mut y = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);

        if x > 0 {
            if y == 0 {
                y = (mh * x + (mw / 2)) / mw;
            }
        } else {
            if y > 0 {
                x = (mw * y + (mh / 2)) / mh;
            } else {
                let max_yx = (mw * max_y + (mh / 2)) / mh;
                x = max_x.min(max_yx);
                let max_xy = (mh * max_x + (mw / 2)) / mw;
                y = max_y.min(max_xy);
            }
        }

        if y % 2 != 0 {
            y += 1;
        }

        let xy = (x * y) as usize;

        if debug {
            println!("[{}:{} {:.2}Hz] -{}-> [{}:{} {:.2}Hz] {:.3}s/{}ms [debug]",
                mw, mh, mr, name,
                x, y, mr / mo as f64,
                vp.duration, clk / 1000);
        }

        decoder.ready_to_read(x, y, color)?;

        let bytes_per_char = if color { 20 } else { 1 };
        let print_size = (x as usize * bytes_per_char + 1) * (y as usize / 2);
        let frame_buf_size = if color { xy * 3 } else { xy };
        
        let frame_buf = vec![0u8; frame_buf_size];
        let mut print_buf = vec![0u8; print_size + 2];
        print_buf[print_size] = b'\n';

        Ok(Self {
            fnt,
            decoder,
            contrast,
            color,
            x,
            y,
            clk,
            mo,
            frame_buf,
            print_buf,
            print_size,
        })
    }
}

impl Encoder for EncoderRT {
    fn read_a_frame(&mut self) -> Result<(), ()> {
        self.decoder.read_a_frame(&mut self.frame_buf).map_err(|_| ())
    }

    fn refresh_buffer(&mut self) {
        if self.contrast && !self.color {
            let mut max_pixel = 0u8;
            let mut min_pixel = 255u8;
            for &p in &self.frame_buf {
                if p > max_pixel { max_pixel = p; }
                if p < min_pixel { min_pixel = p; }
            }

            if max_pixel != min_pixel {
                let range = max_pixel - min_pixel;
                for p in &mut self.frame_buf {
                    *p = ((*p - min_pixel) as u16 * 255 / range as u16) as u8;
                }
            } else {
                let fill = if max_pixel & 128 != 0 { 255 } else { 0 };
                for p in &mut self.frame_buf {
                    *p = fill;
                }
            }
        }

        let mut t = 0;
        let x = self.x as usize;
        let y = self.y as usize;

        for j in 0..(y / 2) {
            for k in 0..x {
                if self.color {
                    let up_r = self.frame_buf[((j * 2) * x + k) * 3];
                    let up_g = self.frame_buf[((j * 2) * x + k) * 3 + 1];
                    let up_b = self.frame_buf[((j * 2) * x + k) * 3 + 2];
                    
                    let dn_r = self.frame_buf[((j * 2 + 1) * x + k) * 3];
                    let dn_g = self.frame_buf[((j * 2 + 1) * x + k) * 3 + 1];
                    let dn_b = self.frame_buf[((j * 2 + 1) * x + k) * 3 + 2];

                    let up_lum = (0.299 * up_r as f32 + 0.587 * up_g as f32 + 0.114 * up_b as f32) as u8;
                    let dn_lum = (0.299 * dn_r as f32 + 0.587 * dn_g as f32 + 0.114 * dn_b as f32) as u8;
                    let ch = self.fnt.get(up_lum, dn_lum);

                    let avg_r = ((up_r as u16 + dn_r as u16) / 2) as u8;
                    let avg_g = ((up_g as u16 + dn_g as u16) / 2) as u8;
                    let avg_b = ((up_b as u16 + dn_b as u16) / 2) as u8;

                    self.print_buf[t] = b'\x1b';
                    self.print_buf[t+1] = b'[';
                    self.print_buf[t+2] = b'3'; self.print_buf[t+3] = b'8'; self.print_buf[t+4] = b';'; self.print_buf[t+5] = b'2'; self.print_buf[t+6] = b';';
                    self.print_buf[t+7] = b'0' + (avg_r / 100) % 10; self.print_buf[t+8] = b'0' + (avg_r / 10) % 10; self.print_buf[t+9] = b'0' + (avg_r % 10);
                    self.print_buf[t+10] = b';';
                    self.print_buf[t+11] = b'0' + (avg_g / 100) % 10; self.print_buf[t+12] = b'0' + (avg_g / 10) % 10; self.print_buf[t+13] = b'0' + (avg_g % 10);
                    self.print_buf[t+14] = b';';
                    self.print_buf[t+15] = b'0' + (avg_b / 100) % 10; self.print_buf[t+16] = b'0' + (avg_b / 10) % 10; self.print_buf[t+17] = b'0' + (avg_b % 10);
                    self.print_buf[t+18] = b'm';

                    self.print_buf[t+19] = ch;
                    t += 20;
                } else {
                    let up = self.frame_buf[(j * 2) * x + k];
                    let down = self.frame_buf[(j * 2 + 1) * x + k];
                    self.print_buf[t] = self.fnt.get(up, down);
                    t += 1;
                }
            }
            self.print_buf[t] = b'\n';
            t += 1;
        }
    }

    fn buffer(&self) -> &[u8] {
        &self.print_buf
    }

    fn buffer_size(&self) -> usize {
        self.print_size
    }

    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn clk(&self) -> u64 { self.clk }
    fn mo(&self) -> u32 { self.mo }

    fn cls(&mut self) {
        self.decoder.cls();
    }
}

pub struct EncoderRe {
    file: File,
    x: i32,
    y: i32,
    clk: u64,
    print_buf: Vec<u8>,
}

impl EncoderRe {
    pub fn new(video: &str, name: &str, debug: bool) -> Result<Self, String> {
        let mut file = File::open(video).map_err(|_| "Failed to open video file".to_string())?;

        let x = Self::read_int(&mut file);
        let y = Self::read_int(&mut file);
        let clk = Self::read_int(&mut file) as u64;

        Self::read_newline(&mut file);

        if debug {
            println!("[{}:{} {:.2}Hz] -{}-> [replay] [debug]",
                x, y, 1.0e6 / clk as f64, name);
        }

        let print_size = (x as usize + 1) * (y as usize);
        let print_buf = vec![0u8; print_size + 1];

        Ok(Self {
            file,
            x,
            y,
            clk,
            print_buf,
        })
    }

    fn read_int(file: &mut File) -> i32 {
        let mut c = [0u8; 1];
        let mut n;
        loop {
            if file.read_exact(&mut c).is_err() { break; }
            if c[0] >= b'0' && c[0] <= b'9' { break; }
        }
        n = (c[0] - b'0') as i32;
        loop {
            if file.read_exact(&mut c).is_err() { break; }
            if c[0] < b'0' || c[0] > b'9' { break; }
            n = n * 10 + (c[0] - b'0') as i32;
        }
        n
    }

    fn read_newline(file: &mut File) {
        let mut c = [0u8; 1];
        if file.read_exact(&mut c).is_ok() && c[0] == b'\n' {
            // Newline consumed
        }
    }
}

impl Encoder for EncoderRe {
    fn read_a_frame(&mut self) -> Result<(), ()> {
        let size = self.buffer_size() + 1;
        self.file.read_exact(&mut self.print_buf[0..size]).map_err(|_| ())
    }

    fn buffer(&self) -> &[u8] {
        &self.print_buf
    }

    fn buffer_size(&self) -> usize {
        (self.x as usize + 1) * (self.y as usize)
    }

    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn clk(&self) -> u64 { self.clk }
    fn mo(&self) -> u32 { 1 }
}
