#include <stdio.h>
#include <stdlib.h>

long sum(long n, long acc) {
    if (n == 0) {
        return acc;
    }
    return sum(n - 1, acc + n);
}

int main(int argc, char *argv[]) {
    long x = 1000000;
    if (argc >= 2) {
        x = strtol(argv[1], NULL, 10);
    }
    long n = sum(x, 0);
    printf("%ld\n", n);
    return 0;
}
