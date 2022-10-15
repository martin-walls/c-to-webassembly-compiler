(module
  (import "console" "printf" (func $printf (param i32)))
  (func $main (param $argc i32) (param $argv i32) (result i32)
    call $abc
    i32.const 0
  )
  ;; print string that was initialised from data segment
  (func $helloworld
    i32.const 0
    call $printf
  )
  ;; write characters a-z to memory then print them
  (func $abc
    (local i32) (local i32)
    ;; memory location to start string at
    i32.const 20
    local.set 0

    ;; first character
    i32.const 97
    local.set 1

    (loop
      ;; store character
      local.get 0
      local.get 1
      i32.store8  ;; store only a single byte, not a full i32

      ;; increment memory pointer
      local.get 0
      i32.const 1
      i32.add
      local.set 0

      ;; increment character
      local.get 1
      i32.const 1
      i32.add
      local.set 1

      ;; check if we've got to z
      local.get 1
      i32.const 123
      i32.lt_s
      br_if 0
    )

    ;; null-terminate string
    local.get 0
    i32.const 0
    i32.store8

    ;; print string
    i32.const 20
    call $printf    
  )
  (memory 1)
  (data (i32.const 0) "Hello world") ;; write string into memory
  (export "memory" (memory 0))
  (export "main" (func $main))
)