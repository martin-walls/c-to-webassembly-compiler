import {PTR_SIZE} from "../memory_constants.mjs";
import {
    read_frame_ptr,
    store_i32,
    read_i32,
} from "../memory_operations.mjs";


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
