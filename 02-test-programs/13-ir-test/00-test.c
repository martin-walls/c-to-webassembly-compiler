#define FOO 1

int add(int x, int y) {
    int z = 1;
    label1:
    return x + y - z;
}

double quadratic(double a, double b, double c, double x) {
    return add(a * x * x + b * x, c);
}

int main(int argc, int argv) {
    return FOO ? quadratic(1, 2, 1, 5) : add(1, 2);
}
