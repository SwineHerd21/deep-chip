/// The memory of the CHIP-8.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Memory {
    /// 4KB of RAM. 0x000-0x1FF is reserved for the interpreter.
    pub ram: [u8; 4096],
}

/// The text font stored in reserved memory.
const CHIP8_FONT: [u8; 16 * 5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

impl Memory {
    /// Create memory with the default font.
    #[inline]
    pub fn new() -> Memory {
        let mut mem = Memory { ram: [0; 4096] };
        mem.ram[0..(16 * 5)].copy_from_slice(&CHIP8_FONT); // Save font
        mem
    }

    /// Clear all non-reserved memory.
    #[inline]
    pub fn reset(&mut self) {
        self.ram = [0; 4096];
        self.ram[0..(16 * 5)].copy_from_slice(&CHIP8_FONT); // Save font
    }

    /// Load a program to memory starting at address 0x200.
    #[inline]
    pub fn load_program(&mut self, rom: &[u8]) {
        self.ram[0x200..(0x200 + rom.len())].copy_from_slice(rom);
    }

    /// Read two bytes at the passed address and combine them into an instruction.
    #[inline]
    pub fn read_opcode(&self, address: u16) -> u16 {
        (self.ram[address as usize] as u16) << 8 | self.ram[(address as usize) + 1] as u16
    }
}
