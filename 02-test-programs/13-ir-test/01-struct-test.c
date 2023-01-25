#include <stdio.h>

struct Complex {
    int real;
    long imaginary;
} complex = {5, 7};

struct Complex complex2;

struct A {
    long a;
    long b;
    long c;
    long d;
} a = {2, 2, 2, 2,};


int main(int argc, int argv) {
    printf("real: %d, imaginary: %ld\n", complex.real, complex.imaginary);

    complex2.real = 12;
    complex2.imaginary = -5;

    printf("real: %d, imaginary: %ld\n", complex2.real, complex2.imaginary);

    struct Complex *c2ptr = &complex2;
    c2ptr->real++;
    c2ptr->imaginary *= 2;
    ++(c2ptr->imaginary);

    printf("real: %d, imaginary: %ld\n", c2ptr->real, c2ptr->imaginary);

    a.a++;
    a.b--;
    ++(a.c);
    --(a.d);
    printf("a: %ld, b: %ld, c: %ld, d: %ld\n", a.a, a.b, a.c, a.d);

    struct A *ap = &a;
    ap->a++;
    ap->b--;
    ++(ap->c);
    --(ap->d);
    printf("a: %ld, b: %ld, c: %ld, d: %ld\n", ap->a, ap->b, ap->c, ap->d);

    return 0;
}
