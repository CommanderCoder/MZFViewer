#!/usr/bin/env python3
"""
ZX80 Binary to BASIC Converter (Enhanced)
- Adds support for inverse characters (e.g., %A for inverse A)
- Detects when tokens expect line numbers (e.g., GOTO 100)
"""

import sys
import os
from typing import Dict, Optional
import struct

def bytes_to_hex_str(byte_data):
    """Helper function to convert a byte string to a hex string for display."""
    return ' '.join(f'{b:02x}' for b in byte_data)

def read_five_byte_float(infile):

    type_byte = infile.read(1)
    value_bytes = infile.read(5)

    # --- Integral Format ---
    # Used for integers between -65535 and +65535.
    if type_byte == 0x0E:
        # The 5 bytes are structured as: [0x00, sign_byte, val_low, val_high, 0x00]
        sign_byte = value_bytes[1]
        
        # Unpack the 2-byte little-endian unsigned integer from the 3rd and 4th bytes.
        unsigned_val = struct.unpack('<H', value_bytes[2:4])[0]

        if sign_byte == 0x00:  # Positive number
            return str(unsigned_val)
        elif sign_byte == 0xFF:  # Negative number
            return str(unsigned_val - 65536)
        else:
            raise ValueError(f"Invalid sign byte for integral format: {hex(sign_byte)}")

    # --- Floating Point Format ---
    # Used for all other numbers.
    elif type_byte == 0x7E:
        # The 5 bytes are structured as: [exponent, mantissa_b1, mantissa_b2, mantissa_b3, mantissa_b4]
        exponent = value_bytes[0] - 128
        
        # Unpack the 4-byte big-endian mantissa.
        mantissa_raw = struct.unpack('>I', value_bytes[1:5])[0]

        # The Most Significant Bit (MSB) of the mantissa is the sign bit.
        sign = -1 if (mantissa_raw & 0x80000000) else 1

        # The actual mantissa value is in the lower 31 bits.
        mantissa_frac = mantissa_raw & 0x7FFFFFFF

        # The original MSB is an implicit '1' due to normalization. We restore it.
        mantissa_reconstructed = mantissa_frac | 0x80000000

        # The final value is calculated by scaling the reconstructed mantissa
        # by the exponent. We use pow(2.0, exp) to ensure floating-point arithmetic.
        # The mantissa is scaled down by 2^32.
        value = sign * mantissa_reconstructed * pow(2.0, exponent - 32)
        
        # For cleaner output, if the float is a whole number, format as an integer.
        if value == int(value):
            return str(int(value))
        else:
            return str(value)

    else:
        raise ValueError(f"Unknown number type byte provided: {hex(type_byte)}")


def process_simple_numeric_var(name_byte, infile):
    """
    Processes a simple numeric variable (e.g., A, B, I).
    Format: 1-byte name, 2-byte value.
    The name byte is the ASCII code of the variable letter.
    """
    var_name = chr(name_byte)
    value_str = read_five_byte_float(infile)
    print(f"  - Simple Numeric Var '{var_name}':")
    print(f"    - Value (5 bytes): {value_str}")


def process_simple_string_var(name_byte, infile):
    """
    Processes a simple string variable (e.g., A$, B$).
    Format: 1-byte name, 2-byte length, string data.
    The name byte is the ASCII code of the variable letter.
    """
    var_name = chr(name_byte) + '$'
    
    # Read the length of the string (in little-endian)
    len_bytes = infile.read(2)
    str_len = int.from_bytes(len_bytes, 'little')
    
    # Read the string data
    str_data_bytes = infile.read(str_len)
    str_data = str_data_bytes.decode('ascii', errors='replace')

    print(f"  - Simple String Var '{var_name}':")
    print(f"    - Length: {str_len}")
    print(f"    - Value: '{str_data}'")

def process_long_name_string_var(name_byte, infile):
    """
    Processes a simple string variable with a multi-character name (e.g., name$).
    Format: 1-byte marker (0xA1-0xBA), name chars, '$' terminator (0x24), 2-byte length, string data.
    """
    name_chars = []
    while True:
        char_byte = infile.read(1)
        if not char_byte or ord(char_byte) == 0x24: # Stop at '$' or end of file
            break
        name_chars.append(char_byte.decode('ascii', errors='?'))
    
    var_name = "".join(name_chars) + '$'

    # Read the length of the string (in little-endian)
    len_bytes = infile.read(2)
    str_len = int.from_bytes(len_bytes, 'little')
    
    # Read the string data
    str_data_bytes = infile.read(str_len)
    str_data = str_data_bytes.decode('ascii', errors='replace')

    print(f"  - Long Name String Var '{var_name}':")
    print(f"    - Length: {str_len}")
    print(f"    - Value: '{str_data}'")


