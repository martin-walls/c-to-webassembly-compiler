//
// node 20-wasm-run.mjs <wasm_filename> [args...]
//
import { readFileSync } from "fs";

const run = async (filename, args) => {
  const buffer = readFileSync(filename);

  let memory;

  // print a null-terminated string from memory to console
  const printf = (offset, ...formatargs) => {
    let str = "";
    const mem = new Uint8Array(memory.buffer);
    let b = mem[offset];
    let c;
    let argcounter = 0;
    while (b !== 0) {
      c = String.fromCharCode(b);
      // handle format args
      if (c === "%") {
        offset++;
        b = mem[offset];
        c = String.fromCharCode(b);
        if (c !== "%" && argcounter < formatargs.length) {
          str += `${formatargs[argcounter]}`
          argcounter++;
          offset++;
          b = mem[offset];
          continue;
        }
      }
      str += c
      offset++;
      b = mem[offset];
    }

    process.stdout.write(str); // only works in node.js, use console.log otherwise, but that prints newline
  };

  // functions that will be passed in to wasm
  const imports = {
    console: {
      log: (arg) => {
        console.log(arg);
        return arg;
      },
      printf,
    },
  };

  const module = await WebAssembly.instantiate(buffer, imports);

  // run 'main' function with command line arguments
  const { main } = module.instance.exports;
  memory = module.instance.exports.memory;
  const argc = args.length;
  const argv = 0;
  // put the arguments into wasm memory
  const argvArray = new Int32Array(memory.buffer, argv, argc);
  argvArray.set(args);
  const result = main(argc, argv);
  console.log(`result: ${result}`);
};

// parse node cli arguments
const args = process.argv.slice(2); // first 2 args: ['node', '<filename>']
if (args.length < 1) {
  console.log("Please specify file to run");
} else {
  const filename = args[0];
  run(filename, args.slice(1));
}
