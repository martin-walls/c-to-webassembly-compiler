# wasm targets that should be generated from wat files
.PHONY: wat
wat: 50-main.wasm 51-sum.wasm 52-printf.wasm 53-module-deconstr.wasm

# compile wat file to wasm
%.wasm: %.wat
	wat2wasm $< -o $@

all: wat