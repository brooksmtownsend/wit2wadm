#!/bin/bash

# Ensure wash is installed
if ! [ -x "$(command -v wash)" ]; then
  echo 'Error: wash is not installed.' >&2
  exit 1
fi

# Ensure jco is installed
if ! [ -x "$(command -v jco)" ]; then
  echo 'Error: jco is not installed.' >&2
  exit 1
fi

# Ensure python3
if ! [ -x "$(command -v python3)" ]; then
  echo 'Error: python3 is not installed.' >&2
  exit 1
fi

wash build
jco transpile ./docs/wit2wadm_component.wasm -o ./docs/transpile --no-typescript
python3 -m http.server -d build