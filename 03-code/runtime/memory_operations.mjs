export const PTR_SIZE = 4;

const FRAME_PTR_ADDR = 0;
const TEMP_FRAME_PTR_ADDR = FRAME_PTR_ADDR + PTR_SIZE;
const STACK_PTR_ADDR = TEMP_FRAME_PTR_ADDR + PTR_SIZE;

export function read_frame_ptr(memory) {
    return read_ptr(FRAME_PTR_ADDR, memory);
}

export function read_stack_ptr(memory) {
    return read_ptr(STACK_PTR_ADDR, memory);
}

export function read_ptr(addr, memory) {
  return read_i32(addr, memory);
}

export function read_i32(addr, memory) {
  // read bytes of int in little-endian order
  let value = memory[addr];
  value |= memory[addr + 1] << 8;
  value |= memory[addr + 2] << 16;
  value |= memory[addr + 3] << 24;
  return value;
}

export function store_stack_ptr(stack_ptr, memory) {
  store_ptr(stack_ptr, STACK_PTR_ADDR, memory);
}

export function store_ptr(address, ptr_value, memory) {
  store_i32(address, ptr_value, memory);
}

export function store_i32(address, value, memory) {
  memory[address] = value & 0xFF;
  memory[address + 1] = (value >> 8) & 0xFF;
  memory[address + 2] = (value >> 16) & 0xFF;
  memory[address + 3] = (value >> 24) & 0xFF;
}
