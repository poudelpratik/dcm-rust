#!/bin/bash

# Run wasm-generator
if [ "${skip_wasm_generator}" != "true" ]; then
    /usr/local/bin/wasm-generator
fi

if [ "${only_wasm_generator}" = "true" ]; then
    exit 0
fi
