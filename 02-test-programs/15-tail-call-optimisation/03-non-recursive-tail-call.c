#include <stdio.h>

int bar(int n);

int foo(int n) {
    if (n <= 0) {
        return n;
    }
    return bar(n - 2);
}

int bar(int n) {
    return foo(n - 1);
}

int main(int argc, char *argv) {
    int x = 20;
    int fx = foo(x);
    printf("foo(%d) = %d\n", x, fx);
    int y = 21;
    int fy = foo(y);
    printf("foo(%d) = %d\n", y, fy);
    return 0;
}
