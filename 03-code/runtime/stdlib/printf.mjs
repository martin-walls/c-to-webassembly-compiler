import {
  I16_SIZE,
  I32_SIZE,
  I64_SIZE,
  MAX_I16,
  MAX_I32,
  MAX_I64,
  PTR_SIZE,
} from "../memory_constants.mjs";
import {
  read_frame_ptr,
  read_ptr,
  store_i32,
  read_int,
} from "../memory_operations.mjs";

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
              if (value < 0) {
                value = MAX_I32 + value;
              }
              break;
            case "h":
              // short
              c = next_char();
              if (c != "i" && c != "d" && c != "u") {
                console.log("Error: invalid format specifier to printf");
              }
              value = next_vararg_int(I16_SIZE);
              if (c === "u") {
                value = MAX_I16 + value;
              }
              break;
            case "l":
              // long
              c = next_char();
              if (c != "i" && c != "d" && c != "u") {
                console.log("Error: invalid format specifier to printf");
              }
              // TODO: JS can't represent the full range of longs
              //       Loading all 8 bytes into a JS number won't work for negatives
              //       or big numbers.
              value = next_vararg_int(I32_SIZE);
              if (c === "u") {
                value = MAX_I64 + value;
              }
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
    store_i32(fp + PTR_SIZE, 0, memory);
  };
}
