#include <stdlib.h>
#include <stdio.h>
#include <stddef.h>

void test_strtol() {
    long x = strtol("3", NULL, 10);
    printf("\"3\": %ld\n", x);

    long z = strtol("-120", NULL, 10);
    printf("\"-120\": %ld\n", z);

    char *str = "   56abc";
    char *endptr = NULL;
    long y = strtol(str, &endptr, 10);
    printf("\"   56abc\": %ld\n", y);
    ptrdiff_t offset = endptr - str;
    printf("endptr offset: %d\n", offset);
}

void test_strtoul() {
    unsigned long x = strtoul("3", NULL, 10);
    printf("\"3\": %lu\n", x);

    unsigned long z = strtoul("-120", NULL, 10);
    printf("\"-120\": %lu\n", z);

    char *str = "   56abc";
    char *endptr = NULL;
    unsigned long y = strtoul(str, &endptr, 10);
    printf("\"   56abc\": %lu\n", y);
    ptrdiff_t offset = endptr - str;
    printf("endptr offset: %d\n", offset);
}

int main(int argc, char *argv) {
    test_strtol();
    test_strtoul();

    return 0;
}
