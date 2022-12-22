// print a null-terminated string from memory to console
export const printf = (memory) => {
    return (offset, ...formatargs) => {
        let str = "";
        const mem = new Uint8Array(memory.buffer);
        let b = mem[offset];
        let c;
        let argcounter = 0;
        while (b !== 0) {
            c = String.fromCharCode(b);
            // handle format args
            if (c === "%") {
                offset++;
                b = mem[offset];
                c = String.fromCharCode(b);
                if (c !== "%" && argcounter < formatargs.length) {
                    str += `${formatargs[argcounter]}`;
                    argcounter++;
                    offset++;
                    b = mem[offset];
                    continue;
                }
            }
            str += c;
            offset++;
            b = mem[offset];
        }

        process.stdout.write(str); // only works in node.js, use console.log otherwise, but that prints newline
    };
};
