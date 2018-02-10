use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    Movhi,
    Movea,
    MovImm,
    Stb,
    Jmp,
    Sub,
    Outw,
}

impl Opcode {
    pub fn from_halfword(halfword: u16) -> Opcode {
        let opcode_bits = halfword >> 10;
        match opcode_bits {
            0b101111 => Opcode::Movhi,
            0b101000 => Opcode::Movea,
            0b010000 => Opcode::MovImm,
            0b110100 => Opcode::Stb,
            0b000110 => Opcode::Jmp,
            0b000010 => Opcode::Sub,
            0b111111 => Opcode::Outw,
            _ => panic!("Unrecognized opcode bits: {:06b}", opcode_bits)
        }
    }

    pub fn instruction_format(&self) -> InstructionFormat {
        match self {          
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::Sub => InstructionFormat::I,
            &Opcode::MovImm => InstructionFormat::II,
            &Opcode::Outw => InstructionFormat::VI,
            &Opcode::Stb => InstructionFormat::VI,
        }
    }

    pub fn num_cycles(&self) -> usize {
        match self {
            &Opcode::Jmp => 3,
            &Opcode::MovImm => 1,
            &Opcode::Sub => 1,
            &Opcode::Movea => 1,
            &Opcode::Movhi => 1,
            &Opcode::Stb => 1,
            &Opcode::Outw => 1,
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::Sub => "sub",
            &Opcode::Movhi => "movhi",
            &Opcode::Movea => "movea",
            &Opcode::MovImm => "mov",
            &Opcode::Jmp => "jmp",
            &Opcode::Outw => "out.w",
            &Opcode::Stb => "st.b",
        };
        write!(f, "{}", mnemonic)
    }
}

#[derive(Debug)]
pub enum InstructionFormat {
    I,
    II,
    V,
    VI
}

impl InstructionFormat {
    pub fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::II => false,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
        }
    }
}
