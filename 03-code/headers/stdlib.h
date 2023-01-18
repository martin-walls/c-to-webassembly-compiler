#ifndef _STDLIB_H
#define _STDLIB_H

#define NULL ((void*)0)

#define EXIT_SUCCESS 0

long strtol(const char *str, char **endptr, int base);

unsigned long strtoul(const char *str, char **endptr, int base);

#endif
