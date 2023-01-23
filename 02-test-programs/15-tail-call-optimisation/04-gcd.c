#include <stdio.h>

long gcd(long a, long b) {
    if (b == 0) {
        return a;
    }
    return gcd(b, a % b);
}

int main(int argc, char *argv) {
    long x = gcd(1071, 462);
    printf("%ld\n", x);
    x = gcd(31487, 21933);
    printf("%ld\n", x);
    return 0;
}
