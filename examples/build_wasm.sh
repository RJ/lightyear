#!/bin/bash -ex
# Run shellcheck on this after modifications.
# Assuming this script is kept in the examples dir
cd "$(dirname "$0")"
BASE_DIR=$(builtin cd .. ; pwd)
EXAMPLES_DIR="$BASE_DIR/examples"
HTDOCS_DIR="$BASE_DIR/htdocs"
CARGO_RELEASE_DIR="$BASE_DIR/target/wasm32-unknown-unknown/release"

export CARGO_BUILD_TARGET=wasm32-unknown-unknown
export TARGET_CC=/usr/bin/clang

EXAMPLES_LIST="simple_box spaceships"

for example in $EXAMPLES_LIST ;
do
	(
	outdir="$HTDOCS_DIR/$example"
	mkdir -p "$outdir"
	echo "Building $example into '$outdir'"
	cd "$EXAMPLES_DIR/$example"
	cargo build --release --no-default-features -F client
	wasm-bindgen --no-typescript --target web --out-dir "$outdir" --out-name "$example" "$CARGO_RELEASE_DIR/$example.wasm"
	sed -e "s/{{name}}/$example/g" "$EXAMPLES_DIR/common/www/index.html" > "$outdir/index.html"
	)
done

echo "htdocs dir: $HTDOCS_DIR"
pwd
ls

# TODO write a proper template for this:
echo "<html><head><title>Lightyear Examples Menu</title></head><body>" > "$HTDOCS_DIR/index.html"

for example in $EXAMPLES_LIST ; do
	echo "<ul><a href=\"$example/\">$example</a></ul>" >> "$HTDOCS_DIR/index.html"
done

echo "</body></html>" >> "$HTDOCS_DIR/index.html"

echo "$HTDOCS_DIR is ready to ship"
