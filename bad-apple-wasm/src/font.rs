use std::fs::File;
use std::io::Read;

const MAXCOL: usize = 256;

pub struct Font {
    map: Box<[[u8; MAXCOL]; MAXCOL]>,
}

impl Font {
    pub fn new(font_path: &str) -> Self {
        let mut f = Self {
            map: Box::new([[b' '; MAXCOL]; MAXCOL]),
        };

        if !font_path.is_empty() {
            if let Ok(mut file) = File::open(font_path) {
                // If the user provides a raw array file (like .data)
                let mut buf = vec![0u8; MAXCOL * MAXCOL];
                if let Ok(bytes_read) = file.read(&mut buf) {
                    if bytes_read == MAXCOL * MAXCOL {
                        for i in 0..MAXCOL {
                            for j in 0..MAXCOL {
                                f.map[i][j] = buf[i * MAXCOL + j];
                            }
                        }
                        return f;
                    }
                }
            }
        }

        // Default heuristic if no external file or failed to read
        for i in 0..MAXCOL {
            for j in 0..MAXCOL {
                let up = i > 127;
                let down = j > 127;
                f.map[i][j] = match (up, down) {
                    (false, false) => b' ',
                    (false, true) => b'_', // Lower
                    (true, false) => b'^', // Upper
                    (true, true) => b'#',  // Full
                };
            }
        }

        f
    }

    pub fn get(&self, x: u8, y: u8) -> u8 {
        self.map[x as usize][y as usize]
    }
}
