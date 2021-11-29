#!/bin/bash

cargo run -- build nl/tests/basic_suite.nl nl/std/std.nl -o BasicSuiteJava.class -t java -c -I nl/std

cargo run -- build nl/tests/basic_suite.nl nl/std/std.nl -o basic_suite_wasm.wasm -t wasm -c -I nl/std

cargo run -- build nl/tests/basic_suite.nl nl/std/std.nl -o basic_suite_x86.o -t linux-elf-x86_64 -c -I nl/std

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
	gcc basic_suite_x86.o nl/std/std.c -o basic_suite_x86
fi