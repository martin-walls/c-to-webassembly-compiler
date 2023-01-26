#include <stdio.h>

/**
 * Returns the nth Fibonacci number.
 */
int fib(int n) {
  if (n <= 0) return 0;
  if (n == 1) return 1;
  return fib(n-1) + fib(n-2);
}

int main(int argc, char *argv[]) {
    for (int i = 0; i < 15; i++) {
        printf("%d: %d\n", i, fib(i));
    }

    return 0;
}
