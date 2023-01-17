import {I32_SIZE, PTR_SIZE} from "../memory_constants.mjs";
import {read_frame_ptr, store_i32, store_int} from "../memory_operations.mjs";

// int atoi(const char *str);
export function atoi(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load param str
        const fp = read_frame_ptr(memory);

        // todo

        // write return value
        store_int(fp + PTR_SIZE, I32_SIZE, 0, memory);
    }
}

// unsigned long strtoul(const char *nptr, char **endptr, int base);
export function strtoul(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        const fp = read_frame_ptr(memory);

        // todo

        // write return value
        store_int(fp + PTR_SIZE, I64_SIZE, 0, memory);
    }
}
