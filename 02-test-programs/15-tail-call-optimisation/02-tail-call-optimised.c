#include <stdio.h>

long sum(long n, long acc) {
    if (n == 0) {
        return acc;
    }
    return sum(n - 1, acc + n);
}

int main(int argc, char *argv) {
    long n = sum(1000, 0);
    printf("%ld\n", n);
    return 0;
}
