#include <stdio.h>
#include <stdlib.h>

/**
 * Returns the nth Fibonacci number.
 */
int fib(int n) {
  if (n <= 0) return 0;
  if (n == 1) return 1;
  return fib(n-1) + fib(n-2);
}

int main(int argc, char *argv[]) {
    if (argc < 1 + 1) {
        for (int i = 0; i < 15; i++) {
            printf("%d: %d\n", i, fib(i));
        }
    } else {
        int n = (int) strtol(argv[1], NULL, 10);
        int f = fib(n);
        printf("fib(%d) = %d\n", n, f);
    }

    return 0;
}
