//
// occurrences.c
//
// Copyright (c) 2013 Stephen Mathieson
// MIT licensed
//

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "occurrences.h"

/*
 * Get the number of occurrences of `needle` in `haystack`
 */

size_t
occurrences(char *needle, char *haystack) {
    if (NULL == needle || NULL == haystack) return -1;

    char *pos = (char *) haystack;
    size_t i = 0;
    size_t l = strlen(needle);
    if (l == 0) return 0;

    while ((pos = strstr(pos, needle))) {
        pos += l;
        i++;
    }

    return i;
}

int main(int argc, char *argv[]) {
    char *text = "panamabananas";
    char *pattern = "ana";
    int count = occurrences(pattern, text);
    printf("occurrences: %d\n", count);
    return 0;
}
