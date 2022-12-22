int main(int argc, char* argv) {
    label1:
    int x = 3;
    label2:
    int *px = &x;
    label3:
    *px += 1;
    label4:
    return x;
}
