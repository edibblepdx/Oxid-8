//! Bit display.

use bitvec::prelude::*;

/// Virtual screen width (64 pixels).
pub const SCREEN_WIDTH: usize = 64;

/// Virtual screen height (32 pixels).
pub const SCREEN_HEIGHT: usize = 32;

/// Virtual screen area (2048 pixels).
pub const SCREEN_AREA: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub type BitDisplay = BitArr!(for (SCREEN_AREA / 8), in u8);

/// Tightly packed screen
#[allow(dead_code)]
pub struct Display(BitDisplay);

#[allow(dead_code)]
impl Display {
    /// Create a new display.
    pub(crate) fn new() -> Self {
        Self(BitArray::ZERO)
    }

    /// Clear the screen.
    pub(crate) fn cls(&mut self) {
        self.0 = BitArray::ZERO;
    }

    /// Draw
    //pub(crate) fn drw(&mut self, x: usize, y: usize, n: u8) {}

    /// Unpacks the screen into out
    pub fn unpack_into<T: FrameBuffer>(&self, out: &mut T) {
        out.unpack(&self.0);
    }

    /// Unpacks the screen as a copy
    pub fn unpack_as<T: FromDisplay>(&self) -> T {
        T::from_display(&self.0)
    }
}

/// Trait for unpacking a BitDisplay into a new buffer.
pub trait FromDisplay {
    fn from_display(display: &BitDisplay) -> Self;
}

/// Auto trait impl for types that implement [`FrameBuffer`].
impl<T> FromDisplay for T
where
    T: FrameBuffer + Default,
{
    fn from_display(display: &BitDisplay) -> Self {
        let mut out = Self::default();
        out.unpack(display);
        out
    }
}

/// Trait for unpacking a BitDisplay into a pre-allocated buffer.
pub trait FrameBuffer {
    fn unpack(&mut self, packed: &BitDisplay);
}

/// Unpack the display into a slice of singular bool values.
/// Useful for black/white displays.
impl FrameBuffer for &mut [bool; SCREEN_AREA] {
    fn unpack(&mut self, packed: &BitDisplay) {
        for (i, px) in packed.iter().by_vals().enumerate() {
            self[i] = px;
        }
    }
}

/// Unpack the display into a slice of singular u8 values.
/// Useful for black/white displays.
impl FrameBuffer for &mut [u8; SCREEN_AREA] {
    fn unpack(&mut self, packed: &BitDisplay) {
        for (i, px) in packed.iter().by_vals().enumerate() {
            self[i] = if px { 255 } else { 0 };
        }
    }
}

/// Unpack the display into a quadruples of singular u8 values.
/// Useful for color displays.
///
/// # Example
/// ```no_run
/// let frame = display.unpack_as<[[u8; 4]; SCREEN_AREA]>().iter().flatten().collect();
/// ```
#[cfg(feature = "std")]
impl FrameBuffer for [[u8; 4]; SCREEN_AREA] {
    fn unpack(&mut self, packed: &BitDisplay) {
        for (i, px) in packed.iter().by_vals().enumerate() {
            self[i] = if px {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            };
        }
    }
}

/// Unpack the display into a vector of singular bool values.
/// Useful for black/white displays.
#[cfg(feature = "std")]
impl FrameBuffer for Vec<bool> {
    fn unpack(&mut self, packed: &BitDisplay) {
        self.clear();
        self.reserve(SCREEN_AREA);

        packed.iter().by_vals().for_each(|px| {
            self.push(px);
        });
    }
}

/// Unpack the display into a vector of singular u8 values.
/// Useful for black/white displays.
#[cfg(feature = "std")]
impl FrameBuffer for Vec<u8> {
    fn unpack(&mut self, packed: &BitDisplay) {
        self.clear();
        self.reserve(SCREEN_AREA);

        packed.iter().by_vals().for_each(|px| {
            self.push(if px { 255 } else { 0 });
        });
    }
}

/// Unpack the display into a quadruples of singular u8 values.
/// Useful for color displays.
///
/// # Example
/// ```no_run
/// let frame = display.unpack_as<Vec<[u8; 4]>>().iter().flatten().collect();
/// ```
#[cfg(feature = "std")]
impl FrameBuffer for Vec<[u8; 4]> {
    fn unpack(&mut self, packed: &BitDisplay) {
        self.clear();
        self.reserve(SCREEN_AREA);

        packed.iter().by_vals().for_each(|px| {
            self.push(if px {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            });
        });
    }
}
