#include <stdio.h>

int main(int argc, char *argv) {
    double x = 3.1;
    double y = 5.7;
    double z = x + y;
    printf("assert correct: %d", z == 8.8);
    return 0;
}
