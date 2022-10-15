;; int main(int argc, int* argv[])
;; prints out all arguments suppied, in order

(module
  (import "console" "log" (func (param i32))) ;; funcidx 0
  (func (param i32) (param i32) (result i32)  ;; funcidx 1
    (block
      (loop
        ;; check if argument count is 0. if so, break out of the block
        local.get 0
        i32.eqz
        br_if 1

        ;; load the i32 at position $argv in memory
        local.get 1
        i32.load
        call 0

        ;; decrement the argument count (to represent the count of remaining arguments)
        local.get 0
        i32.const 1
        i32.sub
        local.set 0

        ;; increase the argument pointer by 4 to point at the next i32 location in memory
        local.get 1
        i32.const 4
        i32.add
        local.set 1

        ;; jump back to top of loop
        br 0
      )
    )

    i32.const 0
  )
  (memory 1) ;; create a linear memory, 1 page in size
  (export "main" (func 1)) ;; export the main function with name "main"
  (export "memory" (memory 0)) ;; export the memory with name "memory"
)

;; (module
;;   (import "console" "log" (func $log (param i32)))
;;   (func $main (param $argc i32) (param $argv i32) (result i32)
;;     (block
;;       (loop
;;         ;; check if argument count is 0. if so, break out of the block
;;         local.get $argc
;;         i32.eqz
;;         br_if 1

;;         ;; load the i32 at position $argv in memory
;;         local.get $argv
;;         i32.load
;;         call $log

;;         ;; decrement the argument count (to represent the count of remaining arguments)
;;         local.get $argc
;;         i32.const 1
;;         i32.sub
;;         local.set $argc

;;         ;; increase the argument pointer by 4 to point at the next i32 location in memory
;;         local.get $argv
;;         i32.const 4
;;         i32.add
;;         local.set $argv

;;         ;; jump back to top of loop
;;         br 0
;;       )
;;     )

;;     i32.const 0
;;   )
;;   (memory 1) ;; create a linear memory, 1 page in size
;;   (export "main" (func $main)) ;; export the main function with name "main"
;;   (export "memory" (memory 0)) ;; export the memory with name "memory"
;; )
