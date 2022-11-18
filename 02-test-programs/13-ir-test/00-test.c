#define FOO 1

typedef unsigned long bar;

bar add(int x, bar y) {
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
