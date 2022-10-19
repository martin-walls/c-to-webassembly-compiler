(module
  (import "console" "log" (func $log (param i32)))
  (func $main (param i32) (param i32)
    i32.const 10
    global.get $foo
    ;; call function at index 0 in table ($add)
    i32.const 0
    call_indirect (type $addtype)
    call $log
  )
  (type $addtype (func (param i32) (param i32) (result i32)))
  (func $add (type $addtype) (param i32) (param i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func $startfunc
    i32.const 4
    global.set $foo
  )
  (start $startfunc)
  (global $foo (mut i32) (i32.const 42))
  (memory 1)
  (data (i32.const 0) "Hello world")
  (table $table 16 funcref)
  (elem (i32.const 0) $add)
  (export "main" (func $main))
  (export "memory" (memory 0))
)