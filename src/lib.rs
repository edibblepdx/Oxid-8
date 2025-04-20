mod renderer;

use rand::{Rng, rng, rngs::ThreadRng};
use std::{fmt, fs, io, time::Instant};

//https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
//http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#0.1

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
const START_ADDR: u16 = 0x200;
const TICK_RATE: u64 = 1 / 700; // 700 instructions per second

#[derive(Debug)]
struct Opcode(u8, u8, u8, u8);

#[allow(dead_code)]
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
    tr: u64,                                      // Tick Rate
    rng: ThreadRng,
}

#[allow(dead_code)]
impl Opcode {
    fn new(byte1: u8, byte2: u8) -> Opcode {
        Opcode(
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

// NOTE: sprites are 8p wide and 1-15p tall
// NOTE: Sprite pixels are XOR'd with corresponding screen pixels.
// NOTE: use bell character for a beep \X07
// NOTE: use the left four columns of 1234 for the keypad
// HACK: TRY TO RENDER IT IN THE TERMINAL!!!

#[allow(dead_code)]
impl Oxid8 {
    pub fn new() -> Oxid8 {
        Oxid8 {
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
            tr: TICK_RATE,
            rng: rng(),
        }
    }

    // WARN: might be a good idea to move self here
    pub fn run(&mut self, filename: &str) -> io::Result<()> {
        self.load_font();
        self.load_rom(filename)?;

        // TODO: timing: 1-4MHz; 100 instructions per second is common
        //  a standard speed of around 700 CHIP-8 instructions per second

        loop {
            let time = Instant::now();
            // TODO: fetch

            let opcode = Opcode::new(self.ram[self.pc as usize], self.ram[self.pc as usize + 1]);
            self.pc += 1; // WARN: is this +1 or +2???

            // TODO: decode
            //
            //  match first half-byte (first hex number) [broad category]
            //  X second half byte looks up register V[0-F]
            //  Y third half byte looks up register V[0-F]
            //  N fourth half byte: 4-bit number
            //
            //  NN The second byte (3rd & 4th half-bytes):
            //      an 8 bit immediate number
            //  NNN The second, third, fourth half-bytes:
            //      a 12-bit immediate memory address
            //
            // TODO: execute
            //
            // execute the instruction

            match opcode.0 {
                0x0 => (),
                0x1 => (),
                0x2 => (),
                0x3 => (),
                0x4 => (),
                0x5 => (),
                0x6 => (),
                0x7 => (),
                0x8 => (),
                0x9 => (),
                0xA => (),
                0xB => (),
                0xC => (),
                0xD => (),
                0xE => (),
                0xF => (),
                _ => (), // unreachable
            }

            while time.elapsed().as_secs() < self.tr {} // spin
            break; // WARN: Temporary (will be removed)
        }

        Ok(())
    }

    pub fn tick_rate(&mut self, tr: u64) {
        self.tr = tr;
    }

    fn load_font(&mut self) {
        self.ram[FONT_ADDR as usize..(FONT_ADDR as usize + FONTSET_SIZE)].copy_from_slice(&FONTSET);
    }

    fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        let rom: Vec<u8> = fs::read(filename)?;
        let len = rom.len();
        if len > (RAM_SIZE - START_ADDR as usize) {
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                format!("ROM too large: {}", len),
            ));
        }

        self.ram[START_ADDR as usize..(START_ADDR as usize + len)].copy_from_slice(&rom);

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

/*
    00E0 (clear screen)                     done
    1NNN (jump)                             done
    6XNN (set register VX)                  done
    7XNN (add value to register VX)         done
    ANNN (set index register I)
    DXYN (display/draw)
*/

/// Oxid8 CPU Instructions
///
/// Naming Conventions:
/// n:      half-byte
/// kk:     byte
/// nnn:    address
/// x,y,i:  register
#[allow(dead_code)]
impl Oxid8 {
    /// 00E0 - Clear the display.
    fn cls(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    /// 00EE - Return from a subroutine.
    fn ret(&self) {
        todo!()
    }

    /// 1nnn - Jump to location nnn.
    fn jp_nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// 2nnn - Call subroutine at nnn.
    fn call_nnn(&self) {
        todo!()
    }

    /// 3xkk - Skip next instruction if Vx = kk.
    fn se_xkk(&self) {
        todo!()
    }

    /// 4xkk - Skip next instruction if Vx != kk.
    fn sne_xkk(&self) {
        todo!()
    }

    /// 5xy0 - Skip next instruction if Vx = Vy.
    fn se_xy(&self) {
        todo!()
    }

    /// 6xkk - Set Vx = kk.
    fn ld_xkk(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = kk;
    }

    /// 7xkk - Set Vx = Vx + kk.
    fn add_xkk(&mut self, x: usize, kk: u8) {
        self.v_reg[x] += kk;
    }

    /// 8xy0 - Set Vx = Vy.
    fn ld_xy(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[y];
    }

    /// 8xy1 - Set Vx = Vx OR Vy.
    fn or(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[x] | self.v_reg[y];
    }

    /// 8xy2 - Set Vx = Vx AND Vy.
    fn and(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[x] & self.v_reg[y];
    }

    /// 8xy3 - Set Vx = Vx XOR Vy.
    fn xor(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[x] ^ self.v_reg[y];
    }

    /// 8xy4 - Set Vx = Vx + Vy, set VF = carry.
    fn add_xy(&self, x: usize, y: usize) {
        todo!()
    }

    /// 8xy5 - Set Vx = Vx - Vy, set VF = NOT borrow.
    fn sub_xy(&self, x: usize, y: usize) {
        todo!()
    }

    /// 8xy6 - Set Vx = Vx SHR 1.
    fn shr(&self, x: usize, _y: usize) {
        todo!()
    }

    /// 8xy7 - Set Vx = Vy - Vx, set VF = NOT borrow.
    fn subn_xy(&self) {
        todo!()
    }

    /// 8xyE - Set Vx = Vx SHL 1.
    fn shl(&self, x: usize, _y: usize) {
        todo!()
    }

    /// 9xy0 - Skip next instruction if Vx != Vy.
    fn sne_xy(&self) {
        todo!()
    }

    /// Annn - Set I = nnn.
    fn ld_innn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    /// Jump to location nnn + V0.
    fn jp_0nnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v_reg[0] as u16;
    }

    /// Cxkk - Set Vx = random byte AND kk.
    fn rnd(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = self.rng.random_range(0..=255) as u8 & kk;
    }

    /// Dxyn - Display n-byte sprite starting at memory location I at (Vx, Vy),
    /// set VF = collision.
    fn drw(&self, x: usize, y: usize, n: u8) {
        let (x, y) = (
            self.v_reg[x] as usize % SCREEN_WIDTH,
            self.v_reg[y] as usize % SCREEN_HEIGHT,
        );
    }

    /// Ex9E Skip next instruction if key with the value of Vx is pressed.
    fn skp(&self) {
        todo!()
    }

    /// Skip next instruction if key with the value of Vx is not pressed.
    fn sknp(&self) {
        todo!()
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
    #[should_panic]
    fn push_panic() {
        let mut c8 = Oxid8::new();
        for _ in 0..=STACK_SIZE {
            c8.push(1);
        }
    }

    #[test]
    fn pop() {
        let mut c8 = Oxid8::new();
        c8.push(1);
        assert_eq!(c8.pop(), 1);
    }

    #[test]
    #[should_panic]
    fn pop_panic() {
        let mut c8 = Oxid8::new();
        c8.pop();
    }

    #[test]
    fn load_font() {
        let mut c8 = Oxid8::new();
        c8.load_font();
        assert_eq!(
            c8.ram[FONT_ADDR as usize..(FONT_ADDR as usize + FONTSET_SIZE)],
            FONTSET
        );
    }
}

#[cfg(test)]
mod test_cpu_instructions {}
