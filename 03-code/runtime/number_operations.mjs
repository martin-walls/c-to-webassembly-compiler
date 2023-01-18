import {MAX_I16, MAX_I32, MAX_I64} from "./memory_constants.mjs";

export function signed_to_unsigned_i16(value) {
    if (value < 0) {
        value += MAX_I16;
    }
    return value;
}

export function signed_to_unsigned_i32(value) {
    if (value < 0) {
        value += MAX_I32;
    }
    return value;
}

export function signed_to_unsigned_i64(value) {
    if (value < 0) {
        value += MAX_I64;
    }
    return value;
}
