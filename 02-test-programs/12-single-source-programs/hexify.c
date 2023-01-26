/* hexify.c
 * See https://github.com/pepaslabs/hexify.c
 * Copyright (C) 2015 Jason Pepas.
 * Released under the terms of the MIT license.
 * See https://opensource.org/licenses/MIT
 */

#include "hexify.h"
#include <stdlib.h>
#include <stdio.h>

int hexify(unsigned char *in, size_t in_size, char *out, size_t out_size)
{
    // originally inspired by http://stackoverflow.com/a/12839870/558735

    if (in_size == 0 || out_size == 0) return 0;

    char map[16+1] = "0123456789abcdef";

    int bytes_written = 0;
    size_t i = 0;
    while(i < in_size && (i*2 + (2+1)) <= out_size)
    {
        unsigned char high_nibble = (in[i] & 0xF0) >> 4;
        *out = map[high_nibble];
        out++;

        unsigned char low_nibble = in[i] & 0x0F;
        *out = map[low_nibble];
        out++;

        bytes_written += 2;
        i++;
    }
    *out = '\0';

    return bytes_written;
}


int main(int argc, char *argv[]) {
    // pack a binary array
    unsigned char binary[3];
    binary[0] = 0xde;
    binary[1] = 0xad;
    binary[2] = 0xbe;

    printf("size: %d\n", sizeof(binary));

    // convert it into a hex string
    char hex[6 + 1];
    printf("size: %d\n", sizeof(hex));
    int bytes_written = hexify(binary, sizeof(binary), hex, sizeof(hex));

    // print the result
    printf("result: %s\nbytes written: %d", hex, bytes_written);

    return EXIT_SUCCESS;
}
