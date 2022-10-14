import { readFileSync } from "fs";

const run = async (filename, args) => {
  const buffer = readFileSync(filename);

  // functions that will be passed in to wasm
  const imports = {
    console: {
      log: (arg) => console.log(arg),
    },
  };

  const module = await WebAssembly.instantiate(buffer, imports);

  // run 'main' function with command line arguments
  const { main, memory } = module.instance.exports;
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
