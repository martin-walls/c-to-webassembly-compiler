// this runs out of memory without tail-call optimisation

#include <stdio.h>

long sum(long n) {
    if (n == 0) {
        return 0;
    }
    return n + sum(n - 1);
}

int main(int argc, char *argv[]) {
    long n = sum(100000);
    printf("%ld\n", n);
    return 0;
}
