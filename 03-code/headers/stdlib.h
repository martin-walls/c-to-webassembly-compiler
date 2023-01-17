#ifndef _STDLIB_H
#define _STDLIB_H

#define NULL ((void*)0)

#define EXIT_SUCCESS 0

int atoi(const char *str);

unsigned long strtoul(const char *nptr, char **endptr, int base);

#endif
