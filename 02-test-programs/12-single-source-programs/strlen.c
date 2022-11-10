#include "strlen.h"
#include <stdint.h>
size_t
strlen(const char *str)
{
    if (!str) {
        return 0;
    }

    const char *ptr = str;
    while (*str) {
        ++str;
    }

    return str - ptr;
}