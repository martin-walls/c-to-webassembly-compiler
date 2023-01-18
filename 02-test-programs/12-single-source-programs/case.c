//
// case.c
//
// Copyright (c) 2014 Stephen Mathieson
// MIT licensed
//

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "case.h"

#define CASE_IS_SEP(c)    ((c) == '-' || (c) == '_' || (c) == ' ')

char *
case_upper(char *str) {
    for (char *s = str; *s; s++) {
        *s = toupper(*s);
    }
    return str;
}

char *
case_lower(char *str) {
    for (char *s = str; *s; s++) {
        *s = tolower(*s);
        printf("%s\n", s);
    }
    return str;
}

char *
case_camel(char *str) {
    char *r = str, *w = str;
    // never cap the first char
    while (CASE_IS_SEP(*r)) {
        r++;
    }
    while (*r && !CASE_IS_SEP(*r)) {
        *w = *r;
        w++;
        r++;
    }
    while (*r) {
        do {
            r++;
        } while (CASE_IS_SEP(*r));
        *w = toupper(*r);
        w++;
        r++;
        while (*r && !CASE_IS_SEP(*r)) {
            *w = *r;
            w++;
            r++;
        }
    }
    *w = 0;
    return str;
}

int main(int argc, char *argv) {
    char str1[] = "hEllOWoRlD";
    char *upper = case_upper(str1);
    printf("upper: %s\n", upper);

    char str2[] = "hEllOWoRlD";
    char *lower = case_lower(str2);
    printf("lower: %s\n", lower);

    char str3[] = "hello world";
    char *camel = case_camel(str3);
    printf("camel: %s\n", camel);

    return 0;
}
