#!/usr/bin/env bash
# Builds, signs and (optionally) uploads the Mac App Store package.
# Must run on a Mac with Xcode command line tools.
#
# One-time setup (details in the roadmap, "Mac App Store" section):
#   1. In src-tauri/Entitlements.plist replace TEAMID with your Team ID.
#   2. Save your "Mac App Store Connect" provisioning profile as
#      src-tauri/embedded.provisionprofile
#   3. Install the two certificates in your keychain:
#      "Apple Distribution" and "Mac Installer Distribution".
#
# Usage:
#   APPLE_SIGNING_IDENTITY="Apple Distribution: Your Name (TEAMID)" \
#   INSTALLER_IDENTITY="3rd Party Mac Developer Installer: Your Name (TEAMID)" \
#   packaging/macos/build-appstore.sh
#
#   Add APPLE_API_KEY_ID and APPLE_API_ISSUER to upload straight to App Store Connect.
set -euo pipefail
cd "$(dirname "$0")/../.."

: "${APPLE_SIGNING_IDENTITY:?set APPLE_SIGNING_IDENTITY}"
: "${INSTALLER_IDENTITY:?set INSTALLER_IDENTITY}"
[ -f src-tauri/embedded.provisionprofile ] || { echo "Missing src-tauri/embedded.provisionprofile"; exit 1; }
grep -q "<string>TEAMID" src-tauri/Entitlements.plist && { echo "Replace TEAMID in src-tauri/Entitlements.plist first"; exit 1; }

rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm ci
npx tauri build --bundles app --target universal-apple-darwin --features mas \
  --config src-tauri/tauri.appstore.conf.json

APP="target/universal-apple-darwin/release/bundle/macos/HappyWiggle.app"
PKG="HappyWiggle.pkg"
xcrun productbuild --sign "$INSTALLER_IDENTITY" --component "$APP" /Applications "$PKG"
echo "Created $PKG"

if [ -n "${APPLE_API_KEY_ID:-}" ] && [ -n "${APPLE_API_ISSUER:-}" ]; then
  xcrun altool --upload-app --type macos --file "$PKG" --apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER"
else
  echo "Not uploading (no API key given). Drag $PKG into the Transporter app instead."
fi
