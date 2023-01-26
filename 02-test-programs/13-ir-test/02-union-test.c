#define FOO 1

union BigOrSmall {
    char x;
    long y;
} u;


int main(int argc, char *argv[]) {
    return u.x;
}
