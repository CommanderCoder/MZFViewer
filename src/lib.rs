// src/lib.rs

mod z80_disasm;
use z80_disasm::Z80Disassembler;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::io::{self}; 

// Token tables for SA-5510
const TOKENS1: [&str; 56] = [
    "REM", "DATA", "", "", "READ", "LIST", "RUN", "NEW", "PRINT", "LET", "FOR",
    "IF", "THEN", "GOTO", "GOSUB", "RETURN", "NEXT", "STOP", "END", "", "ON",
    "LOAD", "SAVE", "VERIFY", "POKE", "DIM", "DEF FN", "INPUT", "RESTORE", "CLR",
    "MUSIC", "TEMPO", "USR(", "WOPEN", "ROPEN", "CLOSE", "MON", "LIMIT", "CONT",
    "GET", "INP#", "OUT#", "CURSOR", "SET", "RESET", "", "", "", "", "", "", "AUTO",
    "", "", "COPY/P", "PAGE/P"
];

const TOKENS2: [&str; 76] = [
    "", "", "", "><", "<>", "=<", "<=", "=>", ">=", "", ">", "<", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "TO", "STEP", "LEFT$(", "RIGHT$(", "MID$(", "LEN(", "CHR$(", "STR$(", "ASC(", "VAL(", "PEEK(", "TAB(", "SPACE$(",
    "SIZE", "", "", "", "STRING$(", "", "CHARACTER$(", "CRS", "CRS", "", "", "", "", "","", "", "","", "", "", "","RND(", "SIN(", "COS(", "TAN(",
    "ATN(", "EXP(", "INT(", "LOG(", "LN(", "ABS(", "SGN(", "SQR("
];

// Token tables for SP-5025
const TOKENS1_SP5025: [&str; 92] = [
    "REM", "DATA", "LIST", "RUN", "NEW", "PRINT", "LET", "FOR", "IF", "GOTO", "READ",
    "GOSUB", "RETURN", "NEXT", "STOP", "END", "ON", "LOAD", "SAVE", "VERIFY", "POKE", "DIM",
    "DEF FN", "INPUT", "RESTORE", "CLR", "MUSIC", "TEMPO", "USR(", "WOPEN", "ROPEN", "CLOSE", "BYE",
    "LIMIT", "CONT", "SET", "RESET", "GET", "INP#", "OUT#", "", "", "", "",
    "", "THEN", "TO", "STEP", "><", "<>", "=<", "<=", "=>", ">=", "=", ">", "<",
    "AND", "OR", "NOT", "+", "-", "*", "/", "LEFT$(", "RIGHT$(", "MID$(", "LEN(", "CHR$(",
    "STR$(", "ASC(", "VAL(", "PEEK(", "TAB(", "SP(", "SIZE", "", "", "", "^", "RND(",
    "SIN(", "COS(", "TAN(", "ATN(", "EXP(", "INT(", "LOG(", "LN(", "ABS(", "SGN(", "SQR("
];

// Token tables for 1Z-013B BASIC
const TOKENS_1Z013B: [&str; 128] = [
    "GOTO", "GOSUB" , "", "RUN", "RETURN", "RESTORE", "RESUME", "LIST", "", "DELETE", "RENUMBER", "AUTO", "", "FOR", "NEXT", "PRINT",
    "", "INsPUT", "", "IF", "DATA", "READ", "DIM", "REM", "END", "STOP", "CONT", "CLS", "", "ON", "LET", "NEW",
    "POKE", "OFF", "MODE", "SKIP", "PLOT", "LINE", "RLINE", "MOVE", "RMOVE", "TRON", "TROFF", "INP#", "", "GET", "PCOLOR", "PHOME",
    "HSET", "GPRINT", "KEY", "AXIS", "LOAD", "SAVE", "MERGE", "", "CONSOLE", "", "OUT", "CIRCLE", "TEST", "PAGE", "", "",
    "ERASE", "ERROR", "", "USR", "BYE", "", "", "DEF", "", "", "", "", "", "", "WOPEN", "CLOSE",
    "ROPEN", "", "", "", "", "", "", "", "", "KILL", "", "", "", "", "", "",
    "TO", "STEP", "THEN", "USING", "", "", "TAB", "SPC", "", "", "", "OR", "AND", "", "><", "<>",
    "=<", "<=", "=>", ">=", "=", ">", "<", "+", "-", "", "", "/", "*", "^","ext1", "ext2"
];

