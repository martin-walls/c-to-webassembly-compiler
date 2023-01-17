import {FRAME_PTR_ADDR, STACK_PTR_ADDR} from "./memory_constants.mjs";

export function read_frame_ptr(memory) {
  return read_ptr(FRAME_PTR_ADDR, memory);
}

export function read_stack_ptr(memory) {
  return read_ptr(STACK_PTR_ADDR, memory);
}

export function store_stack_ptr(stack_ptr, memory) {
  store_ptr(stack_ptr, STACK_PTR_ADDR, memory);
}

export function read_ptr(addr, memory) {
  return read_i32(addr, memory);
}

export function store_ptr(address, ptr_value, memory) {
  store_i32(address, ptr_value, memory);
}

export function read_i32(addr, memory) {
  // read bytes of int in little-endian order
  let value = memory[addr];
  value |= memory[addr + 1] << 8;
  value |= memory[addr + 2] << 16;
  value |= memory[addr + 3] << 24;
  return value;
}

export function store_i32(address, value, memory) {
  memory[address] = value & 0xff;
  memory[address + 1] = (value >> 8) & 0xff;
  memory[address + 2] = (value >> 16) & 0xff;
  memory[address + 3] = (value >> 24) & 0xff;
}

export function read_int(addr, byte_size, memory) {
  // read bytes of int in little-endian order
  // test the highest bit to see whether it's negative (BigInt won't
  //  automatically be negative if so)
  let value = 0n;
  let is_negative = 0;
  for (let i = 0; i < byte_size; i++) {
    const byte = memory[addr + i];
    value |= BigInt(byte) << BigInt(8 * i);
    is_negative = (byte >> 7) & 1;
  }
  if (is_negative) {
    value -= 1n << (BigInt(byte_size) * 8n);
  }
  return value;
}

export function store_int(addr, byte_size, value, memory) {
  // store bytes of int in little-endian order
  memory[addr] = value & 0xff;
  for (let i = 1; i < byte_size; i++) {
    memory[addr + i] = (value >> (8 * i)) & 0xff;
  }
}
