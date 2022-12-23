const PTR_SIZE = 4;
const STACK_PTR_ADDR = PTR_SIZE;

// put the program arguments into wasm memory
// args is an array of strings
export const put_args_into_memory = (args, wasm_memory) => {
    let memory = new Uint8Array(wasm_memory.buffer);

    let stack_ptr = read_stack_ptr(memory);
    const argc = args.length;
    const argv = stack_ptr;

    // allocate space for each pointer in argv
    stack_ptr += PTR_SIZE * argc;

    // store each arg after each other in memory and null-terminate
    for (let i; i < argc.length; i++) {
        // store arg pointer in space we allocated above
        store_ptr(stack_ptr, argv + (i * PTR_SIZE), memory);
        // store arg value at stack ptr
        const arg_bytes = new TextEncoder().encode(args[i]);
        for (const byte in arg_bytes) {
            memory[stack_ptr] = byte;
            stack_ptr += 1;
        }
        // null-terminate
        memory[stack_ptr] = 0;
        stack_ptr += 1;
    }

    store_stack_ptr(stack_ptr, memory);

    return {argc, argv};
}


const read_stack_ptr = (memory) => {
    // read stack ptr bytes from memory -- stored in little-endian order
    let stack_ptr = memory[STACK_PTR_ADDR];
    stack_ptr |= memory[STACK_PTR_ADDR + 1] << 8;
    stack_ptr |= memory[STACK_PTR_ADDR + 2] << 16;
    stack_ptr |= memory[STACK_PTR_ADDR + 3] << 24;
    return stack_ptr;
}


const store_stack_ptr = (stack_ptr, memory) => {
    store_ptr(stack_ptr, STACK_PTR_ADDR, memory);
}

const store_ptr = (ptr_value, address, memory) => {
    memory[address] = ptr_value & 0xFF;
    memory[address + 1] = (ptr_value >> 8) & 0xFF;
    memory[address + 2] = (ptr_value >> 16) & 0xFF;
    memory[address + 3] = (ptr_value >> 24) & 0xFF;
}