const TOKENS_1Z013B_E1: [&str; 48] =  [
    "", "SET", "RESET", "COLOR", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "MUSIC", "TEMPO", "CURSOR", "VERIFY", "CLR", "LIMIT", "", "", "", "", "", "", "BOOT", ""
];

const TOKENS_1Z013B_E2: [&str; 72] =  [
    "INT", "ABS", "SIN", "COS", "TAN", "LN", "EXP", "SQR", "RND", "PEEK", "ATN", "SGN", "LOG", "PAI", "", "RAD",
    "", "", "", "", "", "EOF", "", "", "", "", "", "", "", "", "JOY", "",
    "", "STR$", "HEX$", "", "", "", "", "", "", "", "", "ASC", "LEN", "VAL", "", "",
    "", "", "", "ERN", "ERL", "SIZE", "", "", "", "", "LEFT$", "RIGHT$", "MID$", "", "", "",
    "", "", "", "", "TI$", "", "", "FN"
];



/// Enum representing different BASIC versions.
#[derive(Debug, Clone, PartialEq, Copy)] // Add PartialEq for comparison
enum MZFEncoding {
    SA5510,
    SP5025,
    V1Z013B, // 1Z-013B BASIC version
    Z80,     // Z80 disassembly
}


/// Struct responsible for detokenizing MZ-series BASIC code.
struct MZDetokenizer {
    sharp_ascii: HashMap<u8, char>,
    string_literal_map: HashMap<u8, &'static str>,
    version: MZFEncoding,
}

impl MZDetokenizer {
    /// Creates a new MZDetokenizer instance for a given BASIC version.
    fn new(version: MZFEncoding) -> Self {
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

        let mut string_literal_map = HashMap::new();
        // Mapping for special control characters within string literals
        string_literal_map.insert(0x0D, "↵"); // Carriage Return
        string_literal_map.insert(0x10, "⌫"); // Backspace
        string_literal_map.insert(0x11, "↓"); // Down Arrow
        string_literal_map.insert(0x12, "↑"); // Up Arrow
        string_literal_map.insert(0x13, "→"); // Right Arrow
        string_literal_map.insert(0x14, "←"); // Left Arrow
        string_literal_map.insert(0x15, "⌂"); // Home
        string_literal_map.insert(0x16, "🅲"); // Clear
        string_literal_map.insert(0x18, "⎀"); // Cursor Home

        Self {
            sharp_ascii,
            string_literal_map,
            version,
        }
    }

    /// Returns the appropriate token tables based on the detected BASIC version.
    fn get_token_tables(&self) -> (&[&str], &[&str], &[&str]) {
        match self.version {
            MZFEncoding::SP5025 => (&TOKENS1_SP5025, &[], &[]),
            MZFEncoding::V1Z013B => (&TOKENS_1Z013B, &TOKENS_1Z013B_E1, &TOKENS_1Z013B_E2), // 1Z-013B BASIC tokens
            _ => (&TOKENS1, &TOKENS2, &[]), // Default to SA-5510 tokens if not SP5025
        }
    }

    // Read a single byte from the data stream
    fn read_u8(data: &[u8], offset: &mut usize) -> io::Result<u8> {
        if *offset < data.len() {
            let byte = data[*offset];
            *offset += 1;
            Ok(byte)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of data"))
        }
    }

