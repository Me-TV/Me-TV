#!/bin/sh

MESON_SOURCE_ROOT="$1"
NAME="$2"
BUILDTYPE="$3"

CARGO_TARGET_DIR="$MESON_SOURCE_ROOT"/target

if [ "$BUILDTYPE" = "release" ]; then
    BUILDOPTION="--release"
else
    BUILDOPTION=""
fi

echo "Build in $BUILDTYPE mode"

echo "cargo build --manifest-path $MESON_SOURCE_ROOT/Cargo.toml $BUILDOPTION && \
    cp $CARGO_TARGET_DIR/$BUILDTYPE/$NAME $NAME"

cargo build --manifest-path "$MESON_SOURCE_ROOT"/Cargo.toml "$BUILDOPTION" && \
    cp "$CARGO_TARGET_DIR"/"$BUILDTYPE"/"$NAME" "$NAME"
