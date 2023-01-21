#ifndef _CTYPE_H
#define _CTYPE_H

int isalnum(char c);
int isalpha(char c);
int iscntrl(char c);
int isdigit(char c);
int isxdigit(char c);
int isgraph(char c);
int ispunct(char c);
int isprint(char c);
int islower(char c);
int isupper(char c);
int isspace(char c);
char tolower(char c);
char toupper(char c);

int isalnum(char c) {
    return isalpha(c) || isdigit(c);
}

int isalpha(char c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
}

int iscntrl(char c) {
    return (c >= 0 && c <= 31) || c == 127;
}

int isdigit(char c) {
    return (c >= '0' && c <= '9');
}

int isxdigit(char c) {
    return isdigit(c) || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f');
}

int isgraph(char c) {
    return isprint(c) && (c != ' ');
}

int ispunct(char c) {
    return isgraph(c) && !isalnum(c);
}

int isprint(char c) {
    return !iscntrl(c);
}

int islower(char c) {
    return c >= 'a' && c <= 'z';
}

int isupper(char c) {
    return c >= 'A' && c <= 'Z';
}

int isspace(char c) {
    return c == '\t' || c == '\r' || c == '\n' || c == ' ';
}

char tolower(char c) {
    if (isupper(c)) {
        return c - 'A' + 'a';
    }
    return c;
}

char toupper(char c) {
    if (islower(c)) {
        return c - 'a' + 'A';
    }
    return c;
}

#endif