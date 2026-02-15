//! Bit display.

use bitvec::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {

        use std::vec::Vec;

        /// Type alias for no_std compatibility.
        pub type BoolVec = Vec<bool>;
        /// Type alias for no_std compatibility.
        pub type U8Vec = Vec<u8>;

        /// Newtype over a [`std::vec::Vec`] for quadruples of [`u8`].
        #[derive(Default)]
        pub struct FlatRgba(Vec<u8>);

    } else {

        use heapless::Vec;

        /// Type alias for no_std compatibility.
        pub type BoolVec = Vec<bool, DISPLAY_AREA>;
        /// Type alias for no_std compatibility.
        pub type U8Vec = Vec<u8, DISPLAY_AREA>;

        /// Newtype over a [`heapless::Vec`] for quadruples of [`u8`].
        #[derive(Default)]
        pub struct FlatRgba(Vec<u8, { DISPLAY_AREA * 4 }>);

    }
}

/// Virtual display width (64 pixels).
pub const DISPLAY_WIDTH: usize = 64;

/// Virtual display height (32 pixels).
pub const DISPLAY_HEIGHT: usize = 32;

/// Virtual display area (2048 pixels).
pub const DISPLAY_AREA: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Type alias for a bit array.
pub type BitDisplay = BitArr!(for DISPLAY_AREA, in u8);

#[allow(dead_code)]
/// Tightly packed display.
pub struct Display(BitDisplay);

/// Provide an immutable reference to the inner [`BitDisplay`].
impl core::ops::Deref for Display {
    type Target = BitDisplay;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
impl Display {
    /// Create a new display.
    pub(crate) fn new() -> Self {
        Self(BitArray::ZERO)
    }

    /// Clear the display.
    pub(crate) fn cls(&mut self) {
        self.0 = BitArray::ZERO;
    }

    /// TODO: Draw
    pub(crate) fn drw(&mut self, x: usize, y: usize, n: u8) {}

    /// Unpacks the display into out.
    pub fn unpack_into<T: FrameBuffer>(&self, out: &mut T) {
        out.unpack(&self.0);
    }

    /// Unpacks the display as a copy.
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

/// Unpack the display into a vector of singular [`bool`] values.
/// Useful for black/white displays.
///
/// # Example
/// ```ignore
/// let fb: BoolVec = display.unpack_as();
/// ```
impl FrameBuffer for BoolVec {
    fn unpack(&mut self, packed: &BitDisplay) {
        self.clear();

        #[cfg(feature = "std")]
        self.reserve(DISPLAY_AREA);

        for px in packed.iter().by_vals() {
            // Safe to ignore result: capacity guaranteed.
            #[allow(clippy::let_unit_value)]
            let _ = self.push(px);
        }
    }
}

/// Unpack the display into a vector of singular [`u8`] values.
/// Useful for black/white displays.
///
/// # Example
/// ```ignore
/// let fb: U8Vec = display.unpack_as();
/// ```
impl FrameBuffer for U8Vec {
    fn unpack(&mut self, packed: &BitDisplay) {
        self.clear();

        #[cfg(feature = "std")]
        self.reserve(DISPLAY_AREA);

        for px in packed.iter().by_vals() {
            // Safe to ignore result: capacity guaranteed.
            #[allow(clippy::let_unit_value)]
            let _ = self.push(if px { 255 } else { 0 });
        }
    }
}

/// Unpack the display into quadruples of u8 values.
/// Useful for color displays.
///
/// # Example
/// ```ignore
/// let fb: FlatRgba = display.unpack_as();
/// ```
impl FrameBuffer for FlatRgba {
    fn unpack(&mut self, packed: &BitDisplay) {
        self.0.clear();

        #[cfg(feature = "std")]
        self.0.reserve(DISPLAY_AREA);

        for px in packed.iter().by_vals() {
            // Safe to ignore result: capacity guaranteed.
            #[allow(clippy::let_unit_value)]
            let _ = self.0.extend_from_slice(if px {
                &[255, 255, 255, 255]
            } else {
                &[0, 0, 0, 255]
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn display_as() {
        let display = Display::new();

        let a: BoolVec = display.unpack_as();
        let b: U8Vec = display.unpack_as();
        let c: FlatRgba = display.unpack_as();

        assert_eq!(a, vec![false; DISPLAY_AREA]);
        assert_eq!(b, vec![0u8; DISPLAY_AREA]);
        assert_eq!(
            c.0,
            vec![[0, 0, 0, 255]; DISPLAY_AREA]
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<u8>>()
        );
    }

    #[test]
    #[cfg(not(feature = "std"))]
    fn display_as() {
        let display = Display::new();

        let a: BoolVec = display.unpack_as();
        let b: U8Vec = display.unpack_as();
        let c: FlatRgba = display.unpack_as();

        assert_eq!(a, BoolVec::from_array([false; DISPLAY_AREA]));
        assert_eq!(b, U8Vec::from_array([0u8; DISPLAY_AREA]));

        let mut expected = FlatRgba(heapless::Vec::new());
        for _ in 0..DISPLAY_AREA {
            expected.0.extend_from_slice(&[0, 0, 0, 255]).unwrap();
        }

        assert_eq!(c.0, expected.0);
    }
}
