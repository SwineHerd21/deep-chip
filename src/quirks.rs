/// The desired quirks of the CHIP-8 interpreter.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Quirks {
    /// If `true`, the `8xy1`, `8xy2` and `8xy3` opcodes will set VF to 0.  
    /// If `false`, the `8xy1`, `8xy2` and `8xy3` opcodes will not modify VF.
    pub bitwise_reset_vf: bool,
    /// If `true`, the `8xy6` and `8xyE` opcodes will set Vx to Vx >> 1.  
    /// If `false`, the `8xy6` and `8xyE` opcodes will set Vx to Vy >> 1.
    pub direct_shifting: bool,
    /// If `true`, the `Fx55` and `Fx65` opcodes will not modify I.  
    /// If `false`, the `Fx55` and `Fx65` opcodes will set I to I + x + 1.
    pub save_load_increment: bool,
    /// If `true`, the `Bnnn` opcode will jump to nnn + V0.  
    /// If `false`, the `Bnnn` opcode will jump to nnn + Vx.
    pub jump_to_x: bool,
    /// If `true`, the `Dxyn` opcode will wait for a vblank interrupt before drawing.  
    /// If `false`, the `Dxyn` opcode will draw immediately.
    pub wait_for_vblank: bool,
    /// If `true`, the `Dxyn` opcode will clip sprites that go off the edge of the screen.  
    /// If `false`, the `Dxyn` opcode will wrap sprites that go off the edge of the screen around.
    pub edge_clipping: bool,
}

impl Quirks {
    /// The quirks of the original CHIP-8 implementation on the COSMAC-VIP.  
    ///
    /// - bitwise_reset_vf: true
    /// - direct_shifting: false
    /// - save_load_increment: false
    /// - jump_to_x: false
    /// - wait_for_vblank: true
    /// - edge_clipping: true
    pub const fn vip_chip() -> Quirks {
        Quirks {
            bitwise_reset_vf: true,
            direct_shifting: false,
            save_load_increment: false,
            jump_to_x: false,
            wait_for_vblank: true,
            edge_clipping: true,
        }
    }

    /// The default quirk configuration of the Octo CHIP-8 emulator.  
    ///
    /// - bitwise_reset_vf: false
    /// - direct_shifting: false
    /// - save_load_increment: false
    /// - jump_to_x: false
    /// - wait_for_vblank: false
    /// - edge_clipping: false
    pub const fn octo_chip() -> Quirks {
        Quirks {
            bitwise_reset_vf: false,
            direct_shifting: false,
            save_load_increment: false,
            jump_to_x: false,
            wait_for_vblank: false,
            edge_clipping: false,
        }
    }

    /// The quirks of the SUPER-CHIP 1.1.  
    ///
    /// - bitwise_reset_vf: false
    /// - direct_shifting: true
    /// - save_load_increment: true
    /// - jump_to_x: true
    /// - wait_for_vblank: false
    /// - edge_clipping: true
    pub const fn super_chip1_1() -> Quirks {
        Quirks {
            bitwise_reset_vf: false,
            direct_shifting: true,
            save_load_increment: true,
            jump_to_x: true,
            wait_for_vblank: false,
            edge_clipping: true,
        }
    }
}

/// Determines what CHIP-8 variant to run as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variant {
    /// Run as a CHIP-8 interpreter
    CHIP8,
    /// Run as a SUPER-CHIP 1.1 interpreter
    SCHIP11,
    /// Run as an XO-CHIP interpreter (not implemented)
    XOCHIP,
}

impl Variant {
    /// Check whether the variant supports all features introduced by SUPEP-CHIP
    #[inline]
    pub const fn supports_schip(&self) -> bool {
        match self {
            Variant::CHIP8 => false,
            Variant::SCHIP11 => true,
            Variant::XOCHIP => true,
        }
    }
}
