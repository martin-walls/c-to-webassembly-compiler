int main(int argc, char* argv) {
    int i = sizeof(int);
    int l = sizeof(long);
    int x = sizeof(i + l);
    return x;
}
