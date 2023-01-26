#include <stdio.h>

void a(int x) {
    int arr[x];

    for (int i = 0; i < x; i++) {
        arr[i] = i;
    }

    for (int i = 0; i < x; i++) {
        printf("arr[%d] = %d\n", i, arr[i]);
    }
}

void b() {
    int arr[] = {1, 2, 3, 4};

    for (int i = 0; i < 4; i++) {
        printf("arr[%d] = %d\n", i, arr[i]);
    }
}

int main(int argc, char *argv[]) {
    a(3);
    a(5);
    b();
    return 0;
}
