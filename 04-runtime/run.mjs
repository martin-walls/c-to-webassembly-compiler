//
// node 20-wasm-run.mjs <wasm_filename> [args...]
//
import {readFileSync} from "fs";
import {printf} from "./wasm_stdlib.mjs";
import {put_args_into_memory} from "./init_memory.mjs";

const IMPORTS_MODULE_NAME = "wasm_stdlib";

const run = async (filename, args) => {
    const buffer = readFileSync(filename);

    let memory;

    // functions that will be passed in to wasm
    const imports = {
        IMPORTS_MODULE_NAME: {
            log: (arg) => {
                console.log(arg);
                return arg;
            },
            printf: printf(memory),
        },
    };

    const module = await WebAssembly.instantiate(buffer, imports);

    // get exports from module
    const main = module.instance.exports.main;
    memory = module.instance.exports.memory;


    // put the arguments into wasm memory
    const {argc, argv} = put_args_into_memory(args, memory);

    // run the program
    const result = main(argc, argv);
    console.log(`result: ${result}`);
};

// parse node cli arguments
const args = process.argv.slice(2); // first 2 args: ['node', '<filename>']
if (args.length < 1) {
    console.log("Please specify file to run");
} else {
    const filename = args[0];
    run(filename, args);
}
