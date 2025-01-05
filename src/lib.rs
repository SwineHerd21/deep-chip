use display::Display;
use egui::Color32;
use memory::Memory;
use rand::Rng;

pub use quirks::Quirks;

mod display;
mod memory;
mod quirks;

/// The Chip-8 interpreter context.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[allow(non_snake_case)]
pub struct Chip8 {
    /// 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hex digit.  
    /// VF is used as a flag by some instructions.
    V: [u8; 16],
    /// The address register. 16-bit, but only the lowest 12 bits are used.
    I: u16,
    /// The program counter. 16-bit.
    program_counter: u16,
    /// The stack pointer. 8-bit.
    stack_pointer: u8,
    /// The delay timer, decremented 60 times per second. Is accessible by programs.
    delay: u8,
    /// The sound timer, decremented 60 times per second. Plays a sound frequency when greater than 1.
    sound: u8,
    /// 4KB of RAM. The first 512 bytes are reserved.
    memory: Memory,
    /// A monochrome 64x32-pixel display.
    display: Display,
    /// 16 keys corresponding to hex digits.
    keypad: [bool; 16],
    /// Stores return addresses for up to 16 nested subroutines.
    stack: [u16; 16],

    // Configuration and control
    /// The current cycle in a frame.
    pub frame_cycle: u32,
    /// The desired implementation quirks.
    pub quirks: Quirks,
    /// Sound will play if true.
    pub sound_on: bool,
    /// Whether the interpreter is executing instructions.
    running: bool,
    /// If true (and quirk is enabled), the display is ready for drawing.
    vblank: bool,
    /// True if waiting for a key press with the Fx0A instruction.
    awaiting_key: bool,
    /// Used by the Fx0A instruction: The register to which the pressed key will be saved.
    key_destination: usize,
}

impl Chip8 {
    /// Create a Chip8 interpreter with the quirks of the original COSMAC-VIP implementation.  
    #[inline]
    pub fn chip8() -> Chip8 {
        Chip8 {
            // Registers
            V: [0; 16],
            I: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            delay: 0,
            sound: 0,
            // Devices
            memory: Memory::new(),
            display: Display::new(),
            keypad: [false; 16],
            stack: [0; 16],
            // Configuration
            quirks: Quirks::original_chip8(),
            frame_cycle: 0,
            running: false,
            vblank: true,
            awaiting_key: false,
            key_destination: 0,
            sound_on: false,
        }
    }

    #[inline]
    pub fn start(&mut self) {
        self.running = true;
    }
    #[inline]
    pub fn stop(&mut self) {
        self.running = false;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.V = [0; 16];
        self.I = 0;
        self.program_counter = 0x200;
        self.stack_pointer = 0;
        self.delay = 0;
        self.sound = 0;
        self.memory.reset();
        self.display = Display::new();
        self.keypad = [false; 16];
        self.stack = [0; 16];
        self.awaiting_key = false;
    }

    #[inline]
    pub fn running(&self) -> bool {
        self.running
    }

    // Registers
    #[inline]
    pub fn get_register(&self, i: usize) -> u8 {
        self.V[i]
    }
    #[inline]
    fn set_flag(&mut self, value: u8) {
        self.V[0xF] = value;
    }
    #[inline]
    pub fn get_i(&self) -> u16 {
        self.I
    }
    #[inline]
    pub fn get_program_counter(&self) -> u16 {
        self.program_counter
    }
    #[inline]
    fn increment_program_counter(&mut self) {
        self.program_counter += 2
    }
    #[inline]
    pub fn get_stack_pointer(&self) -> u8 {
        self.stack_pointer
    }
    #[inline]
    pub fn read_stack(&self, i: usize) -> u16 {
        self.stack[i]
    }
    #[inline]
    pub fn get_delay(&self) -> u8 {
        self.delay
    }
    #[inline]
    pub fn get_sound(&self) -> u8 {
        self.sound
    }
    #[inline]
    pub fn update_timers(&mut self) {
        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
    }

