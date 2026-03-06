use std::fs::File;
use std::io::Read;
use crate::decoder::DecoderFFmpeg;
use crate::font::Font;
use rayon::prelude::*;

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
    scanlines: bool,
    noise: bool,
    bloom: bool,
    frame_count: u64,
    row_bufs: Vec<Vec<u8>>,
}

impl EncoderRT {
    pub fn new(
        video: String,
        font: String,
        scale: String,
        fps: f64,
        contrast: bool,
        color: bool,
        scanlines: bool,
        noise: bool,
        bloom: bool,
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

        let bytes_per_char = if color { 
            if bloom { 44 } else { 20 } 
        } else { 1 };
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
            print_buf: Vec::new(),
            print_size: 0,
            scanlines,
            noise,
            bloom,
            frame_count: 0,
            row_bufs: (0..(y / 2)).map(|_| Vec::with_capacity(x as usize * 40)).collect(),
        })
    }

    fn write_u8(vec: &mut Vec<u8>, mut n: u8) {
        if n >= 100 {
            vec.push(b'0' + (n / 100));
            n %= 100;
            vec.push(b'0' + (n / 10));
            vec.push(b'0' + (n % 10));
        } else if n >= 10 {
            vec.push(b'0' + (n / 10));
            vec.push(b'0' + (n % 10));
        } else {
            vec.push(b'0' + n);
        }
    }
}

impl Encoder for EncoderRT {
    fn read_a_frame(&mut self) -> Result<(), ()> {
        self.frame_count += 1;
        self.decoder.read_a_frame(&mut self.frame_buf).map_err(|_| ())
    }

