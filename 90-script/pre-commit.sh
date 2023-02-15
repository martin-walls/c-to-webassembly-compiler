#!/usr/bin/env bash

# only run pre-commit tests if code dir has changed
git diff --cached --name-only | if grep --quiet "03-code/"
then
  echo "Running pre-commit tests"

  # run tests
  cd 03-code || exit
  # this makes sure the commit hook works when committing through CLion
  # without it, it can't find the path to `cargo`
  PATH=$PATH:/home/martin/.cargo/bin
  just test
fi

# $? stores exit value of the last command
if [ $? -ne 0 ]; then
 echo "Tests must pass before commit"
 exit 1
fi

