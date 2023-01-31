#include <stdio.h>

int a1[] = {0, 1, 2, 3, 4};

char a2[5] = {'a', 'b', 'c', 'd', 'e'};

short a3[][4] = {{0, 1, 2,  3},
                 {4, 5, 6,  7},
                 {8, 9, 10, 11}};

int a4[][3] = {{1, 2, 3},
               {4, 5, 6}};

char s1[] = "hello world";

char s2[] = {"goodbye world" };

struct st {
    int x;
    int y;
};

struct st a5[] = {{1, 2},
                  {3, 4},
                  {5, 6} };

struct st1 {
    int x;
    char y[3];
    struct st2 {
        double a;
        int c;
    } z;
} s = {1,
       "ab",
       {1.1, 2}};

union U {
    int x;
    double y;
};

int main(int argc, char *argv[]) {
    printf("a1[1] = %d\n", a1[1]);
//    printf("a3[1][2] = %hd\n", a3[1][2]);
//    printf("a5[1].x = %d\n", a5[1].x);
//    printf("s.y[1] = %c\n", s.y[1]);
    return 0;
}

//union U u1 = 42;
