use std::fs;

use display::{Display, ScrollDirection};
use egui::Color32;
use memory::Memory;
use rand::Rng;

pub use quirks::Quirks;
pub use quirks::Variant;

mod display;
mod memory;
mod quirks;

/// The CHIP-8 interpreter context.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
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
    /// If false, the display will have a resolution of 64x32.
    /// Otherwise, if the selected variant supports it, the resolution will be 128x64.
    pub highres: bool,
    /// 16 keys corresponding to hex digits.
    keypad: [bool; 16],
    /// Stores return addresses for subroutines.
    stack: Vec<u16>,

    // Configuration and control
    /// What kind of CHIP-8 variant to run as.
    pub variant: Variant,
    /// The desired implementation quirks.
    pub quirks: Quirks,
    /// Sound will play if true.
    pub sound_on: bool,
    /// The size of the stack. 12 in CHIP-8 mode, 16 in SCHIP mode.
    pub stack_size: usize,
    /// The current cycle in a frame.
    pub frame_cycle: u32,
    /// How many cycles to execute in one frame.
    pub execution_speed: u32,
    /// Whether the interpreter is executing instructions.
    running: bool,
    /// If the interpreter halts, this will have a message explaining why.
    pub halt_message: Option<String>,
    /// If true (and quirk is enabled), the display is ready for drawing.
    vblank: bool,
    /// True if waiting for a key press with the Fx0A instruction.
    awaiting_key: bool,
    /// Used by the Fx0A instruction: The register to which the pressed key will be saved.
    key_destination: usize,
    /// Used by the Fx75 and Fx85 instructions of SUPER-CHIP and XO-CHIP as runtime storage.
    persistent_flags: [u8; 8],
}

