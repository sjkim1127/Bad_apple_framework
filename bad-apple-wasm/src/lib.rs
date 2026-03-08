mod font;

use font::Font;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct AsciiEngine {
    font: Font,
    frame_count: u64,
}

#[wasm_bindgen]
impl AsciiEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AsciiEngine {
        console_error_panic_hook::set_once();

        // Default font used in the original project
        let default_font = " \\\".,:;iv%xclrsneodwtuQO&0#@M";
        AsciiEngine {
            font: Font::new(default_font),
            frame_count: 0,
        }
    }

    #[wasm_bindgen]
    pub fn render_frame(
        &mut self,
        pixels: &[u8], // RGBA
        width: usize,
        height: usize,
        color: bool,
        bloom: bool,
        scanlines: bool,
        noise: bool,
    ) -> String {
        self.frame_count += 1;

        let output_width = width;
        let output_height = height / 2;

        // Preallocate string with estimated capacity (width + 1 for newline) * output_height * bytes_per_char
        let bytes_per_char = if color { 30 } else { 1 };
        let mut out = String::with_capacity((output_width + 1) * output_height * bytes_per_char);

        for j in 0..output_height {
            let mut seed = (self.frame_count ^ (j as u64)) as u32;

            // For ANSI escape codes caching
            let mut last_fg = (256u16, 256u16, 256u16);
            let mut last_bg = (256u16, 256u16, 256u16);

            for k in 0..output_width {
                let up_idx = ((j * 2) * width + k) * 4;
                // Avoid panic if height is odd and we read past the end
                let dn_idx = if (j * 2 + 1) * width + k < (width * height) {
                    ((j * 2 + 1) * width + k) * 4
                } else {
                    ((j * 2) * width + k) * 4 // Fallback
                };

                let mut up_r = pixels[up_idx];
                let mut up_g = pixels[up_idx + 1];
                let mut up_b = pixels[up_idx + 2];

                let mut dn_r = pixels[dn_idx];
                let mut dn_g = pixels[dn_idx + 1];
                let mut dn_b = pixels[dn_idx + 2];

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
                    dn_b = (dn_b as u16 * 7 / 10) as u8;
                }

                if color {
                    if bloom {
                        let up_lum = (up_r as u16 + up_g as u16 + up_b as u16) / 3;
                        let dn_lum = (dn_r as u16 + dn_g as u16 + dn_b as u16) / 3;
                        if up_lum > 180 {
                            up_r = up_r.saturating_add(30);
                            up_g = up_g.saturating_add(30);
                            up_b = up_b.saturating_add(30);
                        }
                        if dn_lum > 180 {
                            dn_r = dn_r.saturating_add(30);
                            dn_g = dn_g.saturating_add(30);
                            dn_b = dn_b.saturating_add(30);
                        }
                    }

                    // Foreground Color (Top pixel)
                    if (up_r as u16, up_g as u16, up_b as u16) != last_fg {
                        out.push_str(&format!("\x1b[38;2;{};{};{}m", up_r, up_g, up_b));
                        last_fg = (up_r as u16, up_g as u16, up_b as u16);
                    }

                    // Background Color (Bottom pixel)
                    if (dn_r as u16, dn_g as u16, dn_b as u16) != last_bg {
                        out.push_str(&format!("\x1b[48;2;{};{};{}m", dn_r, dn_g, dn_b));
                        last_bg = (dn_r as u16, dn_g as u16, dn_b as u16);
                    }

                    // Half block character
                    out.push_str("▀");
                } else {
                    // Monochrome using ASCII Font
                    // Average RGB to find brightness
                    let up_lum = ((up_r as u16 + up_g as u16 + up_b as u16) / 3) as u8;
                    let dn_lum = ((dn_r as u16 + dn_g as u16 + dn_b as u16) / 3) as u8;

                    let chr = self.font.get(up_lum, dn_lum) as char;
                    out.push(chr);
                }
            }
            if color {
                out.push_str("\x1b[0m\n");
            } else {
                out.push('\n');
            }
        }

        out
    }
}
