use std::collections::HashMap;


/// A decoder for ZX80 BASIC programs.
pub struct ZX80BasicDecoder {
    tokens: HashMap<u8, &'static str>,
    zx_to_ascii: HashMap<u8, char>,
    graphics: HashMap<u8, &'static str>,
    // zx_token_chars: HashMap<u8, char>,
    zxpand_enabled: bool,
}

impl ZX80BasicDecoder {
    /// Creates a new `ZX80BasicDecoder`.
    ///
    /// # Arguments
    ///
    /// * `zxpand_enabled` - A boolean to enable/disable ZXPAND specific tokens.
    pub fn new(zxpand_enabled: bool) -> Self {
        let mut decoder = ZX80BasicDecoder {
            tokens: HashMap::new(),
            zx_to_ascii: HashMap::new(),
            graphics: HashMap::new(),
            // zx_token_chars: HashMap::new(),
            zxpand_enabled,
        };
        decoder.initialize_tokens();
        decoder.initialize_character_maps();
        decoder
    }

    /// Initializes the token map.
    fn initialize_tokens(&mut self) {
        self.tokens.extend([
            (213, " THEN "), (214, " TO "), (219, "NOT "), (224, " AND "), (225, " OR "),
            (226, "**"), (230, " LIST "), (231, " RETURN "), (232, " CLS "), (233, " DIM "),
            (234, " SAVE "), (235, " FOR "), (236, " GO TO "), (237, " POKE "),
            (238, " INPUT "), (239, " RANDOMISE "), (240, " LET "), (243, " NEXT "),
            (244, " PRINT "), (246, " NEW "), (247, " RUN "), (248, " STOP "),
            (249, " CONTINUE "), (250, " IF "), (251, " GO SUB "), (252, " LOAD "),
            (253, " CLEAR "), (254, " REM "),
        ]);
        if self.zxpand_enabled {
            self.tokens.extend([(241, " CONFIG "), (245, " DELETE "), (255, " CAT ")]);
        }
    }

    /// Initializes the character and graphics maps.
    fn initialize_character_maps(&mut self) {
        // Numbers 0-9
        for i in 0..10 {
            self.zx_to_ascii.insert(28 + i, std::char::from_digit(i as u32, 10).unwrap());
        }
        // Letters A-Z
        for i in 0..26 {
            self.zx_to_ascii.insert(38 + i, (b'A' + i) as char);
        }
        self.zx_to_ascii.extend([
            (0, ' '), (1, '"'), (12, '£'), (13, '$'), (14, ':'), (15, '?'),
            (218, '('), (217, ')'), (220, '-'), (221, '+'), (222, '*'), (223, '/'),
            (227, '='), (228, '>'), (229, '<'), (215, ';'), (216, ','), (27, '.'),
        ]);
        self.graphics.extend([
            (0, "  "), (2, "▌"), (3, "▄"), (4, "▘"), (5, "▝"), (6, "▖"), (7, "▗"),
            (8, "▞"), (9, "▒"), (10, ",,"), (11, "~~"), (128, "::"), (130, " :"),
            (131, "''"), (132, ".:"), (133, ":."), (134, "':"), (135, ":'"),
            (136, "'.") ,(137, "@@"), (138, ";;"), (139, "!!"),
        ]);
        // self.zx_token_chars.extend([
        //     (2, 'º'), (3, '®'), (4, '¶'), (5, '·'), (6, '¹'), (7, '²'), (8, '»'), (9, '½'),
        //     (10, '¾'), (11, '¿'), (128, '«'), (129, '†'), (130, '°'), (131, '¸'),
        //     (132, '¬'), (133, 'ª'), (134, '¯'), (135, '¼'), (136, '±'), (137, '³'),
        //     (138, '´'), (139, 'µ'),
        // ]);
        // for i in 0..26 {
        //     self.zx_token_chars.insert(b'a' + i + 0x45, (b'a' + i) as char);
        // }
    }

    /// Decodes a single ZX80 character code.
    fn decode_character(&self, zx_char: u8) -> String {
        if zx_char & 0x80 != 0 {
            let base = zx_char & 0x7F;
            if let Some(c) = self.zx_to_ascii.get(&base) {
                return format!("%{}", c);
            }
        }
        if let Some(s) = self.graphics.get(&zx_char) {
            return s.to_string();
        }
        // if let Some(c) = self.zx_token_chars.get(&zx_char) {
        //     return c.to_string();
        // }
        if let Some(c) = self.zx_to_ascii.get(&zx_char) {
            return c.to_string();
        }
        // format!("[UNK:{}]", zx_char)
        "?".to_string()
    }

    /// Checks if a token supports a line number argument.
    fn token_supports_line_number(&self, token_code: u8) -> bool {
        matches!(token_code, 236 | 251 | 247 | 230) // GOTO, GOSUB, RUN, LIST
    }

    /// Decodes a single line of ZX80 BASIC.
    pub fn decode_line(&self, line_bytes: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        let mut in_rem_comment = false;

        while i < line_bytes.len() {
            let byte = line_bytes[i];

            if in_rem_comment {
                result.push_str(&self.decode_character(byte));
            } else if let Some(token) = self.tokens.get(&byte) {
                result.push_str(token);
                if byte == 254 { // REM token
                    in_rem_comment = true;
                } else if self.token_supports_line_number(byte) {
                    let mut j = i + 1;
                    let mut line_number_str = String::new();
                    while j < line_bytes.len() {
                        if let Some(ch) = self.zx_to_ascii.get(&line_bytes[j]) {
                            if ch.is_ascii_digit() {
                                line_number_str.push(*ch);
                                j += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if !line_number_str.is_empty() {
                        result.push_str(&line_number_str);
                        i = j - 1;
                    }
                }
            } else {
                result.push_str(&self.decode_character(byte));
            }
            i += 1;
        }
        result
    }
}

/// Decodes a byte array representing a ZX80 program into a String.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes containing the ZX80 program.
/// * `zxpand_enabled` - A boolean to enable/disable ZXPAND specific tokens.
///
/// # Returns
///
/// A `Result` containing the decoded program as a `String`, or an error message.
pub fn decode_zx80_bytes(bytes: &[u8], zxpand_enabled: bool) -> Result<String, &'static str> {
    if bytes.len() < 40 {
        return Err("Input byte array is too short to be a valid ZX80 file.");
    }
    
    let decoder = ZX80BasicDecoder::new(zxpand_enabled);
    let mut result = String::new();
    let mut current_pos = 40; // Skip the 40-byte header

    let end_of_program = u16::from_le_bytes([bytes[8], bytes[9]]) as usize - 0x4000;

    while current_pos < end_of_program && current_pos + 2 < bytes.len() {
        let high = bytes[current_pos];
        if high == 0x80 {
            break; // End of program marker
        }
        let low = bytes[current_pos + 1];
        let line_number = u16::from_be_bytes([high, low]);
        current_pos += 2;

        let mut line_bytes = Vec::new();
        while current_pos < bytes.len() {
            let byte = bytes[current_pos];
            current_pos += 1;
            if byte == 0x76 { // End of line marker
                break;
            }
            line_bytes.push(byte);
        }
        
        let line_text = decoder.decode_line(&line_bytes);
        result.push_str(&format!("{} {}\n", line_number, line_text));
    }

    Ok(result)
}