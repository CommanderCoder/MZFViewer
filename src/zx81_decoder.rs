// Adapted from 1993 codebase via
//  https://github.com/ryangray/zx81-utils

/// Output styles for ZX81 BASIC decoding
#[derive(Debug, Clone, Copy)]
pub enum OutputStyle {
    Readable,
}

/// A decoder for ZX81 BASIC programs.
pub struct ZX81BasicDecoder {
    charset: Vec<&'static str>
}

impl ZX81BasicDecoder {
    /// Creates a new `ZX81BasicDecoder` with the specified output style.
    pub fn new(style: OutputStyle) -> Self {
        let charset = match style {
            OutputStyle::Readable => Self::charset_readable(),
        };
        
        ZX81BasicDecoder {
            charset,
        }
    }

    /// Character mapping for readable output (default)
    fn charset_readable() -> Vec<&'static str> {
        vec![
            // 000-009
            " ", "▘", "▝", "▀", "▖", "▌", "▞", "▛", "▒", "&#x1fb8f;",
            // 010-019
            "&#x1fb8e;", "\"", "£", "$", ":", "?", "(", ")", ">", "<",
            // 020-029
            "=", "+", "-", "*", "/", ";", ",", ".", "0", "1",
            // 030-039
            "2", "3", "4", "5", "6", "7", "8", "9", "A", "B",
            // 040-049
            "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
            // 050-059
            "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V",
            // 060-069
            "W", "X", "Y", "Z", "RND", "INKEY$ ", "PI", "#", "#", "#",
            // 070-079
            "#", "#", "#", "#", "#", "#", "#", "#", "#", "#",
            // 080-089
            "#", "#", "#", "#", "#", "#", "#", "#", "#", "#",
            // 090-099
            "#", "#", "#", "#", "#", "#", "#", "#", "#", "#",
            // 100-109
            "#", "#", "#", "#", "#", "#", "#", "#", "#", "#",
            // 110-119
            "#", "#", "#", "#", "#", "#", "#", "#", "#", "#",
            // 120-129
            "#", "#", "#", "#", "#", "#", "#", "#", "§ §", "▟",
            // 130-139
            "▙", "▄", "▜", "▐", "▚", "▗", "▒", "§&#x1fb8f;§", "§&#x1fb8e;§", "§\"§",
            // 140-149
            "§£§", "§$§", "§:§", "§?§", "§(§", "§)§", "§>§", "§<§", "§=§", "§+§",
            // 150-159
            "§-§", "§*§", "§/§", "§;§", "§,§", "§.§", "§0§", "§1§", "§2§", "§3§",
            // 160-169
            "§4§", "§5§", "§6§", "§7§", "§8§", "§9§", "§A§", "§B§", "§C§", "§D§",
            // 170-179
            "§E§", "§F§", "§G§", "§H§", "§I§", "§J§", "§K§", "§L§", "§M§", "§N§",
            // 180-189
            "§O§", "§P§", "§Q§", "§R§", "§S§", "§T§", "§U§", "§V§", "§W§", "§X§",
            // 190-199
            "§Y§", "§Z§", "\"\"", "AT ", "TAB ", "#", "CODE ", "VAL ", "LEN ", "SIN ",
            // 200-209
            "COS ", "TAN ", "ASN ", "ACS ", "ATN ", "LN ", "EXP ", "INT ", "SQR ", "SGN ",
            // 210-219
            "ABS ", "PEEK ", "USR ", "STR$ ", "CHR$ ", "NOT ", "**", " OR ", " AND ", "<=",
            // 220-229
            ">=", "<>", " THEN", " TO ", " STEP ", " LPRINT ", " LLIST ", " STOP", " SLOW", " FAST",
            // 230-239
            " NEW", " SCROLL", " CONT ", " DIM ", " REM ", " FOR ", " GOTO ", " GOSUB ", " INPUT ", " LOAD ",
            // 240-249
            " LIST ", " LET ", " PAUSE ", " NEXT ", " POKE ", " PRINT ", " PLOT ", " RUN ", " SAVE ", " RAND ",
            // 250-255
            " IF ", " CLS", " UNPLOT ", " CLEAR", " RETURN", " COPY"
        ]
    }

    /// Translates a ZX81 program line into readable text
    fn translate_line(&self, line_bytes: &[u8]) -> String {
        let mut result = String::new();
        let mut in_quotes = false;
        let line_len = line_bytes.len();
        
        if line_len == 0 {
            return result;
        }

        let keyword = line_bytes[0];
        let mut i = 0;

        while i < line_len - 1 {
            let c = usize::from(line_bytes[i]);
            let x = if c < self.charset.len() {
                self.charset[c]
            } else {
                "#"
            };

            // Handle quotes (toggle in_quotes state)
            if keyword != 234 && c == 11 { // Not REM and is QUOTE
                in_quotes = !in_quotes;
            }

            // Skip inline floating point numbers (except in REM)
            if keyword != 234 && c == 126 { // Not REM and is NUM_code
                i += 5; // Skip the 5-byte floating point number
                continue;
            }

            result.push_str(x);

            i += 1;
        }

        result
    }
}

/// Reads a ZX81 program line from the byte stream
fn read_zx81_line(bytes: &[u8], pos: &mut usize, remaining: &mut i32) -> Option<(u16, Vec<u8>)> {
    if *remaining < 4 {
        return None;
    }

    // Read line number (big-endian)
    let line_num = ((bytes[*pos] as u16) << 8) | (bytes[*pos + 1] as u16);
    *pos += 2;
    *remaining -= 2;

    // Read line length (little-endian)
    let line_len = (bytes[*pos] as u16) | ((bytes[*pos + 1] as u16) << 8);
    *pos += 2;
    *remaining -= 2;

    if *remaining < line_len as i32 {
        return None;
    }

    // Read the line content
    let line_bytes = bytes[*pos..*pos + line_len as usize].to_vec();
    *pos += line_len as usize;
    *remaining -= line_len as i32;

    Some((line_num, line_bytes))
}

/// Decodes a ZX81 .P file into readable BASIC text
///
/// # Arguments
///
/// * `bytes` - The complete .P file as a byte array
/// * `style` - The output style to use
///
/// # Returns
///
/// A `Result` containing the decoded program as a `String`, or an error message.
pub fn decode_zx81_p_file(bytes: &[u8], style: OutputStyle) -> Result<String, &'static str> {
    if bytes.len() < 116 {
        return Err("Input byte array is too short to be a valid ZX81 .P file.");
    }

    let decoder = ZX81BasicDecoder::new(style);
    let mut result = String::new();
    
    // Skip first 3 bytes of system variables
    let mut pos = 3;
    
    // Read d_file (address of display file)
    let d_file = (bytes[pos] as u16) | ((bytes[pos + 1] as u16) << 8);
    pos += 2;
    
    // Skip remaining system variables (111 bytes)
    pos += 111;
    
    // Calculate total program size
    let mut total = (d_file as i32) - 16509;
    
    // Process lines
    while total >= 0 {
        if let Some((line_num, line_bytes)) = read_zx81_line(bytes, &mut pos, &mut total) {
            let line_text = decoder.translate_line(&line_bytes);
            result.push_str(&format!("{:4} {}\n", line_num, line_text));
        } else {
            break;
        }
    }
    
    if result.is_empty() {
        return Err("No valid BASIC program found in the file.");
    }
    
    Ok(result)
}

/// Convenience function for decoding with readable output (default)
pub fn decode_zx81_bytes(bytes: &[u8]) -> Result<String, &'static str> {
    decode_zx81_p_file(bytes, OutputStyle::Readable)
}
