#include "strlen.h"
#include <stdint.h>
#include <stdio.h>

size_t
strlen(const char *str) {
    if (!str) {
        return 0;
    }

    const char *ptr = str;
    while (*str) {
        ++str;
    }

    return str - ptr;
}

int main(int argc, char *argv[]) {
    char *str1 = "hello world";
    size_t len = strlen(str1);
    printf("len: %d\n", len);
    return 0;
}
