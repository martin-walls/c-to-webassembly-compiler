#define FOO 1

union BigOrSmall {
    char x;
    long y;
} u;


int main(int argc, int argv) {
    return u.x;
}
