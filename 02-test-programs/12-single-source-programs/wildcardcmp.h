#ifndef WILDCARDCMP_H
#define WILDCARDCMP_H 1

/**
 * Check if the given `string` matches `pattern`,
 * using `*` as a wildcard.  Will return `1` if
 * the two match.
 */

int
wildcardcmp(char *pattern, char *string);

#endif
