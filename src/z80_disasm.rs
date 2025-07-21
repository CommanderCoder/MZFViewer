use std::char;

pub struct Z80Disassembler {
    pc: u16,
}

impl Z80Disassembler {
    pub fn new() -> Self {
        Self { pc: 0 }
    }

    pub fn disassemble(&mut self, data: &[u8], start_address: u16, exec_address: u16) -> Vec<String> {
        let mut result = Vec::new();
        let mut pos = 0;
        self.pc = start_address;

        while pos < data.len() {
            let (instruction, bytes_consumed) = self.decode_instruction(&data[pos..]);
            
            let end_pos = (pos + bytes_consumed).min(data.len());
            let instruction_bytes = &data[pos..end_pos];

            let hex_bytes = instruction_bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            
            let ascii = instruction_bytes
                .iter()
                .map(|&b| b as char)
                .collect::<String>();

            // MODIFICATION: The formatting logic below was changed to match the target ASM file.
            let marker = if exec_address == self.pc { ">" } else { " " };
            let marker_and_hex = format!("{} {}", marker, hex_bytes);
            
            // Format the final line with specific padding to align columns.
            // - Instruction field is padded to 22 characters.
            // - Hex bytes field is padded to 15 characters.

            let line = format!(
                "{:<22} ;{:04X} {:<15} {}",
                instruction,
                self.pc,
                marker_and_hex,
                ascii
            );

            result.push(line);
            
            pos += bytes_consumed;
            self.pc = self.pc.wrapping_add(bytes_consumed as u16);
        }

        result
    }

    // Helper functions to read operands
    fn read_u8(data: &[u8]) -> Option<u8> { data.get(0).copied() }
    fn read_i8(data: &[u8]) -> Option<i8> { data.get(0).map(|&b| b as i8) }
    fn read_u16(data: &[u8]) -> Option<u16> {
        if data.len() < 2 {
            None
        } else {
            Some(u16::from_le_bytes([data[0], data[1]]))
        }
    }

