00-fibonacci: 00-fibonacci.c
	gcc -o 00-fibonacci 00-fibonacci.c

01-fibonacci.wasm: 01-fibonacci.wat
	wat2wasm 01-fibonacci.wat -o 01-fibonacci.wasm

03-game-of-life: 03-game-of-life.c
	gcc -o 03-game-of-life 03-game-of-life.c

gameoflife-blinker: 03-game-of-life
	./03-game-of-life 5 5 0000 00100 00100 00100 00000

gameoflife-pulsar: 03-game-of-life
	./03-game-of-life 15 15 000000000000000 \
													000111000111000 \
													000000000000000 \
													010000101000010 \
													010000101000010 \
													010000101000010 \
													000111000111000 \
													000000000000000 \
													000111000111000 \
													010000101000010 \
													010000101000010 \
													010000101000010 \
													000000000000000 \
													000111000111000 \
													000000000000000