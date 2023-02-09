#!/usr/bin/env bash

echo "Running pre-commit hook"

# run tests
cd 03-code || exit
just test

# $? stores exit value of the last command
if [ $? -ne 0 ]; then
 echo "Tests must pass before commit"
 exit 1
fi
