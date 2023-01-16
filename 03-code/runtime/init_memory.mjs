import {PTR_SIZE} from "./memory_constants.mjs";
import {read_stack_ptr, store_ptr, store_stack_ptr} from "./memory_operations.mjs";

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
        store_ptr(argv + (i * PTR_SIZE), stack_ptr, memory);
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
