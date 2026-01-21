#!/bin/bash
# Build script for Claude Config Manager release

set -e

VERSION=${1:-"local"}
OUTPUT_DIR="./dist"

echo "Building Claude Config Manager v$VERSION"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build for current platform
echo "Building release binary..."
cargo build --release --bin ccm

# Copy binary to output directory
if [ "$(uname)" = "Darwin" ]; then
    if [ "$(uname -m)" = "arm64" ]; then
        TARGET="macos-aarch64"
    else
        TARGET="macos-x86_64"
    fi
    cp target/release/ccm "$OUTPUT_DIR/ccm-$TARGET"
    cd "$OUTPUT_DIR"
    tar czf "ccm-$TARGET-$VERSION.tar.gz" "ccm-$TARGET"
    rm "ccm-$TARGET"
    echo "Created: ccm-$TARGET-$VERSION.tar.gz"
elif [ "$(uname)" = "Linux" ]; then
    TARGET="linux-x86_64"
    strip target/release/ccm
    cp target/release/ccm "$OUTPUT_DIR/ccm-$TARGET"
    cd "$OUTPUT_DIR"
    tar czf "ccm-$TARGET-$VERSION.tar.gz" "ccm-$TARGET"
    rm "ccm-$TARGET"
    echo "Created: ccm-$TARGET-$VERSION.tar.gz"
else
    TARGET="windows-x86_64"
    cp target/release/ccm.exe "$OUTPUT_DIR/ccm-$TARGET.exe"
    cd "$OUTPUT_DIR"
    7z a "ccm-$TARGET-$VERSION.zip" "ccm-$TARGET.exe" > /dev/null
    rm "ccm-$TARGET.exe"
    echo "Created: ccm-$TARGET-$VERSION.zip"
fi

echo ""
echo "Build complete! Artifacts in $OUTPUT_DIR"
