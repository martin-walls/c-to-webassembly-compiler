// size_t strlen(const char *s);

import {NULL, PTR_SIZE, SIZE_T_SIZE} from "../memory_constants.mjs";
import {read_frame_ptr, read_ptr, read_string, store_int, store_ptr} from "../memory_operations.mjs";

// (size_t = int)
export function strlen(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load param
        const fp = read_frame_ptr(memory);
        const param_ptr = fp + PTR_SIZE + SIZE_T_SIZE;
        const str_ptr = read_ptr(param_ptr, memory);

        const str = read_string(str_ptr, memory);

        const len = str.length;

        // write return value
        store_int(fp + PTR_SIZE, SIZE_T_SIZE, BigInt(len), memory);
    };
}

// char* strstr(const char *, const char *)
// Returns a pointer to the first occurrence of str2 in str1, or a null pointer if str2 is not part of str1
export function strstr(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load params
        const fp = read_frame_ptr(memory);
        let param_ptr = fp + PTR_SIZE + PTR_SIZE;
        const str1_ptr = read_ptr(param_ptr, memory);
        param_ptr += PTR_SIZE;
        const str2_ptr = read_ptr(param_ptr, memory);

        const str1 = read_string(str1_ptr, memory);
        const str2 = read_string(str2_ptr, memory);

        const offset = str1.indexOf(str2);

        if (offset === -1) {
            // return NULL
            store_ptr(fp + PTR_SIZE, NULL, memory);
        } else {
            // return pointer to occurrence in str1
            const result_ptr = str1_ptr + offset;
            store_ptr(fp + PTR_SIZE, result_ptr, memory);
        }
    }
}
