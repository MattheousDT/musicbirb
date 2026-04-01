# Configuration

project_path := "ios/Musicbirb.xcodeproj"
scheme := "Musicbirb"
bundle_id := "com.musicbirb.Musicbirb"
derived_data := ".build/xcode"
destination := "platform=iOS Simulator,name=iPhone 17 Pro"
has_beautifier := `command -v xcbeautify >/dev/null && echo true || echo false`

# --- iOS Commands ---

clean-ios:
    @echo "🧹 Cleaning iOS build artifacts..."
    @set -o pipefail && xcodebuild -project {{ project_path }} -scheme {{ scheme }} -derivedDataPath {{ derived_data }} clean \
      {{ if has_beautifier == "true" { "| xcbeautify --quiet" } else { "" } }}
    @rm -rf {{ derived_data }}
    @rm -rf ios/Bindings
    @echo "🦀 Cleaning Rust artifacts..."
    @cargo clean
    @echo "✅ Clean complete."

build-ios:
    echo "🏗️ Building {{ scheme }}..."
    xcodebuild \
      -project {{ project_path }} \
      -scheme {{ scheme }} \
      -configuration Debug \
      -destination "{{ destination }}" \
      -derivedDataPath {{ derived_data }} \
      build {{ if has_beautifier == "true" { "| xcbeautify --quiet" } else { "" } }}

run-ios: build-ios
    @echo "📱 Booting Simulator..."
    @open -a Simulator
    @xcrun simctl bootstatus booted -b
    @echo "🚚 Installing App..."
    @xcrun simctl install booted "$(find {{ derived_data }} -name '{{ scheme }}.app' -type d | head -n 1)"
    @echo "🚀 Launching {{ bundle_id }}..."
    @xcrun simctl launch --console booted {{ bundle_id }}

build-ios-ffi:
    cargo run --package ffi --bin dist

list-sims:
    xcrun simctl list devices available
