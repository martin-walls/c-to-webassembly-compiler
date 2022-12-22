int main(int argc, char* argv) {
    int x = 3;
    int *px = &x;
    *px += 1;
    return *px;
}
