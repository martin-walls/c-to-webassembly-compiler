import {read_stack_ptr} from "./memory_operations.mjs";
import {appendFileSync} from "fs";
import {basename, dirname, join} from "path";
import {fileURLToPath} from "url";

function get_log_dir_path() {
    // the dir that this file is in
    const runtime_dir = dirname(fileURLToPath(import.meta.url));
    // relative to root code dir (03-code/)
    const log_output_dir = "logs";

    return join(runtime_dir, "..", log_output_dir);
}

export function init_stack_ptr_log_file(source_filepath) {
    // put timestamp in log file name, so we don't overwrite previous logs
    const log_name = `${basename(source_filepath)}.${Date.now()}.stackptrlog`;
    return join(get_log_dir_path(), log_name);
}

export function log_stack_ptr(wasm_memory, log_file_path) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        const stack_ptr = read_stack_ptr(memory);

        const data = `${stack_ptr}\n`;

        try {
            appendFileSync(log_file_path, data);
        } catch (err) {
            console.log(`Error logging stack pointer: ${err}`);
        }
    }
}
