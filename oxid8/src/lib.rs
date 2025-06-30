use rand::{Rng, rng, rngs::ThreadRng};
use std::{fmt, fs, io, time::Duration};

/// Standard CPU tick rate set to 700Hz. This value is not used internally.
pub const CPU_TICK: Duration = Duration::from_micros(1430);
/// Standard TIMER tick rate set to 60Hz. This value is not used internally.
pub const TIMER_TICK: Duration = Duration::from_micros(16667);

// Source for font and constants:
// https://aquova.net/emudev/chip8/
const FONTSET_SIZE: usize = 80;
const FONT_ADDR: u16 = 0x050;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const VF: usize = 15;
const START_ADDR: u16 = 0x200;

#[derive(Debug)]
struct Opcode(u8, u8, u8, u8);

// struct Oxid8 source modified:
// https://aquova.net/emudev/chip8/
// All methods are original
#[derive(Debug)]
pub struct Oxid8 {
    pc: u16,                                      // Program Counter
    ram: [u8; RAM_SIZE],                          // RAM
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // Monochrome Display
    v_reg: [u8; NUM_REGS],                        // 8-bit V Registers
    i_reg: u16,                                   // 16[12]-bit I Register
    sp: u16,                                      // Stack Pointer
    stack: [u16; STACK_SIZE],                     // Stack
    keys: [bool; NUM_KEYS],                       // Keys (0-F)
    dt: u8,                                       // Delay Timer
    st: u8,                                       // Sound Timer
    rng: ThreadRng,                               // RNG
}

impl Opcode {
    fn new(byte1: u8, byte2: u8) -> Self {
        Self(
            (byte1 & 0xF0) >> 4,
            byte1 & 0x0F,
            (byte2 & 0xF0) >> 4,
            byte2 & 0x0F,
        )
    }

    /// A 16-bit value, the whole instruction.
    fn full(&self) -> u16 {
        (self.0 as u16) << 12 | (self.1 as u16) << 8 | (self.2 as u16) << 4 | (self.3 as u16)
    }

    /// A 12-bit value, the lowest 12 bits of the instruction.
    fn nnn(&self) -> u16 {
        (self.1 as u16) << 8 | (self.2 as u16) << 4 | (self.3 as u16)
    }

    /// A 4-bit value, the lowest 4 bits of the instruction.
    fn n(&self) -> u8 {
        self.3
    }

    /// A 4-bit value, the lower 4 bits of the high byte of the instruction.
    fn x(&self) -> u8 {
        self.1
    }

    /// A 4-bit value, the upper 4 bits of the low byte of the instruction.
    fn y(&self) -> u8 {
        self.2
    }

    /// An 8-bit value, the lowest 8 bits of the instruction.
    fn kk(&self) -> u8 {
        self.2 << 4 | self.3
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.0, self.1, self.2, self.3)
    }
}

