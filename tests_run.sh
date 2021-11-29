#!/bin/bash

echo "========================== java =========================="
java BasicSuiteJava

echo "========================== wasm =========================="
node nl/examples/wasm_exec.js basic_suite_wasm.wasm

# Doesn't check if x86-64
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
	echo "========================== x86 =========================="
	./basic_suite_x86
fi