    fn read_u16(data: &[u8], offset: &mut usize) -> io::Result<u16> {
        if *offset + 2 <= data.len() {
            let value = u16::from_le_bytes([data[*offset], data[*offset + 1]]);
            *offset += 2;
            Ok(value)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of data"))
        }
    }
    /// Detokenizes the BASIC code from the provided binary data.
    /// This is the core logic for converting the binary tokens into human-readable BASIC.
    fn detokenise_basic(&self, data: &[u8]) -> io::Result<String> {
        let mut output = String::new();
        let (tokens1, tokens2, tokens3,) = self.get_token_tables();
        let mut offset = 128; // Start after the header

        loop {
            let line_length = match Self::read_u16(data, &mut offset) {
                Ok(len) => len,
                Err(_) => break,
            };

            if line_length == 0 {
                break;
            }

            let lineno = Self::read_u16(data, &mut offset)?;
            let mut line = format!("{} ", lineno);

            let mut quote = false;
            let mut token = false;
            let mut literal_mode = false;
            let mut line_end = false;
            let mut bytes_read = 4;

            while !line_end && bytes_read < line_length {
                let byte = Self::read_u8(data, &mut offset)?;
                bytes_read += 1;

                if literal_mode {
                    if byte == 0x0D || byte == 0x00{
                        line.push('\n');
                    } else if let Some(&ch) = self.sharp_ascii.get(&byte) {
                        line.push(ch);
                    } else if (0x20..=0x7E).contains(&byte) {
                        line.push(byte as char);
                    } else {
                        line.push('◇');
                    }
                    continue;
                }

                match byte {
                    0x00 | 0x0D => { 
                        line.push('\n');
                        line_end = true;
                    }
                    0x0B | 0x0C if !quote => {
                        let more = Self::read_u16(data, &mut offset)?;
                        bytes_read += 2;
                        line.push_str(&more.to_string());
                    }
                    0x11 if !quote => {
                        let more = Self::read_u16(data, &mut offset)?;
                        bytes_read += 2;
                        line.push_str(&format!("${:X}", more));
                    }
                    0x15 if !quote => {
                        let exponent = Self::read_u8(data, &mut offset)?;
                        bytes_read += 1;
                        
                        let exp_val = if exponent & 0x80 != 0 {
                            (exponent - 0x80) as i32
                        } else if exponent != 0 {
                            (0x80 - exponent) as i32
                        } else {
                            0
                        };
                        
                        let mut mantissa = 0.0;
                        let mut count = 1;
                        
                        for _ in 0..4 {
                            let b = Self::read_u8(data, &mut offset)?;
                            bytes_read += 1;
                            for j in (1..=7).rev() {
                                if b & (1 << j) != 0 {
                                    mantissa += 2.0_f64.powi(-(count as i32));
                                }
                                count += 1;
                            }
                        }
                        
                        mantissa += 0.5;
                        let fp = if exponent != 0 {
                            2.0_f64.powi(exp_val) * mantissa
                        } else {
                            0.0
                        };
                        
                        line.push_str(&fp.to_string());
                    }
                    b if b >= 0x80 && !quote && !token => {
                        match self.version {
                            MZFEncoding::SP5025 => {
                                let tok = (b - 0x80) as usize;
                                if tok < tokens1.len() {
                                    line.push_str(tokens1[tok]);
                                }
                                if b == 0x80 || b == 0x81 {
                                    literal_mode = true;
                                }
                            }
                            MZFEncoding::V1Z013B => {
                                let mut tok = (b - 0x80) as usize;
                                if b == 0xfe || b == 0xff {
                                    let next_byte = Self::read_u8(data, &mut offset)?;
                                    bytes_read += 1;
                                    tok = (next_byte - 0x80) as usize;
                                }
                                match b {
                                    0xfe if tok < tokens2.len() => line.push_str(tokens2[tok]),
                                    0xff if tok < tokens3.len() => line.push_str(tokens3[tok]),
                                    _ if b >= 0x80 => line.push_str(tokens1[tok]),
                                    _ => line.push_str(&format!(":0x{:02X} 0x{:02X}]", b, tok)),
                                }
                                if b == 0x97 || b == 0x94 {
                                    literal_mode = true;
                                }

                            }
                            _ => {
                                if b == 0x80 {
                                    let next_byte = Self::read_u8(data, &mut offset)?;
                                    bytes_read += 1;
                                    if next_byte == 0x80 {
                                        line.push_str("REM");
                                        literal_mode = true;
                                    } else if next_byte == 0x81 {
                                        line.push_str("DATA");
                                        literal_mode = true;
                                    } else {
                                        let tok = (next_byte - 0x80) as usize;
                                        if tok < tokens1.len() {
                                            line.push_str(tokens1[tok]);
                                        }
                                        if next_byte == 0x80 {
                                            token = true;
                                        }
                                    }
                                } else {
                                    let tok = (b - 0x80) as usize;
                                    if tok < tokens2.len() {
                                        line.push_str(tokens2[tok]);
                                    } else {
                                        line.push_str(&format!("[0x{:02X}]", b));
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        if byte == 0x22 {
                            quote = !quote;
                            line.push('"');
                        } else if quote {
                            if let Some(&literal) = self.string_literal_map.get(&byte) {
                                line.push_str(literal);
                            } else if let Some(&ch) = self.sharp_ascii.get(&byte) {
                                line.push(ch);
                            } else if (0x20..=0x7E).contains(&byte) {
                                line.push(byte as char);
                            }
                        } else if let Some(&ch) = self.sharp_ascii.get(&byte) {
                            line.push(ch);
                        } else if (0x20..=0x7E).contains(&byte) {
                            line.push(byte as char);
                        }

                        if byte == 0x3A {
                            token = false;
                        }
                    }
                }
            }

            output.push_str(&line);
        }

        Ok(output)
    }


}


/// WASM-exposed function to process a binary file and detokenize it.
///
/// # Arguments
/// * `data` - A slice of unsigned 8-bit integers (bytes) representing the binary file content.
/// * `mode` - A string indicating the desired BASIC version for detokenization:
///            "hex" for SA-5510 (hexadecimal output from detokenizer, but detokenizer uses SA-5510 rules).
///            "ascii" for SP-5025 (ASCII output from detokenizer, but detokenizer uses SP-5025 rules).
///
/// # Returns
/// A `String` containing the detokenized BASIC listing or an error message.
#[wasm_bindgen]
pub fn process_binary(data: &[u8], mode: String) -> String {
    // Determine the BASIC version based on the selected mode.
    let version = match mode.as_str() {
        "SA" => MZFEncoding::SA5510, // SA for SA-5510 detokenization
        "SP" => MZFEncoding::SP5025, // SP for SP-5025 detokenization
        "1Z" => MZFEncoding::V1Z013B, // 1Z for 1Z-013B detokenization
        "Z80" => MZFEncoding::Z80,     // Z80 for Z80 disassembly
        _ => return "Error: Invalid mode specified. Use 'SA' (SA-5510) or 'SP' (SP-5025) or '1Z' (1Z-013B).".to_string(),
    };

    let detokenizer = MZDetokenizer::new(version);

    if version == MZFEncoding::Z80 {
        let skip_bytes = 128; // No bytes to skip for Z80 disassembly
        let start_address = u16::from_le_bytes([data[0x14], data[0x15]]); // default start address is found at bytes 0x14,0x15 (LE)

        // Disassemble
        let mut disasm = Z80Disassembler::new();
        let result = disasm.disassemble(&data[skip_bytes..], start_address);

        // If the mode is Z80, disassemble the Z80 code.
        result.into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<String>>()
            .join("\n")

    } else {
        // Attempt to detokenize the BASIC code.
        match detokenizer.detokenise_basic(data) {
            Ok(basic_listing) => basic_listing,
            Err(e) => format!("Error detokenizing file: {}", e),
        }
    }
}
