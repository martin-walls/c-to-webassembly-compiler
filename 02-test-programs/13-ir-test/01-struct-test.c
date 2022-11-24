#define FOO 1

struct Complex {
    double real;
    double imaginary;
    int magnitude;
} complex;


int main(int argc, int argv) {
    struct Complex* cptr = &complex;
    int m = cptr->magnitude;
    return m;
}