/// Oxid8 Core
impl Oxid8 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emulates a single cycle.
    pub fn run_cycle(&mut self) -> Result<(), String> {
        // TODO: fix return type
        let opcode = Opcode::new(
            self.ram[self.pc as usize],     //
            self.ram[self.pc as usize + 1], //
        );

        let pc_at_err = self.pc;
        self.pc += 2;

        let invalid = || -> Result<(), String> {
            Err(format!(
                "Invalid Instruction: {:04X} at {}",
                opcode.full(),
                pc_at_err,
            ))
        };

        match opcode.0 {
            0x0 => match opcode.kk() {
                0xE0 => self.cls(),
                0xEE => self.ret(),
                _ => invalid()?,
            },
            0x1 => self.jp_nnn(opcode.nnn()),
            0x2 => self.call(opcode.nnn()),
            0x3 => self.se_xkk(opcode.x() as usize, opcode.kk()),
            0x4 => self.sne_xkk(opcode.x() as usize, opcode.kk()),
            0x5 => self.se_xy(opcode.x() as usize, opcode.y() as usize),
            0x6 => self.ld_xkk(opcode.x() as usize, opcode.kk()),
            0x7 => self.add_xkk(opcode.x() as usize, opcode.kk()),
            0x8 => match opcode.n() {
                0x0 => self.ld_xy(opcode.x() as usize, opcode.y() as usize),
                0x1 => self.or(opcode.x() as usize, opcode.y() as usize),
                0x2 => self.and(opcode.x() as usize, opcode.y() as usize),
                0x3 => self.xor(opcode.x() as usize, opcode.y() as usize),
                0x4 => self.add_xy(opcode.x() as usize, opcode.y() as usize),
                0x5 => self.sub_xy(opcode.x() as usize, opcode.y() as usize),
                0x6 => self.shr(opcode.x() as usize, opcode.y() as usize),
                0x7 => self.subn_xy(opcode.x() as usize, opcode.y() as usize),
                0xE => self.shl(opcode.x() as usize, opcode.y() as usize),
                _ => invalid()?,
            },
            0x9 => self.sne_xy(opcode.x() as usize, opcode.y() as usize),
            0xA => self.ld_innn(opcode.nnn()),
            0xB => self.jp_0nnn(opcode.nnn()),
            0xC => self.rnd(opcode.x() as usize, opcode.kk()),
            0xD => {
                self.drw(
                    opcode.x() as usize, //
                    opcode.y() as usize, //
                    opcode.n(),          //
                );
            }
            0xE => match opcode.kk() {
                0x9E => self.skp(opcode.x() as usize),
                0xA1 => self.sknp(opcode.x() as usize),
                _ => invalid()?,
            },
            0xF => match opcode.kk() {
                0x07 => self.ld_xdt(opcode.x() as usize),
                0x0A => self.ld_xk(opcode.x() as usize),
                0x15 => self.ld_dtx(opcode.x() as usize),
                0x18 => self.ld_stx(opcode.x() as usize),
                0x1E => self.add_ix(opcode.x() as usize),
                0x29 => self.ld_fx(opcode.x() as usize),
                0x33 => self.ld_bx(opcode.x() as usize),
                0x55 => self.ld_ix(opcode.x() as usize),
                0x65 => self.ld_xi(opcode.x() as usize),
                _ => invalid()?,
            },
            _ => invalid()?,
        }
        Ok(())
    }

    /// Decrements the delay and sound and timers.
    pub fn dec_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    /// Returns true if sound timer is zero.
    pub fn sound(&self) -> bool {
        self.st != 0
    }

    /// Sets a key on the virtual keypad.
    ///
    /// # Panics
    ///
    /// If key out of bounds.
    /// Expects 0x0 - 0xF (0 - 15)
    pub fn set_key(&mut self, k: usize, val: bool) {
        // WARN: will panic if key out of bounds
        self.keys[k] = val;
    }

    /// Clears the virtual keypad.
    pub fn clear_keys(&mut self) {
        self.keys = [false; NUM_KEYS];
    }

    /// Returns a reference to the screen.
    pub fn screen_ref(&self) -> &[bool] {
        &self.screen
    }

    /// Instructs the interpreter to load the fontset.
    pub fn load_font(&mut self) {
        self.ram[FONT_ADDR as usize..(FONT_ADDR as usize + FONTSET_SIZE)] //
            .copy_from_slice(&FONTSET);
    }

    /// Loads a rom given a filename.
    pub fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        // TODO: use pathbuf instead
        let rom: Vec<u8> = fs::read(filename)?;
        let len = rom.len();
        if len > (RAM_SIZE - START_ADDR as usize) {
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                format!("ROM too large: {}", len),
            ));
        }

        self.ram[START_ADDR as usize..(START_ADDR as usize + len)] //
            .copy_from_slice(rom.as_slice());

        Ok(())
    }

    fn push(&mut self, val: u16) {
        match self.sp as usize {
            0..STACK_SIZE => {
                self.stack[self.sp as usize] = val;
                self.sp += 1;
            }
            _ => panic!("ERROR::Emulator Stack Overflow"),
        };
    }

    fn pop(&mut self) -> u16 {
        match self.sp as usize {
            1..=STACK_SIZE => {
                self.sp -= 1;
                self.stack[self.sp as usize]
            }
            _ => panic!("ERROR::Emulator Stack Underflow"),
        }
    }
}

