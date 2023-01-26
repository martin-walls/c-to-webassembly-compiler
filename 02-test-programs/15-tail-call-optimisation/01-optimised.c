#include <stdio.h>

long sum(long n) {
    long acc = 0;
    if (n == 0) {
        return acc;
    }
    while (n > 0) {
        acc += n;
        n--;
    }
    return acc;
}

int main(int argc, char *argv[]) {
    long n = sum(100000);
    printf("%ld\n", n);
    return 0;
}
