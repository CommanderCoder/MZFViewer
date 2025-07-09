use std::char;

use crate::MZDetokenizer;

pub struct Z80Disassembler {
    pc: u16,
    d: MZDetokenizer,
}

impl Z80Disassembler {
    pub fn new(detokenizer: MZDetokenizer) -> Self {
        Self { pc: 0, d: detokenizer }
    }

    pub fn disassemble(&mut self, data: &[u8], start_address: u16, exec_address: u16, charset_flag: bool) -> Vec<String> {
        let mut result = Vec::new();
        let mut pos = 0;
        self.pc = start_address;

        while pos < data.len() {
            let (instruction, bytes_consumed) = self.decode_instruction(&data[pos..]);
            
            // Format: ADDRESS: BYTES    INSTRUCTION  ASCII
            let hex_bytes = data[pos..pos + bytes_consumed]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            
            let ascii = data[pos..pos + bytes_consumed]
                .iter()
                .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else if charset_flag {
                        if let Some(&ch) = self.d.sharp_ascii.get(&b) {
                            ch
                        } else {
                            '.'
                        }
                    } else {
                        '.'
                    })
                .collect::<String>();

            let line = format!("{:04X}: {} {:<12} {:<20} ; {}", self.pc, if exec_address == self.pc { ">" } else { " " }, hex_bytes, instruction, ascii);
            result.push(line);
            
            pos += bytes_consumed;
            self.pc = self.pc.wrapping_add(bytes_consumed as u16);
        }

