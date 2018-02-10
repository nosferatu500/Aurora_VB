use encoding::DecoderTrap;
use encoding::all::WINDOWS_31J;
use encoding::types::EncodingRef;

use std::path::Path;
use std::fs::File;

use std::io::{self, Read, Error, ErrorKind};

use std::borrow::Cow;

use std::string::FromUtf8Error;

const MAX_ROM_SIZE: usize = 16777216; // 16 Mb.

pub struct Rom {
    bytes: Box<[u8]>
}

impl Rom {
    pub fn load<P: AsRef<Path>>(rom_file_name: P) -> io::Result<Rom> {
        let mut rom_buf = Vec::new();
        let mut rom_file = File::open(&rom_file_name)?;

        rom_file.read_to_end(&mut rom_buf)?;

        let rom_size = rom_buf.len();

        if rom_size > MAX_ROM_SIZE {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid ROM size."));
        }

        Ok(Rom {
            bytes: rom_buf.into_boxed_slice()
        })
    }

    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn header_offset(&self) -> usize {
        self.size() - 544
    }

    pub fn name(&self) -> Result<String, Cow<'static, str>> {
        let name_offset = self.header_offset();
        let name_bytes = &self.bytes[name_offset..name_offset + 0x14];
        let encoding = WINDOWS_31J as EncodingRef;
        
        encoding.decode(name_bytes, DecoderTrap::Strict)
    }

    pub fn maker_code(&self) -> Result<String, FromUtf8Error> {
        let name_offset = self.header_offset();
        let maker_code_offset = name_offset + 0x19;
        let maker_code_bytes = &self.bytes[maker_code_offset..maker_code_offset + 2];

        let mut maker_code_vec = Vec::new();
        maker_code_vec.extend_from_slice(maker_code_bytes);

        String::from_utf8(maker_code_vec)
    }

    pub fn game_code(&self) -> Result<String, FromUtf8Error> {
        let name_offset = self.header_offset();
        let game_code_offset = name_offset + 0x1b;
        let game_code_bytes = &self.bytes[game_code_offset..game_code_offset + 2];
        
        let mut game_code_vec = Vec::new();
        game_code_vec.extend_from_slice(game_code_bytes);

        String::from_utf8(game_code_vec)
    }

    pub fn game_version(&self) -> u8 {
        let name_offset = self.header_offset();
        self.bytes[name_offset + 0x1f]
    }
}
