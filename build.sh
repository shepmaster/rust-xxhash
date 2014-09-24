#!/bin/sh
mkdir -p build
gcc   -fPIC -Ofast -march=native -std=c99 -o build/libxxhash-gcc.a   -c cbits/xxhash.c
clang -fPIC -Ofast -march=native -std=c99 -o build/libxxhash-clang.a -c cbits/xxhash.c

rustc -Lbuild -C no-vectorize-slp --cfg clang --opt-level=3 --out-dir=build lib.rs --test
