(module
  (import "console" "printf" (func $printf_i32_i32 (param i32) (param i32) (param i32)))
  (func $fib (param i32) (result i32)
    (local i32) ;; return value
    (block
      ;; if n <= 0, return 0
      local.get 0
      i32.const 0
      i32.le_s
      if
        i32.const 0
        local.set 1
        br 1
      end

      ;; if n == 1, return 1
      local.get 0
      i32.const 1
      i32.eq
      if
        i32.const 1
        local.set 1
        br 1
      end

      ;; fib(n-1)
      local.get 0
      i32.const 1
      i32.sub
      call $fib

      ;; fib(n-2)
      local.get 0
      i32.const 2
      i32.sub
      call $fib

      i32.add
      local.set 1
    )
    local.get 1 ;; return value
  )
  (func $main (param i32) (param i32) (result i32)
    (local i32) ;; loop counter
    i32.const 0
    local.set 2
    (loop
      i32.const 16 ;; mem address of print format
      local.get 2  ;; loop counter to print
      local.get 2  ;; calculate fib from loop counter
      call $fib

      call $printf_i32_i32

      ;; increment loop counter
      local.get 2
      i32.const 1
      i32.add
      local.tee 2
      ;; loop until i == 15
      i32.const 15
      i32.lt_s
      br_if 0
    )
    i32.const 0
  )
  (memory 1)
  (data (i32.const 16) "%d: %d\n") ;; printf format string
  (export "main" (func $main))
  (export "memory" (memory 0))
)