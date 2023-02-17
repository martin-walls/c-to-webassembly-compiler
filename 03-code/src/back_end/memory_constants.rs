pub const PTR_SIZE: u32 = 4;

// 0----4---------8----12---
// | FP | temp FP | SP | ...
// -------------------------
pub const FRAME_PTR_ADDR: u32 = 0;
pub const TEMP_FRAME_PTR_ADDR: u32 = FRAME_PTR_ADDR + PTR_SIZE;
pub const STACK_PTR_ADDR: u32 = TEMP_FRAME_PTR_ADDR + PTR_SIZE;