def process_numeric_array(name_byte, infile):
    """
    Processes a numeric array variable (e.g., A(), B()).
    Format: 1-byte name (0x81-0x9A), 2-byte size, 1-byte dims, 2-bytes per dim, data.
    """
    var_name = chr(name_byte - 0x81 + ord('A'))
    print(f"  - Numeric Array '{var_name}()':")
    
    # Read the size of the data to follow (in little-endian)
    data_size_bytes = infile.read(2)
    data_size = int.from_bytes(data_size_bytes, 'little')
    print(f"    - Data Size: {data_size} bytes")

    # Read the number of dimensions
    num_dims_byte = infile.read(1)
    num_dims = int.from_bytes(num_dims_byte, 'little')
    print(f"    - Dimensions: {num_dims}")

    # Read the range for each dimension
    dims = []
    for i in range(num_dims):
        dim_range_bytes = infile.read(2)
        dim_range = int.from_bytes(dim_range_bytes, 'little')
        dims.append(dim_range)
        print(f"      - Dim {i+1} Range: {dim_range}")

    # The remaining data contains the array values (5 bytes each)
    # We can calculate how many bytes of data to read.
    bytes_read_so_far = 2 + 1 + (num_dims * 2)
    data_to_read = data_size - bytes_read_so_far
    array_data = infile.read(data_to_read)
    print(f"    - Data Bytes ({data_to_read} bytes): {bytes_to_hex_str(array_data)}")

