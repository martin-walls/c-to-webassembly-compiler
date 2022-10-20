# Handwritten Fibonacci program in Wasm

This file is written in literate binary. The binary file is produced from all the code blocks appended together. The code blocks should contain just hex strings and comments (starting with `#`). Everything else is ignored, and is just used for explanation.

It's turned into a binary file using the [lb](https://github.com/marhop/literate-binary) tool.

## Module preamble

    0061 736d # magic number
    0100 0000 # wasm version

## Types section

    01 # section code
    12 # section size
    03 # num types

0: (param i32) (param i32) (param i32)

    60 # code for function type
    03 # num params
    7f # i32
    7f
    7f
    00 # num results

1: (param i32) (result i32)

    60
    01
    7f
    01
    7f

2: (param i32) (param i32) (result i32)

    60
    02
    7f
    7f
    01
    7f

## Imports section

    02 # section code
    12 # section size
    01 # num imports

(import "console" "printf")

    07 # string length
    "console"
    06 # string length
    "printf"
    00 # this is a function import
    00 # imported function has type 0

## Functions section

    03 # section code
    03 # section size

    02 # num functions
    01 # $fib type index
    02 # $main type index

## Memory section

    05 # section code
    03 # section size
    01 # num memories

    00 # flags: no maximum size given
    01 # minimum number of pages

## Exports section

    07 # section code
    11 # section size
    02 # num exports

    04 # string length
    "main"
    00 # this is a function export
    02 # export function index 1

    06
    "memory"
    02 # this is a memory export
    00 # export memory 0

## Code section

    0a # section code
    5c # section size
    02 # num functions

fib function

    36 # function body size
    01 # num local declarations
    01
    7f # i32

    02 # block
    40 # empty blocktype
    20 # local.get
    00
    41 # i32.const
    00
    4c # i32.le_s

    04 # if
    40 # empty blocktype
    41 # i32.const
    00
    21 # local.set
    01
    0c # br
    01
    0b # end

    20 # local.get
    00
    41 # i32.const
    01
    46 # i32.eq

    04 # if
    40 # empty blocktype
    41 # i32.const
    01
    21 # local.set
    01
    0c # br
    01
    0b # end

    20 # local.get
    00
    41 # i32.const
    01
    6b # i32.sub
    10 # call
    01

    20 # local.get
    00
    41 # i32.const
    02
    6b # i32.sub
    10 # call
    01

    6a # i32.add
    21 # local.set
    01

    0b # end
    20 # local.get
    01
    0b # end function expression

main function

    23 # function body size
    01 # num local declarations
    01
    7f

    41 # i32.const
    00
    21 # local.set
    02

    03 # loop
    40 # empty blocktype
    41 # i32.const
    10
    20 # local.get
    02
    20 # local.get
    02
    10 # call
    01 # fib function index

    10 # call
    00 # printf function index

    20 # local.get
    02
    41 # i32.const
    01
    6a # i32.add
    22 # local.tee
    02
    41 # i32.const
    0f
    48 # i32.lt_s
    0d # br_if
    00

    0b # end
    41 # i32.const
    00

    0b # end

## Data section

    0b # section code
    0d # section size
    01 # num data segments

    00 # active data segment, referencing memory 0
    # initialiser expression
    41 # i32.const
    10
    0b # end
    07 # data segment size
    "%d: %d" 0a