    fn refresh_buffer(&mut self) {
        if self.contrast && !self.color {
            let (max_pixel, min_pixel) = self.frame_buf.par_iter().fold(
                || (0u8, 255u8),
                |(max_p, min_p), &p| (max_p.max(p), min_p.min(p))
            ).reduce(
                || (0u8, 255u8),
                |(max1, min1), (max2, min2)| (max1.max(max2), min1.min(min2))
            );

            if max_pixel != min_pixel {
                let range = max_pixel - min_pixel;
                self.frame_buf.par_iter_mut().for_each(|p| {
                    *p = ((*p - min_pixel) as u16 * 255 / range as u16) as u8;
                });
            } else {
                let fill = if max_pixel & 128 != 0 { 255 } else { 0 };
                self.frame_buf.par_iter_mut().for_each(|p| {
                    *p = fill;
                });
            }
        }

        let x = self.x as usize;
        let y = self.y as usize;
        let color = self.color;
        let scanlines = self.scanlines;
        let noise = self.noise;
        let bloom = self.bloom;
        let frame_count = self.frame_count;

        let bytes_per_pixel = if color { 3 } else { 1 };
        let _bytes_per_char_hint = if color { if bloom { 45 } else { 21 } } else { 2 };

        // Process rows in parallel using persistent buffers
        let frame_buf = &self.frame_buf;
        let fnt = &self.fnt;

        self.row_bufs.par_iter_mut().enumerate().for_each(|(j, row)| {
            row.clear();
            let mut seed = (frame_count ^ (j as u64)) as u32;

            // Cache previous colors to skip redundant ANSI codes
            let mut last_fg = (256u16, 256u16, 256u16);
            let mut last_bg = (256u16, 256u16, 256u16);

            for k in 0..x {
                if color {
                    let up_idx = ((j * 2) * x + k) * bytes_per_pixel;
                    let dn_idx = ((j * 2 + 1) * x + k) * bytes_per_pixel;

                    let mut up_r = frame_buf[up_idx];
                    let mut up_g = frame_buf[up_idx + 1];
                    let mut up_b = frame_buf[up_idx + 2];
                    
                    let mut dn_r = frame_buf[dn_idx];
                    let mut dn_g = frame_buf[dn_idx + 1];
                    let mut dn_b = frame_buf[dn_idx + 2];

                    if noise {
                        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                        let noise_val = (seed % 30) as i16 - 15;
                        up_r = (up_r as i16 + noise_val).clamp(0, 255) as u8;
                        up_g = (up_g as i16 + noise_val).clamp(0, 255) as u8;
                        up_b = (up_b as i16 + noise_val).clamp(0, 255) as u8;
                        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                        let noise_val_dn = (seed % 30) as i16 - 15;
                        dn_r = (dn_r as i16 + noise_val_dn).clamp(0, 255) as u8;
                        dn_g = (dn_g as i16 + noise_val_dn).clamp(0, 255) as u8;
                        dn_b = (dn_b as i16 + noise_val_dn).clamp(0, 255) as u8;
                    }

                    if scanlines && j % 2 == 0 {
                        up_r = (up_r as u16 * 7 / 10) as u8;
                        up_g = (up_g as u16 * 7 / 10) as u8;
                        up_b = (up_b as u16 * 7 / 10) as u8;
                        dn_r = (dn_r as u16 * 7 / 10) as u8;
                        dn_g = (dn_g as u16 * 7 / 10) as u8;
                        dn_b = (dn_b as i16 * 7 / 10) as u8;
                    }

                    // Bloom effect integration: Boost colors if bright
                    if bloom {
                        let up_lum = (up_r as u16 + up_g as u16 + up_b as u16) / 3;
                        let dn_lum = (dn_r as u16 + dn_g as u16 + dn_b as u16) / 3;
                        if up_lum > 180 {
                            up_r = up_r.saturating_add(30); up_g = up_g.saturating_add(30); up_b = up_b.saturating_add(30);
                        }
                        if dn_lum > 180 {
                            dn_r = dn_r.saturating_add(30); dn_g = dn_g.saturating_add(30); dn_b = dn_b.saturating_add(30);
                        }
                    }

                    // Set Foreground Color (Top pixel)
                    if (up_r as u16, up_g as u16, up_b as u16) != last_fg {
                        row.extend_from_slice(b"\x1b[38;2;");
                        Self::write_u8(row, up_r);
                        row.push(b';');
                        Self::write_u8(row, up_g);
                        row.push(b';');
                        Self::write_u8(row, up_b);
                        row.push(b'm');
                        last_fg = (up_r as u16, up_g as u16, up_b as u16);
                    }

                    // Set Background Color (Bottom pixel)
                    if (dn_r as u16, dn_g as u16, dn_b as u16) != last_bg {
                        row.extend_from_slice(b"\x1b[48;2;");
                        Self::write_u8(row, dn_r);
                        row.push(b';');
                        Self::write_u8(row, dn_g);
                        row.push(b';');
                        Self::write_u8(row, dn_b);
                        row.push(b'm');
                        last_bg = (dn_r as u16, dn_g as u16, dn_b as u16);
                    }

                    // Print Half block character
                    row.extend_from_slice("▀".as_bytes());
                } else {
                    let mut up = frame_buf[(j * 2) * x + k];
                    let mut down = frame_buf[(j * 2 + 1) * x + k];

                    if noise {
                        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                        let noise_val = (seed % 20) as i16 - 10;
                        up = (up as i16 + noise_val).clamp(0, 255) as u8;
                        down = (down as i16 + noise_val).clamp(0, 255) as u8;
                    }

                    if scanlines && j % 2 == 0 {
                        up = (up as u16 * 7 / 10) as u8;
                        down = (down as u16 * 7 / 10) as u8;
                    }

                    row.push(fnt.get(up, down));
                }
            }
            // Reset colors at the end of each row to prevent bleeding
            row.extend_from_slice(b"\x1b[0m\n");
        });

        // Flatten all rows into the main print buffer
        self.print_buf.clear();
        for row in &self.row_bufs {
            self.print_buf.extend_from_slice(row);
        }
        self.print_size = self.print_buf.len();
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
