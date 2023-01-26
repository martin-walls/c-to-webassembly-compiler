#include <stdio.h>
#include <stdlib.h>

long gcd(long a, long b) {
    if (b == 0) {
        return a;
    }
    return gcd(b, a % b);
}

int main(int argc, char *argv[]) {
    long x;
    long y;
    if (argc < 2 + 1) {
        x = 31487;
        y = 21933;
    } else {
        x = strtol(argv[1], NULL, 10);
        y = strtol(argv[2], NULL, 10);
    }

    long result = gcd(x, y);
    printf("gcd(%ld, %ld) = %ld\n", x, y, result);
    return 0;
}
