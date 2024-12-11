#!/bin/bash -ex
# Assuming this script is kept in the examples dir
cd "$(dirname "$0")"
BASE_DIR=$(builtin cd .. ; pwd)
EXAMPLES_DIR="$BASE_DIR/examples"
HTDOCS_DIR="$BASE_DIR/htdocs"
CARGO_RELEASE_DIR="$BASE_DIR/target/wasm32-unknown-unknown/release"

export CARGO_BUILD_TARGET=wasm32-unknown-unknown
export TARGET_CC=/usr/bin/clang

for example in simple_box spaceships ;
do
	outdir="$HTDOCS_DIR/$example"
	mkdir -p "$outdir"
	echo "Building $example into '$outdir'"
	cd "$EXAMPLES_DIR/$example"
	cargo build --release --no-default-features -F client
	wasm-bindgen --no-typescript --target web --out-dir "$outdir" --out-name "$example" "$CARGO_RELEASE_DIR/$example.wasm"
	sed -e "s/{{name}}/$example/g" "$EXAMPLES_DIR/common/www/index.html" > "$outdir/index.html"
done
