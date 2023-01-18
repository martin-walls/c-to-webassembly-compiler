import {FRAME_PTR_ADDR, PTR_SIZE, STACK_PTR_ADDR} from "./memory_constants.mjs";

export function read_frame_ptr(memory) {
  return read_ptr(FRAME_PTR_ADDR, memory);
}

export function read_stack_ptr(memory) {
  return read_ptr(STACK_PTR_ADDR, memory);
}

export function store_stack_ptr(stack_ptr, memory) {
  store_ptr(STACK_PTR_ADDR, stack_ptr, memory);
}

export function read_ptr(addr, memory) {
  return Number(read_int(addr, PTR_SIZE, memory));
}

export function store_ptr(address, ptr_value, memory) {
  store_int(address, PTR_SIZE, BigInt(ptr_value), memory);
}

// export function read_i32(addr, memory) {
//   // read bytes of int in little-endian order
//   let value = memory[addr];
//   value |= memory[addr + 1] << 8;
//   value |= memory[addr + 2] << 16;
//   value |= memory[addr + 3] << 24;
//   return value;
// }

// export function store_i32(address, value, memory) {
//   memory[address] = value & 0xff;
//   memory[address + 1] = (value >> 8) & 0xff;
//   memory[address + 2] = (value >> 16) & 0xff;
//   memory[address + 3] = (value >> 24) & 0xff;
// }

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
  memory[addr] = Number(value & 0xffn);
  for (let i = 1; i < byte_size; i++) {
    memory[addr + i] = Number((value >> BigInt(8 * i)) & 0xffn);
  }
}

export function read_string(addr, memory) {
  // read a null-terminated string starting at addr from memory
  const next_char = () => {
    const byte = memory[addr];
    addr++;
    return String.fromCharCode(byte);
  }

  let str = "";
  let c = next_char();
  while (c !== "\0") {
    str += c;
    c = next_char();
  }
  return str;
}