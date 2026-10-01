#!/bin/sh
# SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
# SPDX-License-Identifier: Apache-2.0

# Needs cargo-ndk (cargo install --locked cargo-ndk) and an NDK found through ANDROID_NDK_HOME or the SDK,
# r28 or newer so the libraries are 16 KB page aligned

set -eu
cd "$(dirname "$0")/.."

jni_libs="android/lib/src/main/jniLibs"

./scripts/generate-bindings.sh

rm -rf "$jni_libs"
# The minimum SDK of Talk Android
cargo ndk --platform 26 -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 -o "$jni_libs" \
    build --locked --release --lib

gradle -p android :lib:assembleRelease :lib:generatePomFileForReleasePublication
echo android/lib/build/outputs/aar/lib-release.aar
echo android/lib/build/publications/release/pom-default.xml