        result
    }

    fn decode_instruction(&self, data: &[u8]) -> (String, usize) {
        if data.is_empty() {
            return ("???".to_string(), 1);
        }

        let opcode = data[0];
        
        match opcode {

            // 8-bit immediate loads
            0x02 => ("LD (BC),A".to_string(), 1),
            0x12 => ("LD (DE),A".to_string(), 1),
            0x22 => self.decode_ld_nn_hl(data),
            0x32 => self.decode_ld_nn_a(data),

            // 8-bit load group
            0x06 => self.decode_ld_r_n(data, "B"),
            0x0E => self.decode_ld_r_n(data, "C"),
            0x16 => self.decode_ld_r_n(data, "D"),
            0x1E => self.decode_ld_r_n(data, "E"),
            0x26 => self.decode_ld_r_n(data, "H"),
            0x2E => self.decode_ld_r_n(data, "L"),
            0x36 => self.decode_ld_hl_n(data),
            0x3E => self.decode_ld_r_n(data, "A"),
            
            // 8-bit register to register loads
            0x40 => ("LD B,B".to_string(), 1),
            0x41 => ("LD B,C".to_string(), 1),
            0x42 => ("LD B,D".to_string(), 1),
            0x43 => ("LD B,E".to_string(), 1),
            0x44 => ("LD B,H".to_string(), 1),
            0x45 => ("LD B,L".to_string(), 1),
            0x46 => ("LD B,(HL)".to_string(), 1),
            0x47 => ("LD B,A".to_string(), 1),
            
            0x48 => ("LD C,B".to_string(), 1),
            0x49 => ("LD C,C".to_string(), 1),
            0x4A => ("LD C,D".to_string(), 1),
            0x4B => ("LD C,E".to_string(), 1),
            0x4C => ("LD C,H".to_string(), 1),
            0x4D => ("LD C,L".to_string(), 1),
            0x4E => ("LD C,(HL)".to_string(), 1),
            0x4F => ("LD C,A".to_string(), 1),
            
            0x50 => ("LD D,B".to_string(), 1),
            0x51 => ("LD D,C".to_string(), 1),
            0x52 => ("LD D,D".to_string(), 1),
            0x53 => ("LD D,E".to_string(), 1),
            0x54 => ("LD D,H".to_string(), 1),
            0x55 => ("LD D,L".to_string(), 1),
            0x56 => ("LD D,(HL)".to_string(), 1),
            0x57 => ("LD D,A".to_string(), 1),
            
            0x58 => ("LD E,B".to_string(), 1),
            0x59 => ("LD E,C".to_string(), 1),
            0x5A => ("LD E,D".to_string(), 1),
            0x5B => ("LD E,E".to_string(), 1),
            0x5C => ("LD E,H".to_string(), 1),
            0x5D => ("LD E,L".to_string(), 1),
            0x5E => ("LD E,(HL)".to_string(), 1),
            0x5F => ("LD E,A".to_string(), 1),
            
            0x60 => ("LD H,B".to_string(), 1),
            0x61 => ("LD H,C".to_string(), 1),
            0x62 => ("LD H,D".to_string(), 1),
            0x63 => ("LD H,E".to_string(), 1),
            0x64 => ("LD H,H".to_string(), 1),
            0x65 => ("LD H,L".to_string(), 1),
            0x66 => ("LD H,(HL)".to_string(), 1),
            0x67 => ("LD H,A".to_string(), 1),
            
            0x68 => ("LD L,B".to_string(), 1),
            0x69 => ("LD L,C".to_string(), 1),
            0x6A => ("LD L,D".to_string(), 1),
            0x6B => ("LD L,E".to_string(), 1),
            0x6C => ("LD L,H".to_string(), 1),
            0x6D => ("LD L,L".to_string(), 1),
            0x6E => ("LD L,(HL)".to_string(), 1),
            0x6F => ("LD L,A".to_string(), 1),
            
            0x70 => ("LD (HL),B".to_string(), 1),
            0x71 => ("LD (HL),C".to_string(), 1),
            0x72 => ("LD (HL),D".to_string(), 1),
            0x73 => ("LD (HL),E".to_string(), 1),
            0x74 => ("LD (HL),H".to_string(), 1),
            0x75 => ("LD (HL),L".to_string(), 1),
            0x76 => ("HALT".to_string(), 1),
            0x77 => ("LD (HL),A".to_string(), 1),
            
            0x78 => ("LD A,B".to_string(), 1),
            0x79 => ("LD A,C".to_string(), 1),
            0x7A => ("LD A,D".to_string(), 1),
            0x7B => ("LD A,E".to_string(), 1),
            0x7C => ("LD A,H".to_string(), 1),
            0x7D => ("LD A,L".to_string(), 1),
            0x7E => ("LD A,(HL)".to_string(), 1),
            0x7F => ("LD A,A".to_string(), 1),
            
            // Arithmetic group
            0x80 => ("ADD A,B".to_string(), 1),
            0x81 => ("ADD A,C".to_string(), 1),
            0x82 => ("ADD A,D".to_string(), 1),
            0x83 => ("ADD A,E".to_string(), 1),
            0x84 => ("ADD A,H".to_string(), 1),
            0x85 => ("ADD A,L".to_string(), 1),
            0x86 => ("ADD A,(HL)".to_string(), 1),
            0x87 => ("ADD A,A".to_string(), 1),
            
            0x88 => ("ADC A,B".to_string(), 1),
            0x89 => ("ADC A,C".to_string(), 1),
            0x8A => ("ADC A,D".to_string(), 1),
            0x8B => ("ADC A,E".to_string(), 1),
            0x8C => ("ADC A,H".to_string(), 1),
            0x8D => ("ADC A,L".to_string(), 1),
            0x8E => ("ADC A,(HL)".to_string(), 1),
            0x8F => ("ADC A,A".to_string(), 1),
            
            0x90 => ("SUB B".to_string(), 1),
            0x91 => ("SUB C".to_string(), 1),
            0x92 => ("SUB D".to_string(), 1),
            0x93 => ("SUB E".to_string(), 1),
            0x94 => ("SUB H".to_string(), 1),
            0x95 => ("SUB L".to_string(), 1),
            0x96 => ("SUB (HL)".to_string(), 1),
            0x97 => ("SUB A".to_string(), 1),
            
            0x98 => ("SBC A,B".to_string(), 1),
            0x99 => ("SBC A,C".to_string(), 1),
            0x9A => ("SBC A,D".to_string(), 1),
            0x9B => ("SBC A,E".to_string(), 1),
            0x9C => ("SBC A,H".to_string(), 1),
            0x9D => ("SBC A,L".to_string(), 1),
            0x9E => ("SBC A,(HL)".to_string(), 1),
            0x9F => ("SBC A,A".to_string(), 1),
            
            0xA0 => ("AND B".to_string(), 1),
            0xA1 => ("AND C".to_string(), 1),
            0xA2 => ("AND D".to_string(), 1),
            0xA3 => ("AND E".to_string(), 1),
            0xA4 => ("AND H".to_string(), 1),
            0xA5 => ("AND L".to_string(), 1),
            0xA6 => ("AND (HL)".to_string(), 1),
            0xA7 => ("AND A".to_string(), 1),
            
            0xA8 => ("XOR B".to_string(), 1),
            0xA9 => ("XOR C".to_string(), 1),
            0xAA => ("XOR D".to_string(), 1),
            0xAB => ("XOR E".to_string(), 1),
            0xAC => ("XOR H".to_string(), 1),
            0xAD => ("XOR L".to_string(), 1),
            0xAE => ("XOR (HL)".to_string(), 1),
            0xAF => ("XOR A".to_string(), 1),
            
            0xB0 => ("OR B".to_string(), 1),
            0xB1 => ("OR C".to_string(), 1),
            0xB2 => ("OR D".to_string(), 1),
            0xB3 => ("OR E".to_string(), 1),
            0xB4 => ("OR H".to_string(), 1),
            0xB5 => ("OR L".to_string(), 1),
            0xB6 => ("OR (HL)".to_string(), 1),
            0xB7 => ("OR A".to_string(), 1),
            
            0xB8 => ("CP B".to_string(), 1),
            0xB9 => ("CP C".to_string(), 1),
            0xBA => ("CP D".to_string(), 1),
            0xBB => ("CP E".to_string(), 1),
            0xBC => ("CP H".to_string(), 1),
            0xBD => ("CP L".to_string(), 1),
            0xBE => ("CP (HL)".to_string(), 1),
            0xBF => ("CP A".to_string(), 1),
            
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
            
            // Jumps and calls
            0x18 => self.decode_jr_e(data),
            0x20 => self.decode_jr_cc_e(data, "NZ"),
            0x28 => self.decode_jr_cc_e(data, "Z"),
            0x30 => self.decode_jr_cc_e(data, "NC"),
            0x38 => self.decode_jr_cc_e(data, "C"),
            
            0xC2 => self.decode_jp_cc_nn(data, "NZ"),
            0xC3 => self.decode_jp_nn(data),
            0xCA => self.decode_jp_cc_nn(data, "Z"),
            0xD2 => self.decode_jp_cc_nn(data, "NC"),
            0xDA => self.decode_jp_cc_nn(data, "C"),
            0xE9 => ("JP (HL)".to_string(), 1),
            0xF2 => self.decode_jp_cc_nn(data, "P"),
            0xFA => self.decode_jp_cc_nn(data, "M"),
            
            0xC4 => self.decode_call_cc_nn(data, "NZ"),
            0xCC => self.decode_call_cc_nn(data, "Z"),
            0xCD => self.decode_call_nn(data),
            0xD4 => self.decode_call_cc_nn(data, "NC"),
            0xDC => self.decode_call_cc_nn(data, "C"),
            
            0xC0 => ("RET NZ".to_string(), 1),
            0xC8 => ("RET Z".to_string(), 1),
            0xC9 => ("RET".to_string(), 1),
            0xD0 => ("RET NC".to_string(), 1),
            0xD8 => ("RET C".to_string(), 1),
            
            // Stack operations
            0xC1 => ("POP BC".to_string(), 1),
            0xC5 => ("PUSH BC".to_string(), 1),
            0xD1 => ("POP DE".to_string(), 1),
            0xD5 => ("PUSH DE".to_string(), 1),
            0xE1 => ("POP HL".to_string(), 1),
            0xE5 => ("PUSH HL".to_string(), 1),
            0xF1 => ("POP AF".to_string(), 1),
            0xF5 => ("PUSH AF".to_string(), 1),
            
            // Increment/Decrement
            0x04 => ("INC B".to_string(), 1),
            0x05 => ("DEC B".to_string(), 1),
            0x0C => ("INC C".to_string(), 1),
            0x0D => ("DEC C".to_string(), 1),
            0x14 => ("INC D".to_string(), 1),
            0x15 => ("DEC D".to_string(), 1),
            0x1C => ("INC E".to_string(), 1),
            0x1D => ("DEC E".to_string(), 1),
            0x24 => ("INC H".to_string(), 1),
            0x25 => ("DEC H".to_string(), 1),
            0x2C => ("INC L".to_string(), 1),
            0x2D => ("DEC L".to_string(), 1),
            0x34 => ("INC (HL)".to_string(), 1),
            0x35 => ("DEC (HL)".to_string(), 1),
            0x3C => ("INC A".to_string(), 1),
            0x3D => ("DEC A".to_string(), 1),
            
            0x03 => ("INC BC".to_string(), 1),
            0x0B => ("DEC BC".to_string(), 1),
            0x13 => ("INC DE".to_string(), 1),
            0x1B => ("DEC DE".to_string(), 1),
            0x23 => ("INC HL".to_string(), 1),
            0x2B => ("DEC HL".to_string(), 1),
            0x33 => ("INC SP".to_string(), 1),
            0x3B => ("DEC SP".to_string(), 1),
            
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
            
            // Extended instructions (CB prefix)
            0xCB => self.decode_cb_instruction(data),
            
            // Extended instructions (ED prefix)
            0xED => self.decode_ed_instruction(data),

            0xEB => ("EX DE,HL".to_string(), 1),
            0xF9 => ("LD SP,HL".to_string(), 1),
            0x3A => self.decode_ld_nn_hl(data),
            0x2A => self.decode_ld_nn_a(data),
            0x09 => ("ADD HL,BC".to_string(), 1),
            0x1A => ("LD A,(DE)".to_string(), 1),
            0x0A => ("LD A,(BC)".to_string(), 1),
            0x10 => ("STOP".to_string(), 1),
            0x19 => ("ADD HL,DE".to_string(), 1),
            0x29 => ("ADD HL,HL".to_string(), 1),
            0x39 => ("ADD HL,SP".to_string(), 1),
            0xCF => ("RST 08H".to_string(), 1),
            0xC7 => ("RST 00H".to_string(), 1),
            0xD7 => ("RST 10H".to_string(), 1),
            0xD9 => ("EXX".to_string(), 1),
            0xE3 => ("EX (SP),HL".to_string(), 1),
            0xF8 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let offset = data[1] as i8;
                (format!("LD HL,SP+{:02X}H", offset), 2)
            },
            0xFC => {
                if data.len() < 3 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], data[2]]);
                (format!("LD A,({:04X}H)", addr), 3)
            },
            0xDF => ("RST 18H".to_string(), 1),
            0xDB => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let port = data[1];
                (format!("IN A,({:02X}H)", port), 2)
            },
            0xE2 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], 0]);
                (format!("LD A,({:04X}H)", addr), 2)
            },
            0xE7 => ("RST 20H".to_string(), 1),
            0xEA => {
                if data.len() < 3 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], data[2]]);
                (format!("LD ({:04X}H),A", addr), 3)
            },
            0xE0 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], 0]);
                (format!("LD A,({:04X}H)", addr), 2)
            },
            0xF7 => ("RST 30H".to_string(), 1),
            0xF4 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], 0]);
                (format!("LD A,({:04X}H)", addr), 2)
            },
            0xF0 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], 0]);
                (format!("LD A,({:04X}H)", addr), 2)
            },
            0xD3 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let port = data[1];
                (format!("OUT ({:02X}H),A", port), 2)
            },
            0xDD => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let opcode = data[1];
                self.decode_fd_instruction(opcode, data)
            },
            0xE8 => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let offset = data[1] as i8;
                let target = self.pc.wrapping_add(2).wrapping_add(offset as u16);
                (format!("RET {:04X}H", target), 2)
            },
            0xFD => {
                if data.len() < 2 {
                    return ("???".to_string(), 1);
                }
                let opcode = data[1];
                self.decode_fd_instruction(opcode, data)
            },

            0xFF => ("RST 38H".to_string(), 1),
            0x08 => {
                if data.len() < 3 {
                    return ("???".to_string(), 1);
                }
                let addr = u16::from_le_bytes([data[1], data[2]]);
                (format!("DB {:04X}H", addr), 3)
            },


            
            _ => (format!("DB ${:02X}", opcode), 1),
        }
    }
    
    fn decode_ld_r_n(&self, data: &[u8], reg: &str) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("LD {},{:02X}H", reg, data[1]), 2)
    }
    
    fn decode_ld_hl_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("LD (HL),{:02X}H", data[1]), 2)
    }
    
    fn decode_ld_dd_nn(&self, data: &[u8], reg: &str) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("LD {},{:04X}H", reg, addr), 3)
    }

    fn decode_ld_nn_hl(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("LD ({:04X}H),HL", addr), 3)
    }

    fn decode_ld_nn_a(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("LD ({:04X}H),A", addr), 3)
    }
    
    fn decode_add_a_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("ADD A,{:02X}H", data[1]), 2)
    }
    
    fn decode_adc_a_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("ADC A,{:02X}H", data[1]), 2)
    }
    
    fn decode_sub_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("SUB {:02X}H", data[1]), 2)
    }
    
    fn decode_sbc_a_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("SBC A,{:02X}H", data[1]), 2)
    }
    
    fn decode_and_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("AND {:02X}H", data[1]), 2)
    }
    
    fn decode_xor_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("XOR {:02X}H", data[1]), 2)
    }
    
    fn decode_or_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("OR {:02X}H", data[1]), 2)
    }
    
    fn decode_cp_n(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        (format!("CP {:02X}H", data[1]), 2)
    }
    
    fn decode_jr_e(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        let offset = data[1] as i8;
        let target = self.pc.wrapping_add(2).wrapping_add(offset as u16);
        (format!("JR {:04X}H", target), 2)
    }
    
    fn decode_jr_cc_e(&self, data: &[u8], condition: &str) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        let offset = data[1] as i8;
        let target = self.pc.wrapping_add(2).wrapping_add(offset as u16);
        (format!("JR {},{:04X}H", condition, target), 2)
    }
    
    fn decode_jp_nn(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("JP {:04X}H", addr), 3)
    }
    
    fn decode_jp_cc_nn(&self, data: &[u8], condition: &str) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("JP {},{:04X}H", condition, addr), 3)
    }
    
    fn decode_call_nn(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("CALL {:04X}H", addr), 3)
    }
    
    fn decode_call_cc_nn(&self, data: &[u8], condition: &str) -> (String, usize) {
        if data.len() < 3 {
            return ("???".to_string(), 1);
        }
        let addr = u16::from_le_bytes([data[1], data[2]]);
        (format!("CALL {},{:04X}H", condition, addr), 3)
    }
    
    fn decode_cb_instruction(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        
        let opcode = data[1];
        let reg_names = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
        let reg = reg_names[opcode as usize & 0x07];
        
        match opcode {
            0x00..=0x07 => (format!("RLC {}", reg), 2),
            0x08..=0x0F => (format!("RRC {}", reg), 2),
            0x10..=0x17 => (format!("RL {}", reg), 2),
            0x18..=0x1F => (format!("RR {}", reg), 2),
            0x20..=0x27 => (format!("SLA {}", reg), 2),
            0x28..=0x2F => (format!("SRA {}", reg), 2),
            0x30..=0x37 => (format!("SLL {}", reg), 2),
            0x38..=0x3F => (format!("SRL {}", reg), 2),
            0x40..=0x7F => {
                let bit = (opcode - 0x40) / 8;
                (format!("BIT {},{}", bit, reg), 2)
            },
            0x80..=0xBF => {
                let bit = (opcode - 0x80) / 8;
                (format!("RES {},{}", bit, reg), 2)
            },
            0xC0..=0xFF => {
                let bit = (opcode - 0xC0) / 8;
                (format!("SET {},{}", bit, reg), 2)
            },
        }
    }
    
    fn decode_fd_instruction(&self, opcode: u8, data: &[u8]) -> (String, usize) {
        match opcode {
            0x7C => ("LD A,IYH".to_string(), 2),
            0x7D => ("LD A,IYL".to_string(), 2),
            0xE1 => ("POP IY".to_string(), 2),
            0xE5 => ("PUSH IY".to_string(), 2),
            0x21 => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD IY,{:04X}H", addr), 4)
            },
            _ => ("???".to_string(), 1),
        }
    }

    fn decode_ed_instruction(&self, data: &[u8]) -> (String, usize) {
        if data.len() < 2 {
            return ("???".to_string(), 1);
        }
        
        let opcode = data[1];
        match opcode {
            0x40 => ("IN B,(C)".to_string(), 2),
            0x41 => ("OUT (C),B".to_string(), 2),
            0x42 => ("SBC HL,BC".to_string(), 2),
            0x43 => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD ({:04X}H),BC", addr), 4)
            },
            0x44 => ("NEG".to_string(), 2),
            0x45 => ("RETN".to_string(), 2),
            0x46 => ("IM 0".to_string(), 2),
            0x47 => ("LD I,A".to_string(), 2),
            0x48 => ("IN C,(C)".to_string(), 2),
            0x49 => ("OUT (C),C".to_string(), 2),
            0x4A => ("ADC HL,BC".to_string(), 2),
            0x4B => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD BC,({:04X}H)", addr), 4)
            },
            0x4F => ("LD R,A".to_string(), 2),
            0x50 => ("IN D,(C)".to_string(), 2),
            0x51 => ("OUT (C),D".to_string(), 2),
            0x52 => ("SBC HL,DE".to_string(), 2),
            0x53 => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD ({:04X}H),DE", addr), 4)
            },
            0x56 => ("IM 1".to_string(), 2),
            0x57 => ("LD A,I".to_string(), 2),
            0x58 => ("IN E,(C)".to_string(), 2),
            0x59 => ("OUT (C),E".to_string(), 2),
            0x5A => ("ADC HL,DE".to_string(), 2),
            0x5B => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD DE,({:04X}H)", addr), 4)
            },
            0x5E => ("IM 2".to_string(), 2),
            0x5F => ("LD A,R".to_string(), 2),
            0x60 => ("IN H,(C)".to_string(), 2),
            0x61 => ("OUT (C),H".to_string(), 2),
            0x62 => ("SBC HL,HL".to_string(), 2),
            0x67 => ("RRD".to_string(), 2),
            0x68 => ("IN L,(C)".to_string(), 2),
            0x69 => ("OUT (C),L".to_string(), 2),
            0x6A => ("ADC HL,HL".to_string(), 2),
            0x6F => ("RLD".to_string(), 2),
            0x70 => ("IN (C)".to_string(), 2),
            0x71 => ("OUT (C),0".to_string(), 2),
            0x72 => ("SBC HL,SP".to_string(), 2),
            0x73 => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD ({:04X}H),SP", addr), 4)
            },
            0x78 => ("IN A,(C)".to_string(), 2),
            0x79 => ("OUT (C),A".to_string(), 2),
            0x7A => ("ADC HL,SP".to_string(), 2),
            0x7B => {
                if data.len() < 4 {
                    return ("???".to_string(), 2);
                }
                let addr = u16::from_le_bytes([data[2], data[3]]);
                (format!("LD SP,({:04X}H)", addr), 4)
            },
            0xA0 => ("LDI".to_string(), 2),
            0xA1 => ("CPI".to_string(), 2),
            0xA2 => ("INI".to_string(), 2),
            0xA3 => ("OUTI".to_string(), 2),
            0xA8 => ("LDD".to_string(), 2),
            0xA9 => ("CPD".to_string(), 2),
            0xAA => ("IND".to_string(), 2),
            0xAB => ("OUTD".to_string(), 2),
            0xB0 => ("LDIR".to_string(), 2),
            0xB1 => ("CPIR".to_string(), 2),
            0xB2 => ("INIR".to_string(), 2),
            0xB3 => ("OTIR".to_string(), 2),
            0xB8 => ("LDDR".to_string(), 2),
            0xB9 => ("CPDR".to_string(), 2),
            0xBA => ("INDR".to_string(), 2),
            0xBB => ("OTDR".to_string(), 2),
            _ => (format!("DB ${:02X},${:02X}", data[0], opcode), 2),
        }
    }
}