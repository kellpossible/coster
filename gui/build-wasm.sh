#!/bin/sh
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
pushd $DIR
wasm-pack build --target web --out-dir ../public/js/gui --debug
# wasm-pack build --target web --out-dir ../public/js/gui
popd
