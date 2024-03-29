export const PTR_SIZE = 4;
export const I8_SIZE = 1;
export const I16_SIZE = 2;
export const I32_SIZE = 4;
export const I64_SIZE = 8;
export const F32_SIZE = 4;
export const F64_SIZE = 8;
export const SIZE_T_SIZE = I32_SIZE;

export const FRAME_PTR_ADDR = 0;
export const TEMP_FRAME_PTR_ADDR = FRAME_PTR_ADDR + PTR_SIZE;
export const STACK_PTR_ADDR = TEMP_FRAME_PTR_ADDR + PTR_SIZE;

export const MAX_I16 = 65_536n;
export const MAX_I32 = 4_294_967_296n;
export const MAX_I64 = 18_446_744_073_709_551_616n;

export const NULL = 0;