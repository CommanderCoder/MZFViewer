#!/usr/bin/env python3
"""
ZX80 Binary to BASIC Converter
Converts ZX80 binary files back to readable BASIC text
"""

import sys
import os
from typing import Dict, Optional

class ZX80BasicDecoder:
    def __init__(self, zxpand_enabled: bool = False):
        self.zxpand_enabled = zxpand_enabled
        self.tokens: Dict[int, str] = {}
        self.zx_to_ascii: Dict[int, str] = {}
        self.graphics: Dict[int, str] = {}
        self.zx_token_chars: Dict[int, str] = {}
        
        self._initialize_tokens()
        self._initialize_character_maps()
    
    def _initialize_tokens(self):
        """Initialize token mappings"""
        # Basic ZX80 tokens
        self.tokens.update({
            213: " THEN ",
            214: " TO ",
            219: "NOT ",
            224: " AND ",
            225: " OR ",
            226: "**",
            230: " LIST ",
            231: " RETURN ",
            232: " CLS ",
            233: " DIM ",
            234: " SAVE ",
            235: " FOR ",
            236: " GO TO ",
            237: " POKE ",
            238: " INPUT ",
            239: " RANDOMISE ",
            240: " LET ",
            243: " NEXT ",
            244: " PRINT ",
            246: " NEW ",
            247: " RUN ",
            248: " STOP ",
            249: " CONTINUE ",
            250: " IF ",
            251: " GO SUB ",
            252: " LOAD ",
            253: " CLEAR ",
            254: " REM ",
        })
        
        # ZXpand tokens
        if self.zxpand_enabled:
            self.tokens.update({
                241: " CONFIG ",
                245: " DELETE ",
                255: " CAT ",
            })
    
    def _initialize_character_maps(self):
        """Initialize character mapping tables"""
        NUMBER_0 = 28
        LETTER_A = 38
        
        # Numbers 0-9
        for i in range(10):
            self.zx_to_ascii[NUMBER_0 + i] = str(i)
        
        # Letters A-Z
        for i in range(26):
            self.zx_to_ascii[LETTER_A + i] = chr(ord('A') + i)
        
        # Special characters
        self.zx_to_ascii.update({
            0: ' ',
            1: '"',
            12: '#',  # or '£'
            13: '$',
            14: ':',
            15: '?',
            218: '(',
            217: ')',
            220: '-',
            221: '+',
            222: '*',
            223: '/',
            227: '=',
            228: '>',
            229: '<',
            215: ';',
            216: ',',
            27: '.',
        })
        
        # Graphics characters
        self.graphics.update({
            0: "  ",
            2: ": ",
            3: "..",
            4: "' ",
            5: " '",
            6: ". ",
            7: " .",
            8: ".'",
            9: "##",
            10: ",,",
            11: "~~",
            128: "::",
            130: " :",
            131: "''",
            132: ".:",
            133: ":.",
            134: "':",
            135: ":'",
            136: "'.",
            137: "@@",
            138: ";;",
            139: "!!",
        })
        
        # ZX Token characters (extended character set)
        self.zx_token_chars.update({
            2: 'º',
            3: '®',
            4: '¶',
            5: '·',
            6: '¹',
            7: '²',
            8: '»',
            9: '½',
            10: '¾',
            11: '¿',
            128: '«',
            129: '†',
            130: '°',
            131: '¸',
            132: '¬',
            133: 'ª',
            134: '¯',
            135: '¼',
            136: '±',
            137: '³',
            138: '´',
            139: 'µ',
        })
        
        # Lowercase letters (a-z mapped to higher codes)
        for i in range(26):
            self.zx_token_chars[ord('a') + i + 0x45] = chr(ord('a') + i)
    
    def decode_line(self, line_bytes: list) -> str:
        """Decode a complete line of ZX80 bytes to text"""
        result = ""
        i = 0
        
        while i < len(line_bytes):
            byte = line_bytes[i]
            
            # Check for tokens first (they take precedence)
            if byte in self.tokens:
                result += self.tokens[byte]
            else:
                # Decode as regular character
                result += self.decode_character(byte)
            
            i += 1
        
        return result
    
    def decode_character(self, zx_char: int) -> str:
        """Decode a single ZX80 character to string (excluding tokens)"""
        # Check if it's an inverted character
        if zx_char & 0x80:
            if zx_char in self.zx_to_ascii:
                return self.zx_to_ascii[zx_char]
            return f"[INV:{zx_char}]"
        
        # Check for graphics
        if zx_char in self.graphics:
            return self.graphics[zx_char]
        
        # Check for ZX token characters
        if zx_char in self.zx_token_chars:
            return self.zx_token_chars[zx_char]
        
        # Check for regular ASCII characters
        if zx_char in self.zx_to_ascii:
            return self.zx_to_ascii[zx_char]
        
        # Unknown character
        return f"[UNK:{zx_char}]"
    
    def decode_binary_file(self, input_file: str, output_file: str) -> bool:
        """Decode a ZX80 binary file to BASIC text"""
        try:
            with open(input_file, 'rb') as infile:
                # Skip system variables (40 bytes)
                infile.seek(40)
                
                with open(output_file, 'w', encoding='utf-8') as outfile:
                    while True:
                        # Read potential line number high byte
                        high_byte = infile.read(1)
                        if not high_byte:
                            break
                        
                        high_byte = high_byte[0]
                        
                        # Check for end of program marker
                        if high_byte == 0x80:
                            break
                        
                        # Read line number low byte
                        low_byte = infile.read(1)
                        if not low_byte:
                            break
                        
                        low_byte = low_byte[0]
                        line_number = (high_byte << 8) | low_byte
                        
                        # Read line content until newline (0x76)
                        line_bytes = []
                        while True:
                            byte = infile.read(1)
                            if not byte:
                                break
                            
                            byte = byte[0]
                            if byte == 0x76:  # Newline character
                                break
                            
                            line_bytes.append(byte)
                        
                        # Decode the entire line content
                        line_content = self.decode_line(line_bytes)
                        outfile.write(f"{line_number} {line_content}\n")
            
            print(f"Successfully decoded {input_file} to {output_file}")
            return True
            
        except FileNotFoundError:
            print(f"Error: Cannot open input file {input_file}")
            return False
        except Exception as e:
            print(f"Error during conversion: {e}")
            return False
    
    def display_file_info(self, filename: str):
        """Display information about a ZX80 binary file"""
        try:
            with open(filename, 'rb') as file:
                print(f"File: {filename}")
                print("System Variables (first 40 bytes):")
                
                # Read and display system variables
                for i in range(40):
                    byte = file.read(1)
                    if not byte:
                        break
                    
                    print(f"{byte[0]:02X} ", end="")
                    if (i + 1) % 16 == 0:
                        print()
                
                print("\n")
                
                # Display file size
                file.seek(0, 2)  # Seek to end
                size = file.tell()
                print(f"File size: {size} bytes")
                
        except FileNotFoundError:
            print(f"Error: Cannot open file {filename}")
        except Exception as e:
            print(f"Error reading file: {e}")

def main():
    print("ZX80 Binary to BASIC Converter")
    print("==============================")
    
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <binary_file> [output_file] [--zxpand] [--info]")
        print("  binary_file: Input ZX80 binary file")
        print("  output_file: Output BASIC text file (optional, defaults to input.bas)")
        print("  --zxpand: Enable ZXpand token support")
        print("  --info: Display file information only")
        return 1
    
    input_file = sys.argv[1]
    output_file = input_file + ".bas"
    zxpand_enabled = False
    info_only = False
    
    # Parse command line arguments
    for i in range(2, len(sys.argv)):
        arg = sys.argv[i]
        if arg == "--zxpand":
            zxpand_enabled = True
        elif arg == "--info":
            info_only = True
        elif not arg.startswith('-'):
            output_file = arg
    
    decoder = ZX80BasicDecoder(zxpand_enabled)
    
    if info_only:
        decoder.display_file_info(input_file)
        return 0
    
    if decoder.decode_binary_file(input_file, output_file):
        print("Conversion completed successfully!")
        return 0
    else:
        print("Conversion failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())