impl Chip8 {
    /// Create a CHIP-8 interpreter with the quirks of the original COSMAC-VIP implementation.  
    #[inline]
    pub fn chip8() -> Chip8 {
        let stack_size = 12;
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
            display: Display::small(),
            highres: false,
            keypad: [false; 16],
            stack: vec![0; stack_size],
            // Configuration
            variant: Variant::CHIP8,
            quirks: Quirks::vip_chip(),
            frame_cycle: 0,
            execution_speed: 15,
            stack_size,
            sound_on: true,
            running: false,
            halt_message: None,
            vblank: true,
            awaiting_key: false,
            key_destination: 0,
            persistent_flags: [0; 8],
        }
    }

    /// Create a SUPER-CHIP 1.1 interpreter.  
    #[inline]
    pub fn super_chip1_1() -> Chip8 {
        let stack_size = 16;
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
            display: Display::big(),
            highres: false,
            keypad: [false; 16],
            stack: vec![0; stack_size],
            // Configuration
            variant: Variant::SCHIP11,
            quirks: Quirks::super_chip1_1(),
            frame_cycle: 0,
            execution_speed: 30,
            stack_size,
            sound_on: true,
            running: false,
            halt_message: None,
            vblank: true,
            awaiting_key: false,
            key_destination: 0,
            persistent_flags: Chip8::load_persistent_flags(),
        }
    }

    /// Set registers and timers to zero, clear the stack, screen and RAM and reload the ROM.
    #[inline]
    pub fn reset(&mut self) {
        self.V = [0; 16];
        self.I = 0;
        self.program_counter = 0x200;
        self.stack_pointer = 0;
        self.delay = 0;
        self.sound = 0;
        self.memory.reset();
        self.display.clear();
        self.highres = false;
        self.keypad = [false; 16];
        self.stack = vec![0; self.stack_size];
        self.awaiting_key = false;
        self.frame_cycle = 0;
        self.vblank = true;
        self.halt_message = None;
    }

    /// Set `running` to `true`.
    #[inline]
    pub fn start(&mut self) {
        self.running = true;
    }
    /// Set `running` to `false`.
    #[inline]
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Set the VF register. Basically boilerplate code.
    #[inline]
    fn set_flag(&mut self, value: u8) {
        self.V[0xF] = value;
    }
    /// Move the program counter to the next instruction (increment by 2).
    #[inline]
    fn increment_program_counter(&mut self) {
        self.program_counter += 2
    }
    /// Subtract one from the timers.
    #[inline]
    pub fn update_timers(&mut self) {
        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
    }

    /// Get the opcode that the PC is pointing to.
    #[inline]
    pub const fn get_current_opcode(&self) -> u16 {
        self.memory.read_opcode(self.program_counter)
    }
    /// Read a byte from memory.
    #[inline]
    pub const fn read_byte(&self, address: u16) -> u8 {
        self.memory.ram[address as usize]
    }
    /// Write a value to memory.
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory.ram[address as usize] = value
    }
    /// Reset memory and load a program into it, starting at 0x200.
    #[inline]
    pub fn load_program(&mut self, program: &[u8]) {
        self.memory.reset();
        self.memory.load_program(program);
    }

    /// Load persistent flag registers from a file.
    #[inline]
    pub fn load_persistent_flags() -> [u8; 8] {
        let mut flags = [0; 8];
        if let Ok(f) = fs::read("flags.dat") {
            for i in 0..8 {
                flags[i] = f[i];
            }
        } else {
            println!("Did not find a persistent flag file");
        }
        return flags;
    }

    /// Save persistent flag registers into a file.
    #[inline]
    pub fn save_persistent_flags(&self) {
        if let Err(e) = fs::write("flags.dat", self.persistent_flags) {
            panic!("Could not save persistent flags! What is wrong with your file system? {e}");
        }
    }

    /// Read the display in the form of a texture.
    #[inline]
    pub fn get_display(&self, background_color: Color32, fill_color: Color32) -> egui::ColorImage {
        self.display
            .render(self.highres, background_color, fill_color)
    }
    /// Set vblank ready.
    #[inline]
    pub fn set_vblank(&mut self) {
        self.vblank = true;
    }

    /// Set keypad state.
    #[inline]
    pub fn set_keys(&mut self, keys: [bool; 16]) {
        self.keypad = keys;
    }
    /// Save the value of the last pressed key into a register as the result of the Fx0A instruction.
    #[inline]
    pub fn save_awaited_key(&mut self, key: u8) {
        self.V[self.key_destination] = key;
        self.awaiting_key = false;
    }

    /// Complete a frame: decrement timers and set vblank.
    pub fn tick_frame(&mut self) {
        self.update_timers();
        self.set_vblank();
        self.frame_cycle = 0;
    }

    /// Get the next instruction and execute it.
    pub fn execute_cycle(&mut self) {
        self.halt_message = None;

        if self.program_counter >= self.memory.ram.len() as u16 - 2 {
            self.stop();
            return;
        }

        self.frame_cycle += 1;

        let instruction: u16 = self.get_current_opcode();

        self.execute_instruction(instruction);
    }

    /// Parse and execute an instruction.
    pub fn execute_instruction(&mut self, opcode: u16) {
        if self.awaiting_key {
            return;
        }

        let addr = opcode & 0x0FFF; // 0nnn
        let x = ((opcode & 0x0F00) >> 8) as usize; // 0x00
        let y = ((opcode & 0x00F0) >> 4) as usize; // 00y0
        let byte = (opcode & 0x00FF) as u8; // 00kk
        let nibble = (opcode & 0x000F) as u8; // 000n

        match opcode >> 12 {
            0x0 => {
                // Reached empty code, just stop
                if opcode == 0x0000 {
                    self.stop();
                }
                // 00Cn - Scroll down by n pixels (SUPER-CHIP)
                else if self.variant.supports_schip() && y == 0xC {
                    {
                        self.display.scroll(
                            ScrollDirection::Down,
                            nibble as usize,
                            self.highres,
                            self.quirks.lowres_scroll,
                        )
                    }
                } else {
                    match byte {
                        // 00E0 - Clear the screen
                        0xE0 => self.display.clear(),
                        // 00EE - Return from subroutine
                        0xEE => {
                            self.stack_pointer = self.stack_pointer.saturating_sub(1);
                            self.program_counter = self.stack[self.stack_pointer as usize];
                            return;
                        }
                        // 00FF - Enable high resolution mode (SUPER-CHIP)
                        0xFF if self.variant.supports_schip() => self.highres = true,
                        // 00FE - Disable high resolution mode (SUPER-CHIP)
                        0xFE if self.variant.supports_schip() => self.highres = false,
                        // 00FB - Scroll the display 4 pixels right (SUPER-CHIP)
                        0xFB if self.variant.supports_schip() => self.display.scroll(ScrollDirection::Right, 4,self.highres,self.quirks.lowres_scroll),
                        // 00FC - Scroll the display 4 pixels left (SUPER-CHIP)
                        0xFC if self.variant.supports_schip() => {
                            self.display.scroll(ScrollDirection::Left, 4,self.highres,self.quirks.lowres_scroll)
                        }
                        // 00FD - Exit the interpreter (SUPER-CHIP)
                        0xFD if self.variant.supports_schip() => {
                            self.stop();
                            self.reset();
                        }
                        _ => self.halt(format!(
                            "Machine code routines are not supported: {:04X}. Try a different CHIP-8 variant.",
                            opcode
                        )),
                    }
                }
            }
            // 1nnn - Jump to nnn
            0x1 => {
                self.program_counter = addr;
                return;
            }
            // 2nnn - Call subroutine at nnn
            0x2 => {
                self.stack[self.stack_pointer as usize] = self.program_counter + 2;
                self.stack_pointer = self.stack_pointer.saturating_add(1);
                self.program_counter = addr;
                return;
            }
            // 3xnn - Skip if Vx == nn
            0x3 => {
                if self.V[x] == byte {
                    self.increment_program_counter();
                }
            }
            // 4xnn - Skip if Vx != nn
            0x4 => {
                if self.V[x] != byte {
                    self.increment_program_counter();
                }
            }
            // 5xy0 - Skip if Vx == Vy
            0x5 if nibble == 0 => {
                if self.V[x] == self.V[y] {
                    self.increment_program_counter();
                }
            }
            // 6xnn - Set Vx = nn
            0x6 => self.V[x] = byte,
            // 7xnn - Set Vx += nn
            0x7 => self.V[x] = self.V[x].wrapping_add(byte),
            0x8 => match nibble {
                // 8xy0 - Set Vx = Vy
                0x0 => self.V[x] = self.V[y],
                // 8xy1 - Set Vx |= Vy
                // Set VF to 0 (quirk)
                0x1 => {
                    self.V[x] |= self.V[y];
                    if self.quirks.bitwise_reset_vf {
                        self.set_flag(0);
                    }
                }
                // 8xy2 - Set Vx &= Vy
                // Set VF to 0 (quirk)
                0x2 => {
                    self.V[x] &= self.V[y];
                    if self.quirks.bitwise_reset_vf {
                        self.set_flag(0);
                    }
                }
                // 8xy3 - Set Vx ^= Vy
                // Set VF to 0 (quirk)
                0x3 => {
                    self.V[x] ^= self.V[y];
                    if self.quirks.bitwise_reset_vf {
                        self.set_flag(0);
                    }
                }
                // 8xy4 - Set Vx += Vy, set VF to 1 if overflowed, to 0 if not
                0x4 => {
                    let flag;
                    (self.V[x], flag) = self.V[x].overflowing_add(self.V[y]);
                    if flag {
                        self.set_flag(1);
                    } else {
                        self.set_flag(0);
                    }
                }
                // 8xy5 - Set Vx -= Vy, set VF to 0 if underflowed, to 1 if not
                0x5 => {
                    let flag;
                    (self.V[x], flag) = self.V[x].overflowing_sub(self.V[y]);
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
                        self.V[x] = self.V[y];
                    }

                    let shifted = self.V[x] & 1;
                    self.V[x] >>= 1;
                    self.set_flag(shifted);
                }
                // 8xy7 - Set Vx = Vy - Vx, set VF to 0 if underflowed, to 1 if not
                0x7 => {
                    let flag;
                    (self.V[x], flag) = self.V[y].overflowing_sub(self.V[x]);
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
                        self.V[x] = self.V[y];
                    }

                    let shifted = self.V[x] & 0b10000000;
                    self.V[x] <<= 1;
                    self.set_flag(shifted >> 7);
                }
                _ => self.halt(format!("Illegal instruction: {:04X}", opcode)),
            },
            // 9xy0 - Skip if Vx != Vy
            0x9 if nibble == 0 => {
                if self.V[x] != self.V[y] {
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
                        self.V[x]
                    } else {
                        self.V[0]
                    } as u16;
                return;
            }
            // Cxnn - Set Vx = a random value & nn
            0xC => self.V[x] = rand::thread_rng().gen::<u8>() & byte,
            // Dxy0 - Draw 16x16 sprite at Vx, Vy from address I (SUPER-CHIP)
            0xD if self.variant.supports_schip() && nibble == 0 => {
                if self.quirks.wait_for_vblank && !self.vblank {
                    return;
                }

                let width = if self.highres { 128 } else { 64 };
                let height = if self.highres { 64 } else { 32 };

                let dx = self.V[x] as u16;
                let dy = self.V[y] as u16;

                let mut overlap = false;
                for row in 0..16 as u16 {
                    let sprite_byte = self.memory.ram[self.I as usize + row as usize * 2];
                    for cell in 0..8 {
                        if self.quirks.edge_clipping
                            && (dx % width + cell > width - 1 || dy % height + row > height - 1)
                        {
                            break;
                        }

                        let sprite_pixel = sprite_byte & (0b10000000 >> cell) != 0;

                        let target_pixel =
                            ((dx + cell) % width + (dy + row) % height * width) as usize;

                        if sprite_pixel {
                            if self.display.pixels[target_pixel] {
                                overlap = true;
                            }
                            self.display.pixels[target_pixel] = !self.display.pixels[target_pixel];
                        }
                    }
                    let sprite_byte = self.memory.ram[self.I as usize + row as usize * 2 + 1];
                    for cell in 8..16 {
                        if self.quirks.edge_clipping
                            && (dx % width + cell > width - 1 || dy % height + row > height - 1)
                        {
                            break;
                        }

                        let sprite_pixel = sprite_byte & (0b10000000 >> (cell - 8)) != 0;

                        let target_pixel =
                            ((dx + cell) % width + (dy + row) % height * width) as usize;

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
            // Dxyn - Draw 8xn sprite at Vx, Vy from address I
            // Optionally wait for a vblank interrupt (quirk)
            0xD => {
                if self.quirks.wait_for_vblank && !self.vblank {
                    return;
                }

                /*
                    I tried to do this by actually XORing the target pixel with the sprite pixel for
                    a while, but I could not pass the clipping test. I always got ERR2 and I did not
                    know why.
                    I gave up and looked at how Octo does this. I copied the part before the pixel
                    setting, but it still did not work. I then copied the rest and run the test.

                    It passed.

                    I have no idea why this way works but my way did not.
                */

                let width = if self.highres { 128 } else { 64 };
                let height = if self.highres { 64 } else { 32 };

                let dx = self.V[x] as u16;
                let dy = self.V[y] as u16;

                let mut overlap = false;
                for row in 0..nibble as u16 {
                    let sprite_byte = self.memory.ram[self.I as usize + row as usize];
                    for cell in 0..8 {
                        if self.quirks.edge_clipping
                            && (dx % width + cell > width - 1 || dy % height + row > height - 1)
                        {
                            break;
                        }

                        let sprite_pixel = sprite_byte & (0b10000000 >> cell) != 0;

                        let target_pixel =
                            ((dx + cell) % width + (dy + row) % height * width) as usize;

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
                    if self.keypad[(self.V[x] & 0x0F) as usize] {
                        self.increment_program_counter();
                    }
                }
                // ExA1 - Skip if key Vx is up
                0xA1 => {
                    if !self.keypad[(self.V[x] & 0x0F) as usize] {
                        self.increment_program_counter();
                    }
                }
                _ => self.halt(format!("Illegal instruction: {:04X}", opcode)),
            },
            0xF => match byte {
                // Fx07 - Set Vx to delay
                0x07 => self.V[x] = self.delay,
                // Fx0A - Wait for a key pressed and released and set it to Vx
                0x0A => {
                    self.awaiting_key = true;
                    self.key_destination = x;
                }
                // Fx15 - Set delay to Vx
                0x15 => self.delay = self.V[x],
                // Fx18 - Set sound to Vx
                0x18 => self.sound = self.V[x],
                // Fx1E - Set I += Vx
                0x1E => self.I += self.V[x] as u16,
                // Fx29 - Set I to the address of the font sprite for Vx's lowest nibble
                0x29 => self.I = (self.V[x] as u16 & 0x000F) * 5,
                // Fx30 - Set I to the address of the large font sprite for Vx's lowest nibble (SUPER-CHIP)
                0x30 if self.variant.supports_schip() => {
                    self.I = (self.V[x] as u16 & 0x000F) * 10 + 16 * 5
                }
                // Fx33 - Write Vx as BCD to addresses I, I+1 and I+2
                0x33 => {
                    self.write_byte(self.I, self.V[x] / 100);
                    self.write_byte(self.I + 1, (self.V[x] / 10) % 10);
                    self.write_byte(self.I + 2, (self.V[x] % 100) % 10);
                }
                // Fx55 - Write V0 to Vx to addresses I to I+x, I is incremented by x
                // Or I is not incremented at all (quirk)
                0x55 => {
                    for i in 0..=x {
                        self.write_byte(self.I + i as u16, self.V[i]);
                    }
                    if !self.quirks.save_load_increment {
                        self.I += x as u16 + 1
                    }
                }
                // Fx65 - Read from addresses I to I+x to V0 to Vx, I is incremented by x
                // Or I is not incremented at all (quirk)
                0x65 => {
                    for i in 0..=x {
                        self.V[i] = self.read_byte(self.I + i as u16);
                    }
                    if !self.quirks.save_load_increment {
                        self.I += x as u16 + 1
                    }
                }
                // Fx75 - Save V0-Vx to persistent storage (SUPER-CHIP)
                0x75 if self.variant.supports_schip() => {
                    for i in 0..=x {
                        self.persistent_flags[i] = self.V[i];
                    }
                    self.save_persistent_flags();
                }
                // Fx85 - Load V0-Vx from persistent storage (SUPER-CHIP)
                0x85 if self.variant.supports_schip() => {
                    for i in 0..=x {
                        self.V[i] = self.persistent_flags[i];
                    }
                }
                _ => self.halt(format!("Illegal instruction: {:04X}", opcode)),
            },
            _ => self.halt(format!("Illegal instruction: {:04X}", opcode)),
        }
        self.increment_program_counter();
    }

    /// Stop execution in case of an exceptional event.
    pub fn halt(&mut self, reason: String) {
        self.stop();
        self.halt_message = Some(reason);
    }
}

/// Functions for state inspection.
impl Chip8 {
    /// Check if `running` is `true`. For the inspector.
    #[inline]
    pub const fn is_running(&self) -> bool {
        self.running
    }
    /// Get register V`i`. For the inspector.
    #[inline]
    pub const fn get_register(&self, i: usize) -> u8 {
        self.V[i]
    }
    /// Get register I. For the inspector.
    #[inline]
    pub const fn get_i(&self) -> u16 {
        self.I
    }
    /// Get the program counter. For the inspector.
    #[inline]
    pub const fn get_program_counter(&self) -> u16 {
        self.program_counter
    }
    /// Get the stack pointer. For the inspector.
    #[inline]
    pub const fn get_stack_pointer(&self) -> u8 {
        self.stack_pointer
    }
    /// Get the length of the stack. 12 for CHIP-8, 16 for SUPER-CHIP and XO-CHIP. For the inspector.
    #[inline]
    pub const fn get_stack_size(&self) -> usize {
        self.stack_size
    }
    /// Get the `i`th value in the stack. For the inspector.
    #[inline]
    pub fn read_stack(&self, i: usize) -> u16 {
        self.stack[i]
    }
    /// Get the delay timer. For the inspector.
    #[inline]
    pub const fn get_delay(&self) -> u8 {
        self.delay
    }
    /// Get the sound timer. For the inspector.
    #[inline]
    pub const fn get_sound(&self) -> u8 {
        self.sound
    }
    /// Get the length of RAM. For the inspector.
    #[inline]
    pub const fn ram_len(&self) -> usize {
        self.memory.ram.len()
    }
    /// Get the index of the register where the next key press will be saved as a result of the Fx0A instruction.
    /// For the inspector.
    #[inline]
    pub const fn get_key_destination_register(&self) -> usize {
        self.key_destination
    }
    /// Get the state of key `i` on the keypad. For the inspector.
    #[inline]
    pub const fn get_key_state(&self, key: usize) -> bool {
        self.keypad[key]
    }
    /// Check if the interpreter is waiting for a key press with the Fx0A instruction. For the inspector.
    #[inline]
    pub const fn is_waiting_for_key(&self) -> bool {
        self.awaiting_key
    }
    /// Get SUPER-CHIP persistent flags. For the inspector.
    #[inline]
    pub const fn get_persistent_flags(&self) -> [u8; 8] {
        self.persistent_flags
    }
    /// Set all persistent flags to zero.
    #[inline]
    pub fn clear_persistent_flags(&mut self) {
        self.persistent_flags = [0; 8];
        self.save_persistent_flags();
    }
}
