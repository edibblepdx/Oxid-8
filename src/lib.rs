use std::{fmt, fs, io, time::Instant};

//https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
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
impl Opcode {
    fn new(byte1: u8, byte2: u8) -> Opcode {
        Opcode(
            (byte1 & 0xF0) >> 4,
            byte1 & 0x0F,
            (byte2 & 0xF0) >> 4,
            byte2 & 0x0F,
        )
    }

    fn full(&self) -> u16 {
        (self.0 as u16) << 12 | (self.1 as u16) << 8 | (self.2 as u16) << 4 | (self.3 as u16)
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.0, self.1, self.2, self.3)
    }
}

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
        }
    }

    pub fn run(&mut self, filename: &str) -> io::Result<()> {
        self.load_font();
        self.load_rom(filename)?;

        // TODO: timing: 1-4MHz; 100 instructions per second is common
        //  a standard speed of around 700 CHIP-8 instructions per second

        loop {
            let time = Instant::now();
            // TODO: fetch

            let opcode = Opcode::new(self.ram[self.pc as usize], self.ram[self.pc as usize + 1]);
            self.pc += 1;

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
            //let nn = opcode.2 ^ (opcode.3 << 2);
            //
            // TODO: execute
            //
            // execute the instruction
            //
            while time.elapsed().as_secs() < self.tr {} // spin
            break; // WARN: Temporary (will be removed)
        }

        Ok(())
    }

    pub fn tick_rate(&mut self, tr: u64) {
        self.tr = tr;
    }

    #[inline(always)]
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
        assert_eq!(opcode.full(), 0x1234);
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
