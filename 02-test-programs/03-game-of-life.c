#include <stdio.h>
#include <stdlib.h>

void nextGeneration(int xLen, int yLen, unsigned long grid[]) {
    unsigned long newGrid[yLen];

    for (int y = 0; y < yLen; y++) {
        newGrid[y] = 0;
        for (int x = xLen - 1; x >= 0; x--) {
            char cell = (grid[y] >> x) & 1;

            // count how many neighbours are alive
            int liveNeighbourCount = 0;
            if (x > 0) {
        liveNeighbourCount += (grid[y] >> (x - 1)) & 1;
      }
      if (x < xLen - 1) {
        liveNeighbourCount += (grid[y] >> (x + 1)) & 1;
      }
      // row above
      if (y > 0) {
        liveNeighbourCount += (grid[y-1] >> x) & 1;
        if (x > 0) {
          liveNeighbourCount += (grid[y-1] >> (x - 1)) & 1;
        }
        if (x < xLen - 1) {
          liveNeighbourCount += (grid[y-1] >> (x + 1)) & 1;
        }
      }
      // row below
      if (y < yLen - 1) {
        liveNeighbourCount += (grid[y+1] >> x) & 1;
        if (x > 0) {
          liveNeighbourCount += (grid[y+1] >> (x - 1)) & 1;
        }
        if (x < xLen - 1) {
          liveNeighbourCount += (grid[y+1] >> (x + 1)) & 1;
        }
      }

      if ((cell && (liveNeighbourCount == 2 || liveNeighbourCount == 3))
          || (!cell && liveNeighbourCount == 3)) {
        newGrid[y] |= 1 << x;
      }
      // all other cells stay as 0
    }
  }

  for (int y = 0; y < yLen; y++) {
    grid[y] = newGrid[y];
  }
}

void printGrid(int xLen, int yLen, unsigned long grid[]) {
  for (int y = 0; y < yLen; y++) {
    for (int x = xLen - 1; x >= 0; x--) {
      char cell = (grid[y] >> x) & 1;
      printf(cell ? "#" : "-");
    }
    printf("\n");
  }
}

void life(int xLen, int yLen, unsigned long grid[], int numGenerations) {
    printGrid(xLen, yLen, grid);
    printf("\n");
    int generationNumber = 0;
    while (generationNumber < numGenerations) {
        generationNumber++;
        nextGeneration(xLen, yLen, grid);
        printGrid(xLen, yLen, grid);
        printf("\n");
    }
}

/**
 * args:
 *  grid size x
 *  grid size y
 *  num generations
 *  array of initial row contents
 */
int main(int argc, char* argv[]) {
    if (argc < 4) {
        printf("Please specify x and y dimensions and number of generations.\n");
        return 1;
    }
    int xLen = (int) strtol(argv[1], NULL, 10);
    int yLen = (int) strtol(argv[2], NULL, 10);
    printf("xLen: %d, yLen: %d\n", xLen, yLen);

    if (xLen <= 0) {
        printf("xlen must be greater than 0.\n");
        return 2;
    }

    if (yLen <= 0) {
        printf("ylen must be greater than 0.\n");
        return 2;
    }

    if (xLen <= 0 || yLen <= 0) {
        printf("Dimensions must be greater than 0.\n");
        return 2;
    }

    int numGenerations = (int) strtol(argv[3], NULL, 10);
    printf("Num generations: %d\n", numGenerations);

    if (numGenerations < 1) {
        printf("Num generations must be at least 1.\n");
        return 3;
    }

    if (argc != yLen + 4) {
        printf("Please specify initial contents for the %d rows\n", yLen);
        return 1;
    }

    unsigned long grid[yLen];
    for (int y = 0; y < yLen; y++) {
        char *ptr;
        unsigned long row = strtoul(argv[y + 4], &ptr, 2);
        printf("row input: %lu\n", row);
        grid[y] = row;
    }

    life(xLen, yLen, grid, numGenerations);

    return 0;
}
