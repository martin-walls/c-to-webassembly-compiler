#!/usr/bin/env node
//
// usage: run.mjs <wasm_filename> [args...]
//
import {readFileSync} from "fs";
import {printf} from "./stdlib/stdio.mjs";
import {put_args_into_memory} from "./init_memory.mjs";
import {atoi, strtoul} from "./stdlib/stdlib.mjs";

const run = async (filename, args) => {
    const buffer = readFileSync(filename);

    let memory = new WebAssembly.Memory({initial: 1});

    // functions that will be passed in to wasm
    const imports = {
        runtime: {
            memory: memory,
        },
        stdlib: {
            printf: printf(memory),
            strtoul: strtoul(memory),
            atoi: atoi(memory),
        },
    };

    const module = await WebAssembly.instantiate(buffer, imports);

    // get exports from module
    const main = module.instance.exports.main;
    // memory = module.instance.exports.memory;

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