def process_string_array(name_byte, infile):
    """
    Processes a string array variable (e.g., A$(), B$()).
    Format: 1-byte name (0xC1-0xDA), 2-byte size, 1-byte dims, 2-bytes per dim, data.
    """
    var_name = chr(name_byte - 0xC1 + ord('A')) + '$'
    print(f"  - String Array '{var_name}()':")

    data_size_bytes = infile.read(2)
    data_size = int.from_bytes(data_size_bytes, 'little')
    print(f"    - Data Size: {data_size} bytes")

    num_dims_byte = infile.read(1)
    num_dims = int.from_bytes(num_dims_byte, 'little')
    print(f"    - Dimensions: {num_dims}")

    dims = []
    for i in range(num_dims):
        dim_range_bytes = infile.read(2)
        dim_range = int.from_bytes(dim_range_bytes, 'little')
        dims.append(dim_range)
        print(f"      - Dim {i+1} Range: {dim_range}")
    
    bytes_read_so_far = 2 + 1 + (num_dims * 2)
    data_to_read = data_size - bytes_read_so_far
    array_data = infile.read(data_to_read)
    # Attempt to decode as ZX81 characters, replacing errors
    try:
        # Note: ZX81 character set is not standard ASCII. This is a best-effort display.
        str_data = array_data.decode('ascii', errors='replace')
        print(f"    - String Data ({data_to_read} bytes): '{str_data}'")
    except Exception:
        print(f"    - String Data (raw): {bytes_to_hex_str(array_data)}")




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
        self.tokens.update({
            213: " THEN ", 214: " TO ", 219: "NOT ", 224: " AND ", 225: " OR ",
            226: "**", 230: " LIST ", 231: " RETURN ", 232: " CLS ", 233: " DIM ",
            234: " SAVE ", 235: " FOR ", 236: " GO TO ", 237: " POKE ",
            238: " INPUT ", 239: " RANDOMISE ", 240: " LET ", 243: " NEXT ",
            244: " PRINT ", 246: " NEW ", 247: " RUN ", 248: " STOP ",
            249: " CONTINUE ", 250: " IF ", 251: " GO SUB ", 252: " LOAD ",
            253: " CLEAR ", 254: " REM "
        })
        if self.zxpand_enabled:
            self.tokens.update({241: " CONFIG ", 245: " DELETE ", 255: " CAT "})

    def _initialize_character_maps(self):
        NUMBER_0, LETTER_A = 28, 38
        for i in range(10):
            self.zx_to_ascii[NUMBER_0 + i] = str(i)
        for i in range(26):
            self.zx_to_ascii[LETTER_A + i] = chr(ord('A') + i)
        self.zx_to_ascii.update({
            0: ' ', 1: '"', 12: '#', 13: '$', 14: ':', 15: '?',
            218: '(', 217: ')', 220: '-', 221: '+', 222: '*', 223: '/',
            227: '=', 228: '>', 229: '<', 215: ';', 216: ',', 27: '.'
        })
        self.graphics.update({
            0: "  ", 2: ": ", 3: "..", 4: "' ", 5: " '", 6: ". ", 7: " .",
            8: ".'", 9: "##", 10: ",,", 11: "~~", 128: "::", 130: " :",
            131: "''", 132: ".:", 133: ":.", 134: "':", 135: ":'",
            136: "'.", 137: "@@", 138: ";;", 139: "!!"
        })
        self.zx_token_chars.update({
            2: 'º', 3: '®', 4: '¶', 5: '·', 6: '¹', 7: '²', 8: '»', 9: '½',
            10: '¾', 11: '¿', 128: '«', 129: '†', 130: '°', 131: '¸',
            132: '¬', 133: 'ª', 134: '¯', 135: '¼', 136: '±', 137: '³',
            138: '´', 139: 'µ'
        })
        for i in range(26):
            self.zx_token_chars[ord('a') + i + 0x45] = chr(ord('a') + i)

    def token_supports_line_number(self, token_code: int) -> bool:
        return token_code in {236, 251, 247, 230}  # GOTO, GOSUB, RUN, LIST

    def decode_line(self, line_bytes: list) -> str:
        result, i = "", 0
        in_rem_comment = False # State variable for REM 

        while i < len(line_bytes):
            byte = line_bytes[i]

            # --- Special Handling for REM Comments ---
            # If we are in a REM comment, all subsequent bytes are literal characters
            if in_rem_comment:
                result += self.decode_character(byte)
            elif byte in self.tokens:
                result += self.tokens[byte]
                # Specific handling after certain tokens
                if byte == 254: # REM token 
                    in_rem_comment = True # All subsequent bytes are literal comment
                elif self.token_supports_line_number(byte):
                    j, line_number_str = i + 1, ""
                    while j < len(line_bytes) and line_bytes[j] in self.zx_to_ascii:
                        ch = self.zx_to_ascii[line_bytes[j]]
                        if not ch.isdigit():
                            break
                        line_number_str += ch
                        j += 1
                    if line_number_str:
                        result += line_number_str
                        i = j - 1
            else:
                result += self.decode_character(byte)
            i += 1
        return result


    def decode_character(self, zx_char: int) -> str:
        if zx_char & 0x80:
            base = zx_char & 0x7F
            if base in self.zx_to_ascii:
                return f"%{self.zx_to_ascii[base]}"
            # return f"%[UNK:{base}]"
        if zx_char in self.graphics:
            return self.graphics[zx_char]
        if zx_char in self.zx_token_chars:
            return self.zx_token_chars[zx_char]
        if zx_char in self.zx_to_ascii:
            return self.zx_to_ascii[zx_char]
        return f"[UNK:{zx_char}]"
    
    def decode_varname(self, var_type: int) -> str:
        var_index = (var_type & 0x1F) + 0x20 # lower 5 bits
        return self.decode_character(var_index)

    def process_for_loop_var(self, name_byte, infile):
        """
        Processes a FOR-NEXT loop control variable.
        Format: 
        """
        var_name = self.decode_character((name_byte & 0x1f) + 0x20 )
        print(f"  - FOR-NEXT Loop Var '{var_name}':")

        value_bytes = infile.read(2)
        current_val = int.from_bytes(value_bytes, byteorder='little', signed=False)
        print(f"    - Current Value: {(current_val)}")

        value_bytes = infile.read(2)
        limit_val = int.from_bytes(value_bytes, byteorder='little', signed=False)
        print(f"    - Limit Value:   {(limit_val)}")

        value_bytes = infile.read(2)
        stmt_num = int.from_bytes(value_bytes, byteorder='little', signed=False)
        print(f"    - Statement Num: {stmt_num}")

    def decode_binary_file(self, input_file: str, output_file: str) -> bool:
        try:
            with open(input_file, 'rb') as infile:
                # Check if the file is a valid ZX80 binary file
                header = infile.read(40)
                if len(header) < 40:
                    print(f"Error: {input_file} is not a valid ZX80 binary file.")
                    return False
                end = (header[9] << 8 | header[8])-0x4000  # Get the end address from the header
                print(f"Header: {end:04X} ")
                with open(output_file, 'w', encoding='utf-8') as outfile:
                    while True:
                        if infile.tell() >= end:
                            # display the variables according rules
                            byte = infile.read(1)
                            if not byte:
                                print("\n--- End of file reached ---")
                                break

                            var_type = ord(byte)

                            # End of variables marker
                            if var_type == 0x80:
                                print("\n--- End of variables marker (0x80) found ---")
                                break
                            
                            # Check for simple numeric variable (A-Z)
                            # Using ASCII range for this example as per some interpretations
                            elif 0x61 <= var_type <= 0x7F:
                                var_name = self.decode_varname(var_type)

                                # Read next two bytes for little-endian integer
                                value_bytes = infile.read(2)
                                value = int.from_bytes(value_bytes, byteorder='little', signed=False)

                                print(f"  - Simple Numeric Var '{var_name}':")
                                print(f"    - Value (2 bytes): {value}")


                            # check for long var name
                            elif 0x41 <= var_type <= 0x5B:
                                
                                var_name = self.decode_varname(var_type)
                                # terminate when top bit of byte is 1
                                while True:
                                    byte = infile.read(1)
                                    if not byte:
                                        break
                                    byte = byte[0] 
                                    var_name+=self.decode_character((byte & 0x1f)+0x20)
                                    if byte & 0x80 == 0x80:
                                        break

                                value_bytes = infile.read(2)
                                value = int.from_bytes(value_bytes, byteorder='little', signed=False)

                                print(f"  - Int (longname) Var '{var_name}':")
                                print(f"    - Value : {value}")


                            # Check String name A$ - Z$
                            elif 0x81 <= var_type <= 0x9F:
                                var_name = self.decode_varname(var_type)
                                # process_long_name_string_var(var_type, infile)
                                print(f"  - String Var '{var_name}':")
                                string_value=[]
                                # terminate with byte is 0x01
                                while True:
                                    byte = infile.read(1)
                                    if not byte:
                                        break
                                    byte = byte[0] 
                                    if byte == 0x01:
                                        break
                                    string_value+=self.decode_character((byte & 0x1f)+0x20)
                                    
                                print(f"    - Value : {string_value}")
                            # integer array A()-Z()
                            elif 0xA1 <= var_type <= 0xBF:
                                var_name = self.decode_varname(var_type)
                                array_len = int(infile.read(1)[0])
                                array_values = infile.read((array_len+1)*2) # two bytes per entry
                                print(f"  - Integer Array '{var_name}':")
                                print(f"    - Values : {array_values}")
                            # Check for FOR-NEXT loop control variable
                            elif 0xE1 <= var_type <= 0xFF:
                                var_name = self.decode_varname(var_type)
                                self.process_for_loop_var(var_type, infile)
                            else:
                                print(f"Unknown or unhandled variable type code: {var_type:02x}")
                                # Attempt to find the next variable or end marker
                            outfile.write(f"{var_type:02x} ")
                            continue

                        start = infile.tell()
                        # get the line number
                        high = infile.read(1)
                        if not high:
                            break
                        high = high[0]
                        if high == 0x80:
                            break
                        low = infile.read(1)
                        if not low:
                            break
                        low = low[0]
                        line_number = (high << 8) | low
                        # get the length of the line
                        line_bytes = []
                        while True:
                            byte = infile.read(1)
                            if not byte:
                                break
                            byte = byte[0]
                            if byte == 0x76:
                                break
                            line_bytes.append(byte)
                        # decode the line
                        line_text = self.decode_line(line_bytes)
                        outfile.write(f"{start} -- {line_number} {line_text} -- {infile.tell()}\n")


            print(f"Successfully decoded {input_file} to {output_file}")
            return True
        except FileNotFoundError:
            print(f"Error: Cannot open input file {input_file}")
            return False
        except Exception as e:
            print(f"Error during conversion: {e}")
            return False

    def display_file_info(self, filename: str):
        try:
            with open(filename, 'rb') as file:
                print(f"File: {filename}")
                print("System Variables (first 40 bytes):")
                for i in range(40):
                    byte = file.read(1)
                    if not byte:
                        break
                    print(f"{byte[0]:02X} ", end="")
                    if (i + 1) % 16 == 0:
                        print()
                print("\n")
                file.seek(0, 2)
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
        return 1

    input_file = sys.argv[1]
    output_file = input_file + ".bas"
    zxpand_enabled = False
    info_only = False

    for arg in sys.argv[2:]:
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

    return 0 if decoder.decode_binary_file(input_file, output_file) else 1

if __name__ == "__main__":
    sys.exit(main())
