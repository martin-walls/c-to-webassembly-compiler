#!/usr/bin/env node
//
// usage: run.mjs <wasm_filename> [args...]
//
import {readFileSync} from "fs";
import {printf} from "./stdlib/stdio.mjs";
import {put_args_into_memory} from "./init_memory.mjs";
import {strtol, strtoul} from "./stdlib/stdlib.mjs";
import {strlen, strstr} from "./stdlib/string.mjs";
import {init_stack_ptr_log_file, log_stack_ptr} from "./profiler.mjs";


const run = async (filename, args) => {
    const buffer = readFileSync(filename);

    const stack_ptr_log_path = init_stack_ptr_log_file(filename);

    let memory = new WebAssembly.Memory({initial: 1});

    // functions that will be passed in to wasm
    const imports = {
        runtime: {
            memory: memory,
        },
        stdlib: {
            printf: printf(memory),
            strtol: strtol(memory),
            strtoul: strtoul(memory),
            strlen: strlen(memory),
            strstr: strstr(memory),
            log_stack_ptr: log_stack_ptr(memory, stack_ptr_log_path)
        },
    };

    const module = await WebAssembly.instantiate(buffer, imports);

    // get exports from module
    const main = module.instance.exports.main;

    // put the arguments into wasm memory
    const {argc, argv} = put_args_into_memory(args, memory);

    // run the program
    const exit_code = main(argc, argv);
    return exit_code;
};

// parse node cli arguments
const args = process.argv.slice(2); // first 2 args: ['node', '<filename>']
if (args.length < 1) {
    console.log("Please specify file to run");
} else {
    const filename = args[0];
    const exit_code = await run(filename, args);
    process.exit(exit_code);
}
