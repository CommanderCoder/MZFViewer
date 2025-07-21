// src/lib.rs

mod z80_disasm;
use z80_disasm::Z80Disassembler;
mod zx80_decoder;
mod zx81_decoder;
mod mz_decoder;

use mz_decoder::MZBasicVersion;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;


/// Enum representing different processing modes.
#[derive(Debug, Clone, PartialEq, Copy)]
enum MZFEncoding {
    SA5510,
    SP5025,
    V1Z013B, // 1Z-013B BASIC version
    Z80,     // Z80 disassembly
    DUMP,    // Hexadecimal output
    ZX80BASIC, // Sinclair ZX80 Basic
    ZX81BASIC, // Sinclair ZX81 Basic
}

#[wasm_bindgen]
pub enum MZFMachine {
    Sharp,
    Sinclair,
}

/// Struct for handling non-BASIC operations (DUMP mode and Sharp ASCII mapping)
struct MZLowerCase
{
    sharp_ascii: HashMap<u8, char>,
}

impl MZLowerCase
 {
    fn new() -> Self {
        let mut sharp_ascii = HashMap::new();
        // Mapping for specific Sharp ASCII characters
        let ascii_map = [
            (146, 'e'), (150, 't'), (151, 'g'), (152, 'h'), (154, 'b'), (155, 'x'), (156, 'd'),
            (157, 'r'), (158, 'p'), (159, 'c'), (160, 'q'), (161, 'a'), (162, 'z'), (163, 'w'),
            (164, 's'), (165, 'u'), (166, 'i'), (169, 'k'), (170, 'f'), (171, 'v'), (175, 'j'),
            (176, 'n'), (179, 'm'), (183, 'o'), (184, 'l'), (189, 'y')
        ];
        
        for (byte, ch) in ascii_map {
            sharp_ascii.insert(byte, ch);
        }

        Self { sharp_ascii }
    }
}

/// WASM-exposed function to process a binary file and detokenize it.
///
/// # Arguments
/// * `data` - A slice of unsigned 8-bit integers (bytes) representing the binary file content.
/// * `mode` - A string indicating the desired BASIC version for detokenization:
///            "SA" for SA-5510, "SP" for SP-5025, "1Z" for 1Z-013B, 
///            "Z80" for Z80 disassembly, "DUMP" for hexadecimal output,
///            "ZX80BASIC" for ZX80 Basic, "ZX81BASIC" for ZX81 Basic.
/// * `machine` : type of machine to process binary
/// * `charset_flag` - A boolean indicating whether to use the ASCII character set for detokenization.
///
/// # Returns
/// A `String` containing the detokenized BASIC listing or an error message.
/// This needs to be safe HTML as it will be interpreted by browser for INV and Special characters
#[wasm_bindgen]
pub fn process_binary(data: &[u8], mode: String, machine: MZFMachine, charset_flag: bool) -> String {
    // Determine the processing mode based on the selected mode.
    let version = match mode.as_str() {
        "SA" => MZFEncoding::SA5510,
        "SP" => MZFEncoding::SP5025,
        "1Z" => MZFEncoding::V1Z013B,
        "Z80" => MZFEncoding::Z80,
        "DUMP" => MZFEncoding::DUMP,
        "ZX80BASIC" => MZFEncoding::ZX80BASIC,
        "ZX81BASIC" => MZFEncoding::ZX81BASIC,
        _ => return "Error: Invalid mode specified. Expected (SA, SP, 1Z, Z80, DUMP, ZX80BASIC, ZX81BASIC)".to_string(),
    };

    match version {
        MZFEncoding::Z80 => {
            let (skip_bytes, start_address, exec_address) = match machine {
                MZFMachine::Sharp => (128,
                    u16::from_le_bytes([data[0x14], data[0x15]]), // default start address is found at bytes 0x14,0x15 (LE)
                    u16::from_le_bytes([data[0x16], data[0x17]]), // default exec address is found at bytes 0x16,0x17 (LE)
                ),
                MZFMachine::Sinclair => (0, 0, 0)
            };
            let lowercase = MZLowerCase::new();

            // Create a dummy decoder for Z80 disassembly (it uses the Sharp ASCII mapping)
            let mut disasm = Z80Disassembler::new();
            let result = disasm.disassemble(&data[skip_bytes..], start_address, exec_address);

            if charset_flag {
                result
                .iter()
                    .map(|line| {
                        line.chars()
                            .map(|c| if c.is_ascii_graphic() || c == ' ' { c } else { 
                                lowercase.sharp_ascii.get(&(c as u8)).copied().unwrap_or('.')
                            })
                            .collect::<String>()
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }else{
                result.join("\n")
            }
        },
        
        MZFEncoding::DUMP => {
            let lowercase = MZLowerCase::new();
            let mut hex_output = String::new();
            
            for (i, byte) in data.iter().enumerate() {
                // each line has 16 bytes
                if i % 16 == 0 {
                    if i > 0 {
                        hex_output.push_str("\n");
                    }
                    // current location in dump (every 16 bytes)
                    hex_output.push_str(&format!("{:04X}: ", i));
                }
                hex_output.push_str(&format!("{:02X} ", byte));
                if i % 16 == 15 || i == data.len() - 1 {
                    let text_part: String = data[i - (i % 16)..=i]
                        .iter()
                        .map(|&b| { 
                            if b >= 32 && b < 127 {
                                // If 'b' is a standard ASCII printable character, use it directly.
                                b as char
                            } else if charset_flag {
                                // If 'charset_flag' is true, and 'b' isn't printable ASCII, default to a full-stop.
                                '.'
                            } else {
                                // Otherwise (charset_flag is false, and 'b' isn't printable ASCII),
                                // use the machine-specific character set.
                                match machine {
                                    MZFMachine::Sharp => {
                                        if let Some(&ch) = lowercase.sharp_ascii.get(&b) {
                                            ch
                                        } else {
                                            '.'
                                        }
                                    }
                                    MZFMachine::Sinclair => '_'
                                }
                            }
                        })
                        .collect();

                    hex_output.push_str(&format!(" | {}", text_part));
                }
            } 
            hex_output 
        },
        
        MZFEncoding::ZX80BASIC => {
            match zx80_decoder::decode_zx80_bytes(data, false) {
                Ok(basic_listing) => basic_listing,
                Err(e) => format!("Error detokenizing file: {}", e),
            }
        },
        
        MZFEncoding::ZX81BASIC => {
            match zx81_decoder::decode_zx81_bytes(data) {
                Ok(basic_listing) => basic_listing,
                Err(e) => format!("Error detokenizing file: {}", e),
            }
        },
        
        // Handle MZ BASIC versions
        MZFEncoding::SA5510 | MZFEncoding::SP5025 | MZFEncoding::V1Z013B => {
            let mz_version = match version {
                MZFEncoding::SA5510 => MZBasicVersion::SA5510,
                MZFEncoding::SP5025 => MZBasicVersion::SP5025,
                MZFEncoding::V1Z013B => MZBasicVersion::V1Z013B,
                _ => unreachable!(),
            };
            
            match mz_decoder::decode_mz_bytes(data, mz_version) {
                Ok(basic_listing) => basic_listing,
                Err(e) => format!("Error detokenizing file: {}", e),
            }
        }
    }
}