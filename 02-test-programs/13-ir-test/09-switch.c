#include <stdio.h>

int main(int argc, char *argv[]) {
    for (int x = 1; x < 7; x++) {
        switch (x) {
            case 1:
            case 2:
                printf("less than 3\n");
                break;
            case 3:
                printf("equal to 3\n");
                break;
            case 4:
                printf("equal to 4\n");
            default:
            case 6:
                printf("greater than 3\n");
                break;
        }
    }
    return 0;
}
