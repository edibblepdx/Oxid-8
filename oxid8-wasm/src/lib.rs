use oxid8_core::{Oxid8, SCREEN_AREA, SCREEN_HEIGHT, SCREEN_WIDTH};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Framebuffer {
    buffer: [u8; SCREEN_AREA], // monochrome
}

impl Default for Framebuffer {
    fn default() -> Self {
        Framebuffer {
            buffer: [0u8; SCREEN_AREA],
        }
    }
}

#[wasm_bindgen]
impl Framebuffer {
    /// Return pointer to framebuffer.
    #[wasm_bindgen]
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    /// Return frame area.
    #[wasm_bindgen(getter)]
    pub fn area() -> usize {
        SCREEN_AREA
    }

    /// Return frame width.
    #[wasm_bindgen(getter)]
    pub fn width() -> usize {
        SCREEN_WIDTH
    }

    /// Return frame height.
    #[wasm_bindgen(getter)]
    pub fn height() -> usize {
        SCREEN_HEIGHT
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Wasm8 {
    pub frame: Framebuffer,
    core: Oxid8,
}

#[wasm_bindgen]
impl Wasm8 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Wasm8::default()
    }

    /// Draw to the framebuffer.
    pub fn draw(&mut self) {
        for (i, &p) in self.core.screen_ref().iter().enumerate() {
            self.frame.buffer[i] = if p { 255 } else { 0 };
        }
    }

    /// Emulate a CPU cycle.
    pub fn run_cycle(&mut self) -> Result<(), String> {
        self.core.run_cycle()
    }

    /// Decrement the delay and sound and timers.
    pub fn dec_timers(&mut self) {
        self.core.dec_timers();
    }

    /// Returns true if sound timer is not zero.
    pub fn sound(&self) -> bool {
        self.core.sound()
    }

    /// Set a key on the virtual keypad.
    pub fn set_key(&mut self, k: usize, val: bool) {
        self.core.set_key(k, val);
    }

    /// Clear the virtual keypad.
    pub fn clear_keys(&mut self) {
        self.core.clear_keys();
    }

    /// Instruct the interpreter to load the fontset.
    pub fn load_font(&mut self) {
        self.core.load_font();
    }

    /// Instruct the interpreter to load a rom from filename.
    // WARN: ignoring `Result`
    pub fn load_rom(&mut self, filename: &str) {
        let _ = self.core.load_rom(filename);
    }
}
