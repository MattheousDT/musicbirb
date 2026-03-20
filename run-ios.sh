#!/bin/bash
# Exit on error, and ensure pipe failures are captured
set -e
set -o pipefail

# Configuration
SCHEME="Musicbirb"
BUNDLE_ID="com.musicbirb.Musicbirb"
DESTINATION="platform=iOS Simulator,name=iPhone 17 Pro"
SIM_NAME="iPhone 17 Pro"

# Check for xcbeautify
if ! command -v xcbeautify >/dev/null 2>&1; then
    echo "💡 Tip: Install xcbeautify for better logs: brew install xcbeautify"
    FORMATTER="cat"
else
    # --quiet only shows errors and warnings
    FORMATTER="xcbeautify --quiet"
fi

echo "🏗️  Building Swift App (Errors only)..."

# We use -parallelizeTargets and -jobs for speed, and pipe to the formatter
# If you need to see EVERYTHING again, remove '| $FORMATTER'
xcodebuild -project ios/Musicbirb.xcodeproj \
           -scheme "$SCHEME" \
           -destination "$DESTINATION" \
           -configuration Debug \
           -derivedDataPath ios/build \
           build | $FORMATTER

echo "📱 Preparing Simulator..."
xcrun simctl boot "$SIM_NAME" > /dev/null 2>&1 || true
xcrun simctl bootstatus "$SIM_NAME" > /dev/null 2>&1 || true

echo "🚀 Installing and Launching..."
APP_PATH=$(find ios/build -name "Musicbirb.app" -type d -print -quit)

if [ -z "$APP_PATH" ]; then
    echo "❌ Error: Could not find built .app file"
    exit 1
fi

xcrun simctl install booted "$APP_PATH"

echo "🚀 Launching App..."
xcrun simctl launch booted "$BUNDLE_ID"

echo "💡 Streaming App Logs (Subsystem: $BUNDLE_ID)..."
# We filter by subsystem to hide all the layout/audio-engine noise
xcrun simctl spawn booted log stream --predicate "subsystem == '$BUNDLE_ID'" --level info
