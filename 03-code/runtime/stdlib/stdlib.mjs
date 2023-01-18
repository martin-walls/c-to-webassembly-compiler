import {I32_SIZE, I64_SIZE, NULL, PTR_SIZE} from "../memory_constants.mjs";
import {read_frame_ptr, read_int, read_ptr, read_string, store_int, store_ptr} from "../memory_operations.mjs";
import {signed_to_unsigned_i64} from "../number_operations.mjs";

// long strtol(const char *str, char **endptr, int base);
export function strtol(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load params
        const fp = read_frame_ptr(memory);
        let param_ptr = fp + PTR_SIZE + I64_SIZE;
        const str_ptr = read_ptr(param_ptr, memory);
        param_ptr += PTR_SIZE;
        const end_ptr = read_ptr(param_ptr, memory);
        param_ptr += PTR_SIZE;
        const base = read_int(param_ptr, I32_SIZE, memory);

        const str = read_string(str_ptr, memory);

        const result = BigInt(parseInt(str, Number(base)));

        if (end_ptr != NULL) {
            let i = 0;
            // skip initial whitespace
            while (i < str.length && /\s/.test(str[i])) {
                i++;
            }
            // skip to end of number
            while (i < str.length && str[i] >= "0" && str[i] <= "9") {
                i++;
            }
            let end_ptr_value = str_ptr + i;
            store_ptr(end_ptr, end_ptr_value, memory);
        }

        // write return value
        store_int(fp + PTR_SIZE, I64_SIZE, result, memory);
    }
}

// unsigned long strtoul(const char *str, char **endptr, int base);
export function strtoul(wasm_memory) {
    return () => {
        const memory = new Uint8Array(wasm_memory.buffer);
        // load params
        const fp = read_frame_ptr(memory);
        let param_ptr = fp + PTR_SIZE + I64_SIZE;
        const str_ptr = read_ptr(param_ptr, memory);
        param_ptr += PTR_SIZE;
        const end_ptr = read_ptr(param_ptr, memory);
        param_ptr += PTR_SIZE;
        const base = read_int(param_ptr, I32_SIZE, memory);

        const str = read_string(str_ptr, memory);

        const result = parseInt(str, Number(base));
        const unsigned_result = signed_to_unsigned_i64(BigInt(result));

        if (end_ptr != NULL) {
            let i = 0;
            // skip initial whitespace
            while (i < str.length && /\s/.test(str[i])) {
                i++;
            }
            // skip to end of number
            while (i < str.length && str[i] >= "0" && str[i] <= "9") {
                i++;
            }
            let end_ptr_value = str_ptr + i;
            store_ptr(end_ptr, end_ptr_value, memory);
        }

        // write return value
        store_int(fp + PTR_SIZE, I64_SIZE, unsigned_result, memory);
    }
}
