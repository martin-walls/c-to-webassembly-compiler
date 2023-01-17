#include <stdio.h>

int main(int argc, char *argv) {
    long x = 64;
    int y = 1;
    long z = x >> y;
    printf("%ld\n", z);
    return 0;
}
