#include <stdio.h>

int main(int argc, char* argv) {
    printf("hello world\n");
    printf("int: %d\n", -3);
    printf("unsigned int: %u, long: %ld\n", -7, 10000000);
    short x = 20;
    long y = -100;
    printf("short: %hd, unsigned long: %lu\n", x, y);
    return 0;
}
