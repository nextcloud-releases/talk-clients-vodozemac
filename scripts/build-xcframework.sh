#!/bin/sh
# SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
# SPDX-License-Identifier: Apache-2.0

set -eu
cd "$(dirname "$0")/.."

target_dir="${CARGO_TARGET_DIR:-target}"
# The minimum deployment target of Talk iOS
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-16.0}"

./scripts/generate-bindings.sh

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release --lib --target "$target"
done

rm -rf build/TalkOlmFFI.xcframework build/TalkOlmFFI.xcframework.zip
xcodebuild -create-xcframework \
    -library "$target_dir/aarch64-apple-ios/release/libtalk_olm.a" -headers build/include \
    -library "$target_dir/aarch64-apple-ios-sim/release/libtalk_olm.a" -headers build/include \
    -output build/TalkOlmFFI.xcframework

cp LICENSES/Apache-2.0.txt build/TalkOlmFFI.xcframework/LICENSE.txt
cp THIRD_PARTY_LICENSES.md build/TalkOlmFFI.xcframework/THIRD_PARTY_LICENSES.md

(cd build && ditto -c -k --sequesterRsrc --keepParent TalkOlmFFI.xcframework TalkOlmFFI.xcframework.zip)
swift package compute-checksum build/TalkOlmFFI.xcframework.zip
