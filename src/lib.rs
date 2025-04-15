//https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
const FONTSET_SIZE: usize = 80;

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
}

struct Opcode {}

// sprites are 8p wide and 1-15p tall
// Sprite pixels are XOR'd with corresponding screen pixels.
// NOTE: use bell character for a beep \X07
// NOTE: use the left four columns of 1234 for the keypad
// HACK: TRY TO RENDER IT IN THE TERMINAL!!!

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
        }
    }

    #[inline(always)]
    fn load_font(&mut self) {
        self.ram[0..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn run(&mut self) {
        self.load_font();

        loop {
            // TODO: timing: 1-4MHz; 100 instructions per second is common
            // TODO: fetch
            //
            //  read two bytes
            //  increment pc by 2
            //
            // TODO: decode (probably into an opcode struct)
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
            break;
        }
    }

    pub fn push(&mut self, val: u16) {
        match self.sp as usize {
            0..STACK_SIZE => {
                self.stack[self.sp as usize] = val;
                self.sp += 1;
            }
            _ => panic!("Stack is full"),
        };
    }

    // TODO: Think about if this should panic with an "underflow" or return None
    pub fn pop(&mut self) -> Option<u16> {
        match self.sp as usize {
            1..=STACK_SIZE => {
                self.sp -= 1;
                Some(self.stack[self.sp as usize])
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn push_panic() {
        let mut c8 = Oxid8::new();
        for _ in 0..=STACK_SIZE {
            c8.push(1);
        }
    }

    #[test]
    fn pop_some() {
        let mut c8 = Oxid8::new();
        c8.push(1);
        assert_eq!(c8.pop(), Some(1));
    }

    #[test]
    fn pop_none() {
        let mut c8 = Oxid8::new();
        assert_eq!(c8.pop(), None);
    }

    #[test]
    fn load_font() {
        let mut c8 = Oxid8::new();
        c8.load_font();
        assert_eq!(c8.ram[0..FONTSET_SIZE], FONTSET);
    }
}
