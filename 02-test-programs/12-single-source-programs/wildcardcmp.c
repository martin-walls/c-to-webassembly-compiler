#include <stdlib.h>
#include <stdio.h>
#include "wildcardcmp.h"

int
wildcardcmp(const char *pattern, const char *string) {
    const char *w = NULL; // last `*`
    const char *s = NULL; // last checked char

    // malformed
    if (!pattern || !string) return 0;

    // loop 1 char at a time
    while (1) {
        if (!*string) {
            if (!*pattern) return 1;
            if ('*' == *pattern) return 1;
            if (!*s) return 0;
            string = s++;
            pattern = w;
            continue;
        } else {
            if (*pattern != *string) {
                if ('*' == *pattern) {
                    w = ++pattern;
                    s = string;
                    // "*" -> "foobar"
                    if (*pattern) continue;
                    return 1;
                } else if (w) {
                    string++;
                    // "*ooba*" -> "foobar"
                    continue;
                }
                return 0;
            }
        }

        string++;
        pattern++;
    }

    return 1;
}

/*
 * Expected output: 1111111110000
 */
int main(int argc, char* argv) {
    // should return 1
    printf("%d", wildcardcmp("foo*", "foo"));
    printf("%d", wildcardcmp("foobar", "foobar"));
    printf("%d", wildcardcmp("*", "foobar"));
    printf("%d", wildcardcmp("foo*", "foobar"));
    printf("%d", wildcardcmp("fo*bar", "foobar"));
    printf("%d", wildcardcmp("*bar", "foobar"));
    printf("%d", wildcardcmp("f*b*r", "foobar"));
    printf("%d", wildcardcmp("f**b*r", "foobar"));
    printf("%d", wildcardcmp("f*", "foobar"));
    // negative - should return 0
    printf("%d", wildcardcmp("FOOBAR", "foobar"));
    printf("%d", wildcardcmp("foo", "foobar"));
    printf("%d", wildcardcmp("bar*", "foobar"));
    printf("%d\n", wildcardcmp("f*R", "foobar"));
    return 0;
}