impl Default for Oxid8 {
    fn default() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            rng: rng(),
        }
    }
}

/// Oxid8 CPU Instructions
/// Source: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#0.1
impl Oxid8 {
    /// Naming Conventions:
    /// -------------------
    /// n:      half-byte
    /// kk:     byte
    /// nnn:    address
    /// x,y,i:  register
    /// dt:     delay timer
    /// st:     sound timer
    /// k:      key
    /// -------------------

    /// 00E0 - Clear the display.
    fn cls(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    /// 00EE - Return from a subroutine.
    fn ret(&mut self) {
        self.pc = self.pop();
    }

    /// 1nnn - Jump to location nnn.
    fn jp_nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// 2nnn - Call subroutine at nnn.
    fn call(&mut self, nnn: u16) {
        self.push(self.pc);
        self.pc = nnn;
    }

    /// 3xkk - Skip next instruction if Vx = kk.
    fn se_xkk(&mut self, x: usize, kk: u8) {
        if self.v_reg[x] == kk {
            self.pc += 2;
        }
    }

    /// 4xkk - Skip next instruction if Vx != kk.
    fn sne_xkk(&mut self, x: usize, kk: u8) {
        if self.v_reg[x] != kk {
            self.pc += 2;
        }
    }

    /// 5xy0 - Skip next instruction if Vx = Vy.
    fn se_xy(&mut self, x: usize, y: usize) {
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }

    /// 6xkk - Set Vx = kk.
    fn ld_xkk(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = kk;
    }

    /// 7xkk - Set Vx = Vx + kk.
    fn add_xkk(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = self.v_reg[x].wrapping_add(kk);
    }

    /// 8xy0 - Set Vx = Vy.
    fn ld_xy(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[y];
    }

    /// 8xy1 - Set Vx = Vx OR Vy.
    fn or(&mut self, x: usize, y: usize) {
        self.v_reg[x] |= self.v_reg[y];
    }

    /// 8xy2 - Set Vx = Vx AND Vy.
    fn and(&mut self, x: usize, y: usize) {
        self.v_reg[x] &= self.v_reg[y];
    }

    /// 8xy3 - Set Vx = Vx XOR Vy.
    fn xor(&mut self, x: usize, y: usize) {
        self.v_reg[x] ^= self.v_reg[y];
    }

    /// 8xy4 - Set Vx = Vx + Vy, set VF = carry.
    fn add_xy(&mut self, x: usize, y: usize) {
        let (vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
        self.v_reg[x] = vx;
        self.v_reg[VF] = carry as u8;
    }

    /// 8xy5 - Set Vx = Vx - Vy, set VF = NOT borrow.
    fn sub_xy(&mut self, x: usize, y: usize) {
        let (vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
        self.v_reg[x] = vx;
        self.v_reg[VF] = !borrow as u8;
    }

    /// 8xy6 - Set Vx = Vx SHR 1.
    fn shr(&mut self, x: usize, _y: usize) {
        let vx = self.v_reg[x];
        self.v_reg[x] = vx >> 1;
        self.v_reg[VF] = vx & 1;
    }

    /// 8xy7 - Set Vx = Vy - Vx, set VF = NOT borrow.
    fn subn_xy(&mut self, x: usize, y: usize) {
        let (vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
        self.v_reg[x] = vx;
        self.v_reg[VF] = !borrow as u8;
    }

    /// 8xyE - Set Vx = Vx SHL 1.
    fn shl(&mut self, x: usize, _y: usize) {
        let vx = self.v_reg[x];
        self.v_reg[x] = vx << 1;
        self.v_reg[VF] = (vx >> 7) & 1;
    }

    /// 9xy0 - Skip next instruction if Vx != Vy.
    fn sne_xy(&mut self, x: usize, y: usize) {
        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }

    /// Annn - Set I = nnn.
    fn ld_innn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    /// Bnnn - Jump to location nnn + V0.
    fn jp_0nnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v_reg[0] as u16;
    }

    /// Cxkk - Set Vx = random byte AND kk.
    fn rnd(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = self.rng.random_range(0..=0xFF) as u8 & kk;
    }

    /// Dxyn - Display n-byte sprite starting at memory location I at (Vx, Vy),
    /// set VF = collision.
    fn drw(&mut self, x: usize, y: usize, n: u8) {
        // a sprite is a byte wide and n in [1,15] rows where n is an integer
        let (x, y) = (
            self.v_reg[x] as usize % SCREEN_WIDTH,  // wrap
            self.v_reg[y] as usize % SCREEN_HEIGHT, // wrap
        );
        self.v_reg[VF] = 0; // turn off collision flag
        let start_pixel: usize = (y * SCREEN_WIDTH) + x;
        let start_addr: usize = self.i_reg as usize;

        // draw n bytes to the screen
        for i in 0..n as usize {
            if y + i >= SCREEN_HEIGHT {
                break; // clip
            }
            let pixel_posn: usize = start_pixel + (SCREEN_WIDTH * i);
            let sprite_row: u8 = self.ram[start_addr + i];

            // for each bit
            for j in 0..8 {
                if x + j >= SCREEN_WIDTH {
                    break; // clip
                }
                let ref mut pixel_ref = self.screen[pixel_posn + j];
                let old_pixel = *pixel_ref;

                let sprite_pixel = (sprite_row >> (0x7 - j)) & 0x1;
                *pixel_ref ^= sprite_pixel != 0;

                if !(*pixel_ref) && old_pixel {
                    self.v_reg[VF] = 1; // turn on collision flag
                }
            }
        }
    }

    /// Ex9E - Skip next instruction if key with the value of Vx is pressed.
    fn skp(&mut self, x: usize) {
        if self.keys[self.v_reg[x] as usize] {
            self.pc += 2;
        }
    }

    /// ExA1 - Skip next instruction if key with the value of Vx is not pressed.
    fn sknp(&mut self, x: usize) {
        if !self.keys[self.v_reg[x] as usize] {
            self.pc += 2;
        }
    }

    /// Fx07 - Set Vx = delay timer value.
    fn ld_xdt(&mut self, x: usize) {
        self.v_reg[x] = self.dt;
    }

    /// Fx0A - Wait for a key press, store the value of the key in Vx.
    fn ld_xk(&mut self, x: usize) {
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x] = i as u8;
                pressed = true;
            }
        }
        if !pressed {
            // set pc to previous state
            self.pc -= 2;
        }
    }

