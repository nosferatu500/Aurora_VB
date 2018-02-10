extern crate encoding;

use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use std::env;
use std::fs::File;
use std::io::Read;

const MAX_ROM_SIZE: usize = 16777216; // 16 Mb.

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    println!("\n--------------------");
    println!("\nAurora VB Emulator");
    println!("\n--------------------");

    println!("\nLoading ROM file '{}'", rom_file_name);

    let mut rom_buf = Vec::new();
    let mut rom_file = File::open(&rom_file_name).unwrap();

    rom_file.read_to_end(&mut rom_buf).unwrap();

    let rom_size = rom_buf.len();

    if rom_size > MAX_ROM_SIZE {
        panic!("Invalid ROM size.");
    }

    let header_offset = rom_size - 544;

    println!("\nHeader info:");

    // Game title
    let name_bytes = &rom_buf[header_offset..header_offset + 0x14];
    let encoding = encoding_from_whatwg_label("shift-jis").unwrap();
    let name = encoding.decode(name_bytes, DecoderTrap::Strict).unwrap();

    //Maker code
    let maker_code_offset = header_offset + 0x19;
    let maker_code_bytes = &rom_buf[maker_code_offset..maker_code_offset + 2];

    let mut maker_code_vec = Vec::new();
    maker_code_vec.extend_from_slice(maker_code_bytes);

    let maker_code = String::from_utf8(maker_code_vec).unwrap();

    //Game code
    let game_code_offset = header_offset + 0x1b;
    let game_code_bytes = &rom_buf[game_code_offset..game_code_offset + 2];
    
    let mut game_code_vec = Vec::new();
    game_code_vec.extend_from_slice(game_code_bytes);

    let game_code = String::from_utf8(game_code_vec).unwrap();

    //Game version
    let game_version_byte = &rom_buf[header_offset + 0x1f];    

    println!("\nGame: {}", name);
    println!("\nMaker code: {}", maker_code);
    println!("\nGame code: {}", game_code);
    println!("\nGame version: 1.{:#02}\n", game_version_byte);
}