    fn decode_instruction(&self, data: &[u8]) -> (String, usize) {
        if data.is_empty() {
            return ("???".to_string(), 1);
        }
        let opcode = data[0];
        
        match opcode {
            // 8-bit immediate loads
            0x06 => self.decode_ld_r_n(data, "B"),
            0x0E => self.decode_ld_r_n(data, "C"),
            0x16 => self.decode_ld_r_n(data, "D"),
            0x1E => self.decode_ld_r_n(data, "E"),
            0x26 => self.decode_ld_r_n(data, "H"),
            0x2E => self.decode_ld_r_n(data, "L"),
            0x36 => self.decode_ld_hl_n(data),
            0x3E => self.decode_ld_r_n(data, "A"),

            // 8-bit loads to/from memory
            0x02 => ("LD (BC),A".to_string(), 1),
            0x0A => ("LD A,(BC)".to_string(), 1),
            0x12 => ("LD (DE),A".to_string(), 1),
            0x1A => ("LD A,(DE)".to_string(), 1),
            0x22 => self.decode_ld_to_nn_from_hl(data),
            0x2A => self.decode_ld_hl_from_nn(data),
            0x32 => self.decode_ld_to_nn_from_a(data),
            0x3A => self.decode_ld_a_from_nn(data),
            
            // 8-bit register-to-register loads
            0x40..=0x7F => {
                if opcode == 0x76 {
                    ("HALT".to_string(), 1)
                } else {
                    let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
                    let dest_reg = regs[((opcode & 0x38) >> 3) as usize];
                    let src_reg = regs[(opcode & 0x07) as usize];
                    (format!("LD {},{}", dest_reg, src_reg), 1)
                }
            },
            
            // Arithmetic group
            0x80..=0xBF => {
                let ops = ["ADD A,", "ADC A,", "SUB ", "SBC A,", "AND ", "XOR ", "OR ", "CP "];
                let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
                let op = ops[((opcode & 0x38) >> 3) as usize];
                let reg = regs[(opcode & 0x07) as usize];
                (format!("{}{}", op, reg), 1)
            },
            
            // Immediate arithmetic
            0xC6 => self.decode_add_a_n(data),
            0xCE => self.decode_adc_a_n(data),
            0xD6 => self.decode_sub_n(data),
            0xDE => self.decode_sbc_a_n(data),
            0xE6 => self.decode_and_n(data),
            0xEE => self.decode_xor_n(data),
            0xF6 => self.decode_or_n(data),
            0xFE => self.decode_cp_n(data),
            
            // 16-bit loads
            0x01 => self.decode_ld_dd_nn(data, "BC"),
            0x11 => self.decode_ld_dd_nn(data, "DE"),
            0x21 => self.decode_ld_dd_nn(data, "HL"),
            0x31 => self.decode_ld_dd_nn(data, "SP"),
            0xF9 => ("LD SP,HL".to_string(), 1),
            
            // 16-bit arithmetic
            0x09 => ("ADD HL,BC".to_string(), 1),
            0x19 => ("ADD HL,DE".to_string(), 1),
            0x29 => ("ADD HL,HL".to_string(), 1),
            0x39 => ("ADD HL,SP".to_string(), 1),

            // Jumps, Calls, and Returns
            0x10 => self.decode_djnz_e(data),
            0x18 => self.decode_jr_e(data),
            0x20 => self.decode_jr_cc_e(data, "NZ"),
            0x28 => self.decode_jr_cc_e(data, "Z"),
            0x30 => self.decode_jr_cc_e(data, "NC"),
            0x38 => self.decode_jr_cc_e(data, "C"),
            
            0xC3 => self.decode_jp_nn(data),
            0xC2 => self.decode_jp_cc_nn(data, "NZ"),
            0xCA => self.decode_jp_cc_nn(data, "Z"),
            0xD2 => self.decode_jp_cc_nn(data, "NC"),
            0xDA => self.decode_jp_cc_nn(data, "C"),
            0xE2 => self.decode_jp_cc_nn(data, "PO"),
            0xEA => self.decode_jp_cc_nn(data, "PE"),
            0xF2 => self.decode_jp_cc_nn(data, "P"),
            0xFA => self.decode_jp_cc_nn(data, "M"),
            0xE9 => ("JP (HL)".to_string(), 1),
            
            0xCD => self.decode_call_nn(data),
            0xC4 => self.decode_call_cc_nn(data, "NZ"),
            0xCC => self.decode_call_cc_nn(data, "Z"),
            0xD4 => self.decode_call_cc_nn(data, "NC"),
            0xDC => self.decode_call_cc_nn(data, "C"),
            0xE4 => self.decode_call_cc_nn(data, "PO"),
            0xEC => self.decode_call_cc_nn(data, "PE"),
            0xF4 => self.decode_call_cc_nn(data, "P"),
            0xFC => self.decode_call_cc_nn(data, "M"),

            0xC9 => ("RET".to_string(), 1),
            0xC0 => ("RET NZ".to_string(), 1),
            0xC8 => ("RET Z".to_string(), 1),
            0xD0 => ("RET NC".to_string(), 1),
            0xD8 => ("RET C".to_string(), 1),
            0xE0 => ("RET PO".to_string(), 1),
            0xE8 => ("RET PE".to_string(), 1),
            0xF0 => ("RET P".to_string(), 1),
            0xF8 => ("RET M".to_string(), 1),
            
            // Stack operations
            0xC5 => ("PUSH BC".to_string(), 1),
            0xD5 => ("PUSH DE".to_string(), 1),
            0xE5 => ("PUSH HL".to_string(), 1),
            0xF5 => ("PUSH AF".to_string(), 1),
            0xC1 => ("POP BC".to_string(), 1),
            0xD1 => ("POP DE".to_string(), 1),
            0xE1 => ("POP HL".to_string(), 1),
            0xF1 => ("POP AF".to_string(), 1),
            
            // Increment/Decrement
            0x03 => ("INC BC".to_string(), 1), 0x0B => ("DEC BC".to_string(), 1),
            0x13 => ("INC DE".to_string(), 1), 0x1B => ("DEC DE".to_string(), 1),
            0x23 => ("INC HL".to_string(), 1), 0x2B => ("DEC HL".to_string(), 1),
            0x33 => ("INC SP".to_string(), 1), 0x3B => ("DEC SP".to_string(), 1),
            
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
                (format!("INC {}", regs[((opcode & 0x38) >> 3) as usize]), 1)
            },
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
                (format!("DEC {}", regs[((opcode & 0x38) >> 3) as usize]), 1)
            },
            