    /// Fx15 - Set delay timer = Vx.
    fn ld_dtx(&mut self, x: usize) {
        self.dt = self.v_reg[x];
    }

    /// Fx18 - Set sound timer = Vx.
    fn ld_stx(&mut self, x: usize) {
        self.st = self.v_reg[x];
    }

    /// Fx1E - Set I = I + Vx.
    fn add_ix(&mut self, x: usize) {
        self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
    }

    /// Fx29 - Set I = location of sprite for digit Vx.
    fn ld_fx(&mut self, x: usize) {
        self.i_reg = FONT_ADDR + (self.v_reg[x] as u16 * 5);
    }

    /// Fx33 - Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn ld_bx(&mut self, x: usize) {
        let i = self.i_reg as usize;
        let v = self.v_reg[x];
        self.ram[i] = (v / 100) % 10;
        self.ram[i + 1] = (v / 10) % 10;
        self.ram[i + 2] = v % 10;
    }

    /// Fx55 - Store registers V0 through Vx in memory starting at location I.
    fn ld_ix(&mut self, x: usize) {
        for i in 0..=x {
            self.ram[self.i_reg as usize + i] = self.v_reg[i];
        }
    }

    /// Fx65 - Read registers V0 through Vx from memory starting at location I.
    fn ld_xi(&mut self, x: usize) {
        for i in 0..=x {
            self.v_reg[i] = self.ram[self.i_reg as usize + i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // for misc testing
        let a: [u8; 5] = [255, 155, 100, 55, 5];
        let i: u16 = 0;
        assert_eq!(255, a[i as usize]);
        assert_eq!(155, a[i as usize + 1]);
    }

    #[test]
    fn opcode_new() {
        let opcode = Opcode::new(0x12, 0x34);
        assert_eq!(opcode.0, 0x1);
        assert_eq!(opcode.1, 0x2);
        assert_eq!(opcode.2, 0x3);
        assert_eq!(opcode.3, 0x4);
    }

    #[test]
    fn opcode_decode() {
        let opcode = Opcode::new(0x12, 0x34);
        assert_eq!(opcode.full(), 0x1234);
        assert_eq!(opcode.nnn(), 0x234);
        assert_eq!(opcode.n(), 0x4);
        assert_eq!(opcode.x(), 0x2);
        assert_eq!(opcode.y(), 0x3);
        assert_eq!(opcode.kk(), 0x34);
    }

    #[test]
    fn invalid_opcode() {
        let mut emu = Oxid8::new();
        emu.ram[START_ADDR as usize] = 0xFF;
        emu.ram[START_ADDR as usize + 1] = 0xFF;
        assert!(emu.run_cycle().is_err_and(|msg| msg
            == format!(
                "Invalid Instruction: FFFF at {}", //
                START_ADDR                         //
            )))
    }

    #[test]
    fn push_pop() {
        let mut emu = Oxid8::new();
        assert_eq!(emu.sp, 0); // base stack pointer
        emu.push(1); // push
        assert_eq!(emu.sp, 1); // inc stack pointer
        assert_eq!(emu.stack[0], 1); // value on stack
        assert_eq!(emu.pop(), 1); // pop
        assert_eq!(emu.sp, 0); // dec stack pointer
    }

    #[test]
    #[should_panic(expected = "Stack Overflow")]
    fn push_panic() {
        let mut emu = Oxid8::new();
        for _ in 0..=STACK_SIZE {
            emu.push(1);
        }
    }

    #[test]
    #[should_panic(expected = "Stack Underflow")]
    fn pop_panic() {
        let mut emu = Oxid8::new();
        emu.pop();
    }

    #[test]
    fn load_font() {
        let mut emu = Oxid8::new();
        emu.load_font();
        assert_eq!(
            emu.ram[FONT_ADDR as usize..(FONT_ADDR as usize + FONTSET_SIZE)],
            FONTSET
        );
    }

    #[test]
    fn draw_basic() {
        // just two x on top of each other 8x15
        let sprite = [
            0x81, 0x42, 0x24, 0x18, //
            0x18, 0x24, 0x42, 0x81, //
            0x42, 0x24, 0x18, 0x18, //
            0x24, 0x42, 0x81, //
        ];

        let screen = [
            true, false, false, false, false, false, false, true, // 1
            false, true, false, false, false, false, true, false, // 2
            false, false, true, false, false, true, false, false, // 3
            false, false, false, true, true, false, false, false, // 4
            false, false, false, true, true, false, false, false, // 5
            false, false, true, false, false, true, false, false, // 6
            false, true, false, false, false, false, true, false, // 7
            true, false, false, false, false, false, false, true, // 8
            false, true, false, false, false, false, true, false, // 9
            false, false, true, false, false, true, false, false, // 10
            false, false, false, true, true, false, false, false, // 11
            false, false, false, true, true, false, false, false, // 12
            false, false, true, false, false, true, false, false, // 13
            false, true, false, false, false, false, true, false, // 14
            true, false, false, false, false, false, false, true, // 15
        ];

        let mut emu = Oxid8::new();

        emu.i_reg = START_ADDR;
        let start = START_ADDR as usize;

        emu.ram[start..start + sprite.len()].copy_from_slice(&sprite);
        emu.drw(0, 0, sprite.len() as u8);

        for i in 0..15 {
            let offset1: usize = i * SCREEN_WIDTH;
            let offset2: usize = i * 8;
            assert_eq!(
                emu.screen[offset1 + 0..offset1 + 8],
                screen[offset2 + 0..offset2 + 8]
            );
        }
    }
}
