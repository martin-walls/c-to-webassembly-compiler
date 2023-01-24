#include <stdio.h>

int x = 5;

void foo() {
    printf("x: %d\n", x);
}

int main(int argc, char *argv) {
    printf("x: %d\n", x);
    foo();
    return 0;
}
