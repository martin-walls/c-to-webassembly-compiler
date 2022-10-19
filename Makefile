# wasm targets that should be generated from wat files
.PHONY: wat
wat: 01-research/50-main.wasm 01-research/51-sum.wasm 01-research/52-printf.wasm 01-research/53-module-deconstr.wasm

# compile wat file to wasm
%.wasm: %.wat
	wat2wasm $< -o $@