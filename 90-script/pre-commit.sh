#!/usr/bin/env bash

git diff --cached --name-only | if grep --quiet "03-code/"
then
  echo "Running pre-commit tests"

  # run tests
  cd 03-code || exit
  just test
fi

# $? stores exit value of the last command
if [ $? -ne 0 ]; then
 echo "Tests must pass before commit"
 exit 1
fi

