int a1[] = { 0, 1, 2, 3, 4};

char a2[5] = { 'a', 'b', 'c', 'd', 'e'};

short a3[][] = { { 0, 1, 2, 3 },
                 { 4, 5, 6, 7 },
                 { 8, 9, 10, 11 } };

int a4[2][] = { {1, 2, 3},
                {4, 5, 6} };

char s1[] = "hello world";

char s2[] = { "goodbye world" };

struct st {
    int x;
    int y;
};

struct st a5[] = { {1, 2},
            {3, 4},
            {5, 6} };

struct st1 {
    int x;
    char y[3];
    struct st2 {double a; int c;} z;
} s = {1, "ab", {1.1, 2} };

union U {
    int x;
    double y;
};

union U u1 = 42;
