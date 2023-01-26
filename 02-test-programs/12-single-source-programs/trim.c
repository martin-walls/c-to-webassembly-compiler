#include "trim.h"
#include <ctype.h>
#include <string.h>
#include <stdio.h>

char *
trim(char *str)
{
    char *end;

    // ltrim
    while (isspace(*str)) {
        str++;
    }

    if (*str == 0) // only spaces
        return str;

    // rtrim
    end = str + strlen(str) - 1;
    while (end > str && isspace(*end)) {
        end--;
    }

    // null terminator
    *(end+1) = 0;

    return str;
}

int main(int argc, char *argv[]) {
    char str[] = "    hello world  ";
    char *trimmed = trim(str);
    printf("%s\n", trimmed);
    return 0;
}
