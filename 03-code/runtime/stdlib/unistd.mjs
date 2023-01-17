// unsigned int sleep(unsigned int);
export function sleep(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);

        // todo

        // write return value
        store_int(fp + PTR_SIZE, I32_SIZE, 0, memory);
    }
}
