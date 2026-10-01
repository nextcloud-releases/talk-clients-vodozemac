#!/bin/sh
# SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
# SPDX-License-Identifier: Apache-2.0

# The header and module map in build/include are for the XCFramework

set -eu
cd "$(dirname "$0")/.."

target_dir="${CARGO_TARGET_DIR:-target}"
out_dir="build/bindings"

cargo build --locked --release --lib

case "$(uname -s)" in
    Darwin) library="$target_dir/release/libtalk_olm.dylib" ;;
    *) library="$target_dir/release/libtalk_olm.so" ;;
esac

rm -rf "$out_dir" build/include
# Unformatted, so the output is the same with or without swift-format installed
cargo run --locked --quiet --features bindgen --bin uniffi-bindgen -- \
    generate "$library" --language swift --no-format --out-dir "$out_dir"

mkdir -p build/include swift/Sources/TalkOlm
mv "$out_dir/TalkOlm.swift" swift/Sources/TalkOlm/TalkOlm.swift
mv "$out_dir/TalkOlmFFI.h" build/include/TalkOlmFFI.h
mv "$out_dir/TalkOlmFFI.modulemap" build/include/module.modulemap