    // RAM
    #[inline]
    pub fn get_current_opcode(&self) -> u16 {
        self.memory.read_opcode(self.get_program_counter())
    }
    #[inline]
    pub fn ram_len(&self) -> usize {
        self.memory.ram.len()
    }
    #[inline]
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory.ram[address as usize]
    }
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory.ram[address as usize] = value
    }
    #[inline]
    pub fn load_program(&mut self, program: &[u8]) {
        self.memory.reset();
        self.memory.load_program(program);
    }

    // Display
    #[inline]
    pub fn get_display(&self, background_color: Color32, fill_color: Color32) -> egui::ColorImage {
        self.display.render(background_color, fill_color)
    }
    #[inline]
    pub fn set_vblank(&mut self) {
        self.vblank = true;
    }

    // Keypad
    #[inline]
    pub fn set_keys(&mut self, keys: [bool; 16]) {
        self.keypad = keys;
    }
    #[inline]
    pub fn get_key_state(&self, key: usize) -> bool {
        self.keypad[key]
    }
    #[inline]
    pub fn is_waiting_for_key(&self) -> bool {
        self.awaiting_key
    }
    #[inline]
    pub fn save_awaited_key(&mut self, key: u8) {
        self.V[self.key_destination] = key;
        self.awaiting_key = false;
    }

    /// A frame has completed: decrement the timers and set vblank.
    pub fn tick_frame(&mut self) {
        self.update_timers();
        self.set_vblank();
        self.frame_cycle = 0;
    }

    /// Get the next instruction and execute it.
    pub fn execute_cycle(&mut self) {
        if self.get_program_counter() >= self.memory.ram.len() as u16 - 2 {
            self.stop();
            return;
        }

        self.frame_cycle += 1;

        let instruction: u16 = self.get_current_opcode();

        self.execute_instruction(instruction);
    }

    /// Parse and execute the instruction.
    pub fn execute_instruction(&mut self, opcode: u16) {
        if self.awaiting_key {
            return;
        }

        let addr = opcode & 0x0FFF; // 0nnn
        let x = ((opcode & 0x0F00) >> 8) as usize; // 0x00
        let y = ((opcode & 0x00F0) >> 4) as usize; // 00y0
        let byte = (opcode & 0x00FF) as u8; // 00kk
        let nibble = (opcode & 0x000F) as u8; // 0nnn

        match opcode >> 12 {
            0x0 => match byte {
                // 00E0 - Clear the screen
                0xE0 => self.display.pixels = [false; 64 * 32],
                // 00EE - Return from subroutine
                0xEE => {
                    self.stack_pointer = self.get_stack_pointer().saturating_sub(1);
                    self.program_counter = self.stack[self.get_stack_pointer() as usize];
                    return;
                }
                _ => (),
            },
            // 1nnn - Jump to nnn
            0x1 => {
                self.program_counter = addr;
                return;
            }
            // 2nnn - Call subroutine at nnn
            0x2 => {
                self.stack[self.get_stack_pointer() as usize] = self.get_program_counter() + 2;
                self.stack_pointer = self.get_stack_pointer().saturating_add(1);
                self.program_counter = addr;
                return;
            }
            // 3xnn - Skip if Vx == nn
            0x3 => {
                if self.get_register(x) == byte {
                    self.increment_program_counter();
                }
            }
            // 4xnn - Skip if Vx != nn
            0x4 => {
                if self.get_register(x) != byte {
                    self.increment_program_counter();
                }
            }
            // 5xy0 - Skip if Vx == Vy
            0x5 if nibble == 0 => {
                if self.get_register(x) == self.get_register(y) {
                    self.increment_program_counter();
                }
            }
            // 6xnn - Set Vx = nn
            0x6 => self.V[x] = byte,
            // 7xnn - Set Vx += nn
            0x7 => self.V[x] = self.get_register(x).wrapping_add(byte),
            0x8 => match nibble {
                // 8xy0 - Set Vx = Vy
                0x0 => self.V[x] = self.V[y],
                // 8xy1 - Set Vx |= Vy
                // Set VF to 0 (quirk)
                0x1 => {
                    self.V[x] |= self.get_register(y);
                    if self.quirks.bitwise_reset_vf {
                        self.set_flag(0);
                    }
                }
                // 8xy2 - Set Vx &= Vy
                // Set VF to 0 (quirk)
                0x2 => {
                    self.V[x] &= self.get_register(y);
                    if self.quirks.bitwise_reset_vf {
                        self.set_flag(0);
                    }
                }
                // 8xy3 - Set Vx ^= Vy
                // Set VF to 0 (quirk)
                0x3 => {
                    self.V[x] ^= self.get_register(y);
                    if self.quirks.bitwise_reset_vf {
                        self.set_flag(0);
                    }
                }
                // 8xy4 - Set Vx += Vy, set VF to 1 if overflowed, to 0 if not
                0x4 => {
                    let flag;
                    (self.V[x], flag) = self.get_register(x).overflowing_add(self.get_register(y));
                    if flag {
                        self.set_flag(1);
                    } else {
                        self.set_flag(0);
                    }
                }
                // 8xy5 - Set Vx -= Vy, set VF to 0 if underflowed, to 1 if not
                0x5 => {
                    let flag;
                    (self.V[x], flag) = self.get_register(x).overflowing_sub(self.get_register(y));
                    if flag {
                        self.set_flag(0);
                    } else {
                        self.set_flag(1);
                    }
                }
                // 8xy6 - Set Vx = Vy >> 1, set VF to the bit that was shifted out
                // Or set Vx >>= 1 (quirk)
                0x6 => {
                    if !self.quirks.direct_shifting {
                        self.V[x] = self.get_register(y);
                    }

                    let shifted = self.get_register(x) & 1;
                    self.V[x] >>= 1;
                    self.set_flag(shifted);
                }
                // 8xy7 - Set Vx = Vy - Vx, set VF to 0 if underflowed, to 1 if not
                0x7 => {
                    let flag;
                    (self.V[x], flag) = self.get_register(y).overflowing_sub(self.get_register(x));
                    if flag {
                        self.set_flag(0);
                    } else {
                        self.set_flag(1);
                    }
                }
                // 8xyE - Set Vx = Vy << 1, set VF to the bit that was shifted out
                // Or set Vx <<= 1 (quirk)
                0xE => {
                    if !self.quirks.direct_shifting {
                        self.V[x] = self.get_register(y);
                    }

                    let shifted = self.get_register(x) & 0b10000000;
                    self.V[x] <<= 1;
                    self.set_flag(shifted >> 7);
                }
                _ => (),
            },
            // 9xy0 - Skip if Vx != Vy
            0x9 if nibble == 0 => {
                if self.get_register(x) != self.get_register(y) {
                    self.increment_program_counter();
                }
            }
            // Annn - Set I to nnn
            0xA => self.I = addr,
            // Bnnn - Jump to nnn + V0
            // Bxnn - Jump to xnn + Vx (quirk)
            0xB => {
                self.program_counter = addr
                    + if self.quirks.jump_to_x {
                        self.get_register(x)
                    } else {
                        self.get_register(0)
                    } as u16;
                return;
            }
            // Cxnn - Set Vx = a random value & nn
            0xC => self.V[x] = rand::thread_rng().gen::<u8>() & byte,
            // Dxyn - Draw 8xn sprite at Vx, Vy from address I
            // Optionally wait for a vblank interrupt (quirk)
            0xD => {
                if self.quirks.wait_for_vblank && !self.vblank {
                    return;
                }

                let mut overlap = false;
                for row in 0..nibble as u16 {
                    let sprite_byte = self.memory.ram[self.get_i() as usize + row as usize];
                    for cell in 0..8 {
                        let dx = self.get_register(x) as u16;
                        let dy = self.get_register(y) as u16;

                        if self.quirks.edge_clipping && (dx % 64 + cell > 63 || dy % 32 + row > 31)
                        {
                            break;
                        }

                        let sprite_pixel = sprite_byte & (0b10000000 >> cell) != 0;

                        let target_pixel = ((dx + cell) % 64 + (dy + row) % 32 * 64) as usize;

                        if sprite_pixel {
                            if self.display.pixels[target_pixel] {
                                overlap = true;
                            }
                            self.display.pixels[target_pixel] = !self.display.pixels[target_pixel];
                        }
                    }
                }
                self.set_flag(if overlap { 1 } else { 0 });

                self.vblank = false;
            }
            0xE => match byte {
                // Ex9E - Skip if key Vx is down
                0x9E => {
                    if self.keypad[(self.get_register(x) & 0x0F) as usize] {
                        self.increment_program_counter();
                    }
                }
                // ExA1 - Skip if key Vx is up
                0xA1 => {
                    if !self.keypad[(self.get_register(x) & 0x0F) as usize] {
                        self.increment_program_counter();
                    }
                }
                _ => (),
            },
            0xF => match byte {
                // Ex07 - Set Vx to delay
                0x07 => self.V[x] = self.get_delay(),
                // Fx0A - Wait for a key pressed and released and set it to Vx
                0x0A => {
                    self.awaiting_key = true;
                    self.key_destination = x;
                }
                // Fx15 - Set delay to Vx
                0x15 => self.delay = self.get_register(x),
                // Fx18 - Set sound to Vx
                0x18 => self.sound = self.get_register(x),
                // Fx1E - Set I += Vx
                0x1E => self.I += self.get_register(x) as u16,
                // Fx29 - Set I to the address of the font sprite for Vx's lowest nibble
                0x29 => self.I = (self.get_register(x) as u16) % 16 * 5,
                // Fx33 - Write Vx as BCD to addresses I, I+1 and I+2
                0x33 => {
                    self.write_byte(self.get_i(), self.get_register(x) / 100);
                    self.write_byte(self.get_i() + 1, (self.get_register(x) / 10) % 10);
                    self.write_byte(self.get_i() + 2, (self.get_register(x) % 100) % 10);
                }
                // Fx55 - Write V0 to Vx to addresses I to I+x, I is incremented by x
                // Or I is incremented by x+1 (quirk)
                0x55 => {
                    for i in 0..=x {
                        self.write_byte(self.get_i() + i as u16, self.get_register(i));
                    }
                    self.I += if self.quirks.save_load_increment {
                        x as u16 + 1
                    } else {
                        x as u16
                    };
                }
                // Fx65 - Read from addresses I to I+x to V0 to Vx, I is incremented by x
                // Or I is incremented by x+1 (quirk)
                0x65 => {
                    for i in 0..=x {
                        self.V[i] = self.read_byte(self.get_i() + i as u16);
                    }
                    self.I += if self.quirks.save_load_increment {
                        x as u16 + 1
                    } else {
                        x as u16
                    };
                }
                _ => (),
            },
            _ => (),
        }
        self.increment_program_counter();
    }
}
