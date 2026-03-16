#!/bin/bash
set -e

# Configuration
LIB_NAME="musicbirb"      # The name in [lib] name = "..."
SCAFFOLD_NAME="musicbirb"  # The name in uniffi::setup_scaffolding!("...")
BINDINGS_DIR="ios/Bindings"
TEMP_HEADERS="$BINDINGS_DIR/headers"

echo "🧨 Cleaning up old artifacts..."
rm -rf "$BINDINGS_DIR"
mkdir -p "$TEMP_HEADERS"

echo "🦀 Building host library for metadata extraction..."
cargo build -p ffi --release
if [[ "$OSTYPE" == "darwin"* ]]; then
    HOST_LIB="target/release/lib$LIB_NAME.dylib"
else
    HOST_LIB="target/release/lib$LIB_NAME.so"
fi

echo "🧬 Generating UniFFI Swift bindings..."
# Note: UniFFI uses the name from setup_scaffolding!() for filenames
cargo run -p ffi --bin uniffi-bindgen generate --library "$HOST_LIB" --language swift --out-dir "$BINDINGS_DIR"

# FIX: Use curly braces ${VAR} so bash doesn't look for a variable named SCAFFOLD_NAMEFFI
SWIFT_FILE="$BINDINGS_DIR/${SCAFFOLD_NAME}.swift"
HEADER_FILE="$BINDINGS_DIR/${SCAFFOLD_NAME}FFI.h"
MODULEMAP_FILE="$BINDINGS_DIR/${SCAFFOLD_NAME}FFI.modulemap"

# Verify files exist before proceeding
if [[ ! -f "$HEADER_FILE" ]]; then
    echo "❌ Error: Expected header file not found at $HEADER_FILE"
    echo "Files in $BINDINGS_DIR:"
    ls "$BINDINGS_DIR"
    exit 1
fi

echo "📦 Preparing XCFramework headers..."
cp "$HEADER_FILE" "$TEMP_HEADERS/"
cp "$MODULEMAP_FILE" "$TEMP_HEADERS/"

echo "🏗️ Building Rust static libraries for iOS..."
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo build -p ffi --target aarch64-apple-ios --release
cargo build -p ffi --target aarch64-apple-ios-sim --release

echo "📦 Creating XCFramework..."
xcodebuild -create-xcframework \
    -library "target/aarch64-apple-ios/release/lib$LIB_NAME.a" -headers "$TEMP_HEADERS" \
    -library "target/aarch64-apple-ios-sim/release/lib$LIB_NAME.a" -headers "$TEMP_HEADERS" \
    -output "$BINDINGS_DIR/$LIB_NAME.xcframework"

# Clean up temp headers, but keep the originals in BINDINGS_DIR for Xcode's bridging header
rm -rf "$TEMP_HEADERS"

echo "🛠️ Generating Xcode Project..."
if ! command -v xcodegen &> /dev/null; then
    brew install xcodegen
fi

cd ios
xcodegen generate
cd ..

echo "✅ Done! Project generated at ios/Musicbirb.xcodeproj"