            // Misc
            0x00 => ("NOP".to_string(), 1),
            0x07 => ("RLCA".to_string(), 1),
            0x0F => ("RRCA".to_string(), 1),
            0x17 => ("RLA".to_string(), 1),
            0x1F => ("RRA".to_string(), 1),
            0x27 => ("DAA".to_string(), 1),
            0x2F => ("CPL".to_string(), 1),
            0x37 => ("SCF".to_string(), 1),
            0x3F => ("CCF".to_string(), 1),
            0xF3 => ("DI".to_string(), 1),
            0xFB => ("EI".to_string(), 1),
            0x08 => ("EX AF,AF'".to_string(), 1),
            0xEB => ("EX DE,HL".to_string(), 1),
            0xD9 => ("EXX".to_string(), 1),
            0xE3 => ("EX (SP),HL".to_string(), 1),

            // RST
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                (format!("RST {:02X}H", opcode & 0x38), 1)
            },

            // I/O
            0xDB => self.decode_in_a_n(data),
            0xD3 => self.decode_out_n_a(data),

            // Prefixed Instructions
            0xCB => self.decode_cb_prefixed(data),
            0xED => self.decode_ed_prefixed(data),
            0xDD => {
                let (mut instruction, mut bytes) = self.decode_ixy_prefixed(&data[1..], "IX");
                if data.len() > 2 && data[1] == 0xCB {
                    // Handle DDCB prefix
                    let (inst, b) = self.decode_ixycb_prefixed(&data[2..], "IX");
                    instruction = inst;
                    bytes = b + 1; // +1 for the CB
                }
                (instruction, bytes + 1)
            },
            0xFD => {
                let (mut instruction, mut bytes) = self.decode_ixy_prefixed(&data[1..], "IY");
                 if data.len() > 2 && data[1] == 0xCB {
                    // Handle FDCB prefix
                    let (inst, b) = self.decode_ixycb_prefixed(&data[2..], "IY");
                    instruction = inst;
                    bytes = b + 1; // +1 for the CB
                }
                (instruction, bytes + 1)
            },

            // _ => (format!("DB ${:02X}", opcode), 1),
        }
    }
    
    // --- Decoding Helper Functions ---

    fn decode_ld_r_n(&self, data: &[u8], reg: &str) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("LD {},{:02X}H", reg, n), 2)
        )
    }
    
    fn decode_ld_hl_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("LD (HL),{:02X}H", n), 2)
        )
    }

    fn decode_ld_to_nn_from_hl(&self, data: &[u8]) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("LD ({:04X}H),HL", nn), 3)
        )
    }

    fn decode_ld_hl_from_nn(&self, data: &[u8]) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("LD HL,({:04X}H)", nn), 3)
        )
    }

    fn decode_ld_to_nn_from_a(&self, data: &[u8]) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("LD ({:04X}H),A", nn), 3)
        )
    }
    
    fn decode_ld_a_from_nn(&self, data: &[u8]) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("LD A,({:04X}H)", nn), 3)
        )
    }

    fn decode_ld_dd_nn(&self, data: &[u8], reg: &str) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("LD {},{:04X}H", reg, nn), 3)
        )
    }

    fn decode_add_a_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("ADD A,{:02X}H", n), 2)
        )
    }
    
    fn decode_adc_a_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("ADC A,{:02X}H", n), 2)
        )
    }
    
    fn decode_sub_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("SUB {:02X}H", n), 2)
        )
    }
    
    fn decode_sbc_a_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("SBC A,{:02X}H", n), 2)
        )
    }
    
    fn decode_and_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("AND {:02X}H", n), 2)
        )
    }
    
    fn decode_xor_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("XOR {:02X}H", n), 2)
        )
    }
    
    fn decode_or_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("OR {:02X}H", n), 2)
        )
    }
    
    fn decode_cp_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("CP {:02X}H", n), 2)
        )
    }
    
    fn decode_jr_e(&self, data: &[u8]) -> (String, usize) {
        Self::read_i8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |e| {
                let target = self.pc.wrapping_add(2).wrapping_add(e as u16);
                (format!("JR {:04X}H", target), 2)
            }
        )
    }

    fn decode_djnz_e(&self, data: &[u8]) -> (String, usize) {
         Self::read_i8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |e| {
                let target = self.pc.wrapping_add(2).wrapping_add(e as u16);
                (format!("DJNZ {:04X}H", target), 2)
            }
        )
    }
    
    fn decode_jr_cc_e(&self, data: &[u8], condition: &str) -> (String, usize) {
        Self::read_i8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |e| {
                let target = self.pc.wrapping_add(2).wrapping_add(e as u16);
                (format!("JR {},{:04X}H", condition, target), 2)
            }
        )
    }
    
    fn decode_jp_nn(&self, data: &[u8]) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("JP {:04X}H", nn), 3)
        )
    }
    
    fn decode_jp_cc_nn(&self, data: &[u8], condition: &str) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("JP {},{:04X}H", condition, nn), 3)
        )
    }
    
    fn decode_call_nn(&self, data: &[u8]) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("CALL {:04X}H", nn), 3)
        )
    }
    
    fn decode_call_cc_nn(&self, data: &[u8], condition: &str) -> (String, usize) {
        Self::read_u16(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |nn| (format!("CALL {},{:04X}H", condition, nn), 3)
        )
    }

    fn decode_in_a_n(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("IN A,({:02X}H)", n), 2)
        )
    }

    fn decode_out_n_a(&self, data: &[u8]) -> (String, usize) {
        Self::read_u8(&data[1..]).map_or_else(
            || ("???".to_string(), 1),
            |n| (format!("OUT ({:02X}H),A", n), 2)
        )
    }
    
    fn decode_cb_prefixed(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 { return ("???".to_string(), 1); }
        let opcode = data[1];
        let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
        let reg = regs[opcode as usize & 0x07];
        
        match opcode {
            0x00..=0x07 => (format!("RLC {}", reg), 2),
            0x08..=0x0F => (format!("RRC {}", reg), 2),
            0x10..=0x17 => (format!("RL {}", reg), 2),
            0x18..=0x1F => (format!("RR {}", reg), 2),
            0x20..=0x27 => (format!("SLA {}", reg), 2),
            0x28..=0x2F => (format!("SRA {}", reg), 2),
            0x30..=0x37 => (format!("SLL {}", reg), 2), // Undocumented
            0x38..=0x3F => (format!("SRL {}", reg), 2),
            0x40..=0x7F => (format!("BIT {},{}", (opcode - 0x40) / 8, reg), 2),
            0x80..=0xBF => (format!("RES {},{}", (opcode - 0x80) / 8, reg), 2),
            0xC0..=0xFF => (format!("SET {},{}", (opcode - 0xC0) / 8, reg), 2),
        }
    }
    
    fn decode_ed_prefixed(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 { return ("???".to_string(), 1); }
        let opcode = data[1];
        
        let regs16_sp = ["BC", "DE", "HL", "SP"];
        
        match opcode {
            // Block instructions
            0xA0 => ("LDI".to_string(), 2), 0xA1 => ("CPI".to_string(), 2),
            0xA2 => ("INI".to_string(), 2), 0xA3 => ("OUTI".to_string(), 2),
            0xA8 => ("LDD".to_string(), 2), 0xA9 => ("CPD".to_string(), 2),
            0xAA => ("IND".to_string(), 2), 0xAB => ("OUTD".to_string(), 2),
            0xB0 => ("LDIR".to_string(), 2), 0xB1 => ("CPIR".to_string(), 2),
            0xB2 => ("INIR".to_string(), 2), 0xB3 => ("OTIR".to_string(), 2),
            0xB8 => ("LDDR".to_string(), 2), 0xB9 => ("CPDR".to_string(), 2),
            0xBA => ("INDR".to_string(), 2), 0xBB => ("OTDR".to_string(), 2),

            // Interrupts
            0x44 => ("NEG".to_string(), 2), // Or RETN on some Z80s
            0x45 => ("RETN".to_string(), 2),
            0x4D => ("RETI".to_string(), 2),
            0x46 => ("IM 0".to_string(), 2),
            0x56 => ("IM 1".to_string(), 2),
            0x5E => ("IM 2".to_string(), 2),
            
            // Register loads
            0x47 => ("LD I,A".to_string(), 2), 0x57 => ("LD A,I".to_string(), 2),
            0x4F => ("LD R,A".to_string(), 2), 0x5F => ("LD A,R".to_string(), 2),

            // 16-bit ADC/SBC
            0x4A | 0x5A | 0x6A | 0x7A => (format!("ADC HL,{}", regs16_sp[((opcode - 0x4A) / 16) as usize]), 2),
            0x42 | 0x52 | 0x62 | 0x72 => (format!("SBC HL,{}", regs16_sp[((opcode - 0x42) / 16) as usize]), 2),
            
            // 16-bit loads from/to memory
            0x43 | 0x53 | 0x63 | 0x73 => {
                Self::read_u16(&data[2..]).map_or_else(
                    || ("???".to_string(), 2),
                    |nn| (format!("LD ({:04X}H),{}", nn, regs16_sp[((opcode - 0x43) / 16) as usize]), 4)
                )
            },
            0x4B | 0x5B | 0x6B | 0x7B => {
                Self::read_u16(&data[2..]).map_or_else(
                    || ("???".to_string(), 2),
                    |nn| (format!("LD {},({:04X}H)", regs16_sp[((opcode - 0x4B) / 16) as usize], nn), 4)
                )
            },

            // Rotates
            0x67 => ("RRD".to_string(), 2),
            0x6F => ("RLD".to_string(), 2),
            
            // I/O
            0x40..=0x79 => { // IN/OUT (C) group, excluding handled cases
                let regs = ["B", "C", "D", "E", "H", "L", "F", "A"]; // F is for undocumented IN F,(C)
                let reg = regs[((opcode & 0x38) >> 3) as usize];
                if opcode & 0x01 == 0 { (format!("IN {},(C)", reg), 2) }
                else { (format!("OUT (C),{}", reg), 2) }
            },
            
            _ => (format!("DB ${:02X},${:02X}", 0xED, opcode), 2),
        }
    }

    fn decode_ixy_prefixed(&self, data: &[u8], reg: &str) -> (String, usize) {
        if data.is_empty() { return ("???".to_string(), 1); }
        let opcode = data[0];
        let regs8 = ["B", "C", "D", "E", "H", "L", "", "A"];
        // let regs16 = ["BC", "DE", reg, "SP"];

        match opcode {
            // Overridden main opcodes
            0x21 => Self::read_u16(&data[1..]).map_or(
                ("???".to_string(), 1), |nn| (format!("LD {},{:04X}H", reg, nn), 3)),
            0x22 => Self::read_u16(&data[1..]).map_or(
                ("???".to_string(), 1), |nn| (format!("LD ({:04X}H),{}", nn, reg), 3)),
            0x2A => Self::read_u16(&data[1..]).map_or(
                ("???".to_string(), 1), |nn| (format!("LD {},({:04X}H)", reg, nn), 3)),
            0x23 => (format!("INC {}", reg), 1),
            0x2B => (format!("DEC {}", reg), 1),
            0x29 => (format!("ADD {},{}", reg, reg), 1),
            0x09 => (format!("ADD {},BC", reg), 1),
            0x19 => (format!("ADD {},DE", reg), 1),
            0x39 => (format!("ADD {},SP", reg), 1),
            0xE1 => (format!("POP {}", reg), 1),
            0xE3 => (format!("EX (SP),{}", reg), 1),
            0xE5 => (format!("PUSH {}", reg), 1),
            0xE9 => (format!("JP ({})", reg), 1),
            0xF9 => (format!("LD SP,{}", reg), 1),
            
            // LD r,(IX/Y+d)
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E => {
                Self::read_i8(&data[1..]).map_or(
                    ("???".to_string(), 1),
                    |d| (format!("LD {},({}{:+#}H)", regs8[((opcode & 0x38) >> 3) as usize], reg, d), 2)
                )
            },
            // LD (IX/Y+d),r
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                Self::read_i8(&data[1..]).map_or(
                    ("???".to_string(), 1),
                    |d| (format!("LD ({}{:+#}H),{}", reg, d, regs8[(opcode & 0x07) as usize]), 2)
                )
            },
            // LD (IX/Y+d),n
            0x36 => {
                if data.len() < 3 { return ("???".to_string(), 1); }
                let d = data[1] as i8;
                let n = data[2];
                (format!("LD ({}{:+#}H),{:02X}H", reg, d, n), 3)
            },

            // Arithmetic (IX/Y+d)
            0x86 => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("ADD A,({}{:+#}H)", reg, d), 2)),
            0x8E => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("ADC A,({}{:+#}H)", reg, d), 2)),
            0x96 => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("SUB ({}{:+#}H)", reg, d), 2)),
            0x9E => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("SBC A,({}{:+#}H)", reg, d), 2)),
            0xA6 => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("AND ({}{:+#}H)", reg, d), 2)),
            0xAE => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("XOR ({}{:+#}H)", reg, d), 2)),
            0xB6 => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("OR ({}{:+#}H)", reg, d), 2)),
            0xBE => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("CP ({}{:+#}H)", reg, d), 2)),
            
            // INC/DEC (IX/Y+d)
            0x34 => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("INC ({}{:+#}H)", reg, d), 2)),
            0x35 => Self::read_i8(&data[1..]).map_or(("???".to_string(), 1), |d| (format!("DEC ({}{:+#}H)", reg, d), 2)),

            // CB is handled separately in the main dispatcher
            0xCB => ( "???".to_string(), 1 ), // Should not be reached
            _ => (format!("DB ${:02X}", opcode), 1)
        }
    }

    fn decode_ixycb_prefixed(&self, data: &[u8], reg: &str) -> (String, usize) {
        if data.len() < 2 { return ("???".to_string(), 2); }
        let d = data[0] as i8;
        let opcode = data[1];
        let operand = format!("({}{:+#}H)", reg, d);
        let regs = ["B", "C", "D", "E", "H", "L", "", "A"];
        let dest_reg = regs[opcode as usize & 0x07];

        let op_str = match opcode {
            0x00..=0x07 => format!("RLC {},{}", dest_reg, operand), // Undocumented
            0x08..=0x0F => format!("RRC {},{}", dest_reg, operand), // Undocumented
            0x10..=0x17 => format!("RL {},{}", dest_reg, operand), // Undocumented
            0x18..=0x1F => format!("RR {},{}", dest_reg, operand), // Undocumented
            0x20..=0x27 => format!("SLA {},{}", dest_reg, operand), // Undocumented
            0x28..=0x2F => format!("SRA {},{}", dest_reg, operand), // Undocumented
            0x30..=0x37 => format!("SLL {},{}", dest_reg, operand), // Undocumented
            0x38..=0x3F => format!("SRL {},{}", dest_reg, operand), // Undocumented
            0x40..=0x7F => format!("BIT {},{}", (opcode - 0x40) / 8, operand),
            0x80..=0xBF => format!("RES {},{}", (opcode - 0x80) / 8, operand),
            0xC0..=0xFF => format!("SET {},{}", (opcode - 0xC0) / 8, operand),
        };
        (op_str, 2)
    }
}