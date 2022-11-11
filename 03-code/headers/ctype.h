#ifndef _CTYPE_H
#define _CTYPE_H

int isalnum(int c);
int isalpha(int c);
int iscntrl(int c);
int isdigit(int c);
int isxdigit(int c);
int isgraph(int c);
int ispunct(int c);
int isprint(int c);
int islower(int c);
int isupper(int c);
int isspace(int c);
int tolower(int c);
int toupper(int c);

int isalnum(int c) {
    return isalpha(c) || isdigit(c);
}

int isalpha(int c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
}

int iscntrl(int c) {
    return (c >= 0 && c <= 31) || c == 127;
}

int isdigit(int c) {
    return (c >= '0' && c <= '9');
}

int isxdigit(int c) {
    return isdigit(c) || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f');
}

int isgraph(int c) {
    return isprint(c) && (c != ' ');
}

int ispunct(int c) {
    return isgraph(c) && !isalnum(c);
}

int isprint(int c) {
    return !iscntrl(c);
}

int islower(int c) {
    return c >= 'a' && c <= 'z';
}

int isupper(int c) {
    return c >= 'A' && c <= 'Z';
}

int isspace(int c) {
    return c == '\t' || c == '\r' || c == '\n' || c == ' ';
}

int tolower(int c) {
    if (isupper(c)) {
        return c - 'A' + 'a';
    }
    return c;
}

int toupper(int c) {
    if (islower(c)) {
        return c - 'a' + 'A';
    }
    return c;
}

#endif