#!/usr/bin/env bash

# Enable debugging mode
set -x
# Immediately exit on first fail command and print the first exit status of the first failing command in the pipeline.
set -eo pipefail

if ! [ -x "$(command -v cargo-clippy)" ]; then
  echo >&2 "Error: clippy is not installed."
  echo >&2 "Use:"
  echo >&2 "    rustup add component clippy"
  echo >&2 "to install it."
fi

if ! [ -x "$(command -v cargo-tarpaulin)" ]; then
  echo >&2 "Error: tarpaulin is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install cargo-tarpaulin"
  echo >&2 "to install it."
fi


clippy_filename="clippy.json"
if [ -f "$clippy_filename" ]; then
  rm "$clippy_filename"
  echo >&2 "Removed file $clippy_filename"
fi
cargo clippy --message-format=json >> "$clippy_filename"
echo >&2 "Created code report file: clippy.json"

lcov_filename="lcov.info"
if [ -f "$lcov_filename" ]; then
  rm "$lcov_filename"
  echo >&2 "Removed file $lcov_filename"
fi
cargo tarpaulin --out Lcov
echo >&2 "Created code coverage file: lcov.info"
