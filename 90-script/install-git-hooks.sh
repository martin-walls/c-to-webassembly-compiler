#!/usr/bin/env bash

GIT_DIR=$(git rev-parse --git-dir)

echo "Installing git hooks..."
# this command creates symlink to our pre-commit script
ln -s ../../90-script/pre-commit.sh $GIT_DIR/hooks/pre-commit
echo "Done!"
