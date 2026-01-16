#!/bin/zsh

if [[ "$1" == "" ]]; then
  echo "Specify a module to compile: make <main>"
  exit -1
fi

mkdir -p target
as -o target/$1.o $1.s && ld -o target/$1 target/$1.o -lSystem -L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib && target/$1
