#!/bin/sh
set -o errexit -o nounset
for arg in "$@"; do
  echo "$arg"
done
