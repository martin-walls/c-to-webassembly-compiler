import {
    PTR_SIZE,
    read_frame_ptr,
    store_i32,
    read_i32,
    read_ptr,
} from "./memory_operations.mjs";

// print a null-terminated string from memory to console
// int printf(const char *format, ...);
export const printf = (wasm_memory, format_arg_count) => {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load param: addr of format str
        const fp = read_frame_ptr(memory);
        let format_str_ptr = read_ptr(fp + PTR_SIZE + 4, memory);

        let str = "";
        let b = memory[format_str_ptr];
        let c;
        // let argcounter = 0;
        while (b !== 0) {
            c = String.fromCharCode(b);
            // handle format args
            // if (c === "%") {
            //     offset++;
            //     b = memory[offset];
            //     c = String.fromCharCode(b);
            //     if (c !== "%" && argcounter < formatargs.length) {
            //         str += `${formatargs[argcounter]}`;
            //         argcounter++;
            //         offset++;
            //         b = memory[offset];
            //         continue;
            //     }
            // }
            str += c;
            format_str_ptr++;
            b = memory[format_str_ptr];
        }

        process.stdout.write(str); // only works in node.js, use console.log otherwise, but that prints newline

        // write return value
        store_i32(fp + PTR_SIZE, 0, memory);
    };
};

// int log(int x);
export function log(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load param x
        const fp = read_frame_ptr(memory);
        const x = read_i32(fp + PTR_SIZE + 4, memory);

        console.log(x);

        // write return value
        store_i32(fp + PTR_SIZE, 0, memory);
    };
}
