import {
    F64_SIZE,
    I16_SIZE,
    I32_SIZE,
    I64_SIZE,
    PTR_SIZE,
} from "../memory_constants.mjs";
import {
    read_frame_ptr,
    read_ptr,
    read_int,
    store_int,
    read_string,
    read_double,
} from "../memory_operations.mjs";
import {signed_to_unsigned_i16, signed_to_unsigned_i32, signed_to_unsigned_i64} from "../number_operations.mjs";

// print a null-terminated string from memory to console
// int printf(const char *format, ...);
export function printf(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load param: addr of format str
        const fp = read_frame_ptr(memory);
        let format_str_ptr = read_ptr(fp + PTR_SIZE + I32_SIZE, memory);
        let vararg_ptr = fp + PTR_SIZE + I32_SIZE + PTR_SIZE;

        const next_char = () => {
            const byte = memory[format_str_ptr];
            format_str_ptr++;
            return String.fromCharCode(byte);
        };

        const next_vararg_int = (byte_size) => {
            const value = read_int(vararg_ptr, byte_size, memory);
            vararg_ptr += byte_size;
            return value;
        };

        const next_vararg_double = () => {
            const value = read_double(vararg_ptr, memory);
            vararg_ptr += F64_SIZE;
            return value;
        }

        const next_vararg_ptr = () => {
            return next_vararg_int(PTR_SIZE);
        }

        let str = "";
        let c = next_char();
        while (c !== "\0") {
            // handle format args
            if (c === "%") {
                c = next_char();
                if (c !== "%") {
                    // read next vararg
                    let value;
                    switch (c) {
                        case "i":
                        case "d":
                            // int
                            value = next_vararg_int(I32_SIZE);
                            break;
                        case "u":
                            // unsigned int
                            value = next_vararg_int(I32_SIZE);
                            value = signed_to_unsigned_i32(value);
                            break;
                        case "h":
                            // short
                            c = next_char();
                            if (c != "i" && c != "d" && c != "u") {
                                console.log("Error: invalid format specifier to printf");
                            }
                            value = next_vararg_int(I16_SIZE);
                            if (c === "u") {
                                value = signed_to_unsigned_i16(value);
                            }
                            break;
                        case "l":
                            // long
                            c = next_char();
                            if (c != "i" && c != "d" && c != "u") {
                                console.log("Error: invalid format specifier to printf");
                            }
                            value = next_vararg_int(I64_SIZE);
                            if (c === "u") {
                                value = signed_to_unsigned_i64(value);
                            }
                            break;
                        case "f":
                            // double
                            value = next_vararg_double();
                            break;
                        case "s":
                            // string
                            const str_ptr = next_vararg_ptr();
                            value = read_string(str_ptr, memory);
                            break;
                        default:
                            console.log("Error: invalid format specifier to printf");
                            break;
                    }

                    str += `${value}`;
                    c = next_char();
                    continue;
                }
            }
            str += c;
            c = next_char();
        }

        process.stdout.write(str); // only works in node.js, use console.log otherwise, but that prints newline

        // write return value
        store_int(fp + PTR_SIZE, I32_SIZE, 0n, memory);
    };
}
