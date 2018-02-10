extern crate encoding;

#[macro_use]
extern crate nom;

mod rom;
mod interconnect;

use nom::{IResult, eof, space, digit};

use rom::*;
use interconnect::*;

use std::env;
use std::io::{stdin, stdout, Write};
use std::borrow::Cow;
use std::str::{self, FromStr};
use std::fmt;

#[derive(PartialEq, Eq)]
enum Opcode {
    Movhi,
    Movea,
    Jmp,
    Outw,
}

impl Opcode {
    fn from_halfword(halfword: u16) -> Opcode {
        let opcode_bits = halfword >> 10;
        match opcode_bits {
            0b101111 => Opcode::Movhi,
            0b101000 => Opcode::Movea,
            0b000110 => Opcode::Jmp,
            0b111111 => Opcode::Outw,
            _ => panic!("Unrecognized opcode bits: {:06b}", opcode_bits)
        }
    }

    fn instruction_format(&self) -> InstructionFormat {
        match self {          
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::Movhi => "movhi",
            &Opcode::Movea => "movea",
            &Opcode::Jmp => "jmp",
            &Opcode::Outw => "outw",
        };
        write!(f, "{}", mnemonic)
    }
}

#[derive(Debug)]
enum InstructionFormat {
    I,
    V,
    VI
}

impl InstructionFormat {
    fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Disassemble(usize),
    Exit,
    Repeat,
}

impl FromStr for Command {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match command(s.as_bytes()) {
            IResult::Done(_, c) => Ok(c),
            err => Err(format!("Unable to parse command: {:?}", err).into()),
        }
    }
}

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    println!("\n--------------------");
    println!("\nAurora VB Emulator");
    println!("\n--------------------");

    println!("\nLoading ROM file '{}'", rom_file_name);

    let rom = Rom::load(rom_file_name).unwrap();

    println!("\nHeader info:");

    println!("\nGame: {}", rom.name().unwrap());
    println!("\nMaker code: {}", rom.maker_code().unwrap());
    println!("\nGame code: {}", rom.game_code().unwrap());
    println!("\nGame version: 1.{:#02}\n", rom.game_version());

    let interconnect = Interconnect::new(rom);

    let mut cursor = 0xfffffff0;

    let mut last_command = None;

    loop {
        print!("Aurora VB: ");

        stdout().flush().unwrap();

        let command = match (read_stdin().parse(), last_command) {
            (Ok(Command::Repeat), Some(c)) => Ok(c),
            (Ok(Command::Repeat), None) => Err("No last command".into()),
            (Ok(c), _) => Ok(c),
            (Err(e), _) => Err(e),
        };

        match command {
            Ok(Command::Disassemble(count)) => {
                for _ in 0..count {
                    print!("0x{:#08x} ", cursor);

                    let first_halfword = interconnect.read_halfword(cursor);
                    cursor = cursor.wrapping_add(2); // jump through distination & source registers.

                    print!("{:02x}{:02x}", first_halfword & 0xff, first_halfword >> 8);

                    let opcode = Opcode::from_halfword(first_halfword);

                    let instruction_format = opcode.instruction_format();

                    let second_halfword = if instruction_format.has_second_halfword() {
                        let second_halfword = interconnect.read_halfword(cursor);
                        print!("{:02x}{:02x}", second_halfword & 0xff, second_halfword >> 8);

                        cursor = cursor.wrapping_add(2);
                        second_halfword
                    } else {
                        print!("      ");
                        0
                    };

                    print!("      ");

                    match instruction_format {
                        InstructionFormat::I => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                            if opcode == Opcode::Jmp {
                                println!("jmp [r{}]", reg1);
                            } else {
                                println!("{}, r{}, r{}", opcode, reg1, reg2)    
                            }

                            let imm16 = second_halfword;

                            println!("{}, {:#x}, r{}, r{}", opcode, imm16, reg1, reg2)
                        }

                        InstructionFormat::V => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                            let imm16 = second_halfword;

                            println!("{} {:#x}, r{}, r{}", opcode, imm16, reg1, reg2)
                        }

                        InstructionFormat::VI => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                            let disp16 = second_halfword as i16;

                            println!("{} {}[r{}], r{}", opcode, disp16, reg1, reg2)
                        }
                    }
                }
            }
            Ok(Command::Exit) => break,
            Ok(Command::Repeat) => unreachable!(),
            Err(ref e) => println!("{}", e),
        }

        if let Ok(c) = command {
            last_command = Some(c);
        }
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}

named!(
    command<Command>,
    complete!(
        terminated!(
            alt_complete!(
                disassemble | exit | repeat
            ),
            eof
        )
    )
);

named!(
    disassemble<Command>,
    chain!(
        alt_complete!(
            tag!("disassemble") | tag!("d")
        ) ~ count: opt!(preceded!(
            space,
            usize_parser
        )), 
        || Command::Disassemble(count.unwrap_or(4))
    )
);

named!(
    exit<Command>,
    map!(
        alt_complete!(
            tag!("exit") | tag!("quit") | tag!("q") | tag!("e")
        ),
        |_| Command::Exit
    )
);

named!(
    repeat<Command>,
    value!(Command::Repeat)
);

named!(
    usize_parser<usize>,
    map_res!(
        map_res!(
            digit, 
            str::from_utf8
        ),
        FromStr::from_str
    )
);
