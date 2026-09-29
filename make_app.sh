#!/bin/bash
# Build a separate personal app, without replacing the installed Shuffle.
set -euo pipefail
cd "$(dirname "$0")"
PROFILE="${SHUFFLE_PROFILE:-release}"
case "$PROFILE" in
    release) cargo build --locked --release --features runtime-shaders,local-build ;;
    debug) cargo build --locked --features runtime-shaders,local-build ;;
    *) echo "SHUFFLE_PROFILE must be release or debug" >&2; exit 1 ;;
esac
BUILD_DIR="${CARGO_TARGET_DIR:-target}"
APP="Shuffle Flow.app"
VERSION=$(sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -1)
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BUILD_DIR/$PROFILE/shuffle" "$APP/Contents/MacOS/shuffle"
cp AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
# Use the same optional native helpers as the upstream bundle, for the host Mac.
for helper in removebg cloudctl; do
    if { [ ! -f "$APP/Contents/MacOS/$helper" ] || [ "$helper.swift" -nt "$APP/Contents/MacOS/$helper" ]; } && command -v swiftc >/dev/null 2>&1; then
        if swiftc -O "$helper.swift" -o "$APP/Contents/MacOS/$helper"; then
            codesign --force --sign - "$APP/Contents/MacOS/$helper"
        else
            echo "Optional helper $helper could not be built." >&2
        fi
    fi
done
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Shuffle Flow</string>
<key>CFBundleDisplayName</key><string>Shuffle Flow</string>
<key>CFBundleExecutable</key><string>shuffle</string>
<key>CFBundleIdentifier</key><string>com.shuffle.local.zh</string>
<key>CFBundleIconFile</key><string>AppIcon</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$VERSION</string>
<key>CFBundleVersion</key><string>$VERSION</string>
<key>CFBundleDevelopmentRegion</key><string>zh-Hans</string>
<key>CFBundleLocalizations</key><array><string>zh-Hans</string><string>en</string></array>
<key>LSMinimumSystemVersion</key><string>12.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
</dict></plist>
PLIST
codesign --force --sign - --identifier com.shuffle.local.zh "$APP"
echo "Built $APP"
