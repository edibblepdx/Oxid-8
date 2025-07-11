/// Tightly packed screen
#[allow(dead_code)]
pub struct Screen {
    pixels: [u8; 256],
    pub draw: bool,
}

#[allow(dead_code)]
impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: [0; 256],
            draw: false,
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [0; 256];
        self.draw = true;
    }

    pub fn draw(&mut self, x: usize, y: usize, n: u8) {}

    /// Unpacks the screen into out
    pub fn unpack_into<T>(&self, out: &mut T)
    where
        T: BitUnpacker,
    {
        out.unpack(&self.pixels);
    }
}

fn bits(byte: u8) -> impl Iterator<Item = u8> {
    (0..8).rev().map(move |i| (byte >> i) & 1)
}

pub trait BitUnpacker {
    fn unpack(&mut self, packed: &[u8; 256]);
}

impl BitUnpacker for Vec<u8> {
    fn unpack(&mut self, packed: &[u8; 256]) {
        self.clear();
        self.reserve(2048);

        for &byte in packed {
            for bit in bits(byte) {
                self.push(bit * 255)
            }
        }
    }
}

impl BitUnpacker for Vec<bool> {
    fn unpack(&mut self, packed: &[u8; 256]) {
        self.clear();
        self.reserve(2048);

        for &byte in packed {
            for bit in bits(byte) {
                self.push(if bit != 0 { true } else { false })
            }
        }
    }
}
