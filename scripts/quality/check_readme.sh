#!/bin/bash


README="cargo readme"

version=$(cargo readme -V 2>&1)

RES=$?

if [[ $RES -ne 0 ]]; then
  echo 1>&2 "Please install cargo-readme with "'`'"cargo install cargo-readme"'`'
  exit 1
fi

set -e

param=$1

# Function to check the README file of a crate
function check {
  if [ ! -d $1 ]; then
    echo 1>&2 "The directory $1 does not exist"
    exit 1
  fi

  if [ ! -f $1/Cargo.toml ]; then
    echo 1>&2 "The directory $1 does not contain a Cargo.toml file"
    exit 1
  fi

  if [ ! -f $1/README.md ]; then
    echo 1>&2 "The directory $1 does not contain a README.md file"
    exit 1
  fi

  if [ "$param" == "generate" ]; then
    cargo readme -r $1 > $1/README.md
  else
    diff <(cargo readme -r $1) $1/README.md || (echo 1>&2 "Please update the $1/README with "'`'"cargo readme -r $1 > $1/README.md"'`' && exit 1 )
  fi
}

# Check the README file of the agglayer crates
check crates/agglayer-signer
