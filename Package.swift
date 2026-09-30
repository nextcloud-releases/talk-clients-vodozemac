// swift-tools-version: 5.9
//
// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0
//

import Foundation
import PackageDescription

// Set on the release tag by the release workflow
let releaseURL = "https://github.com/nextcloud-releases/talk-clients-vodozemac/releases/download/0.0.0/TalkOlmFFI.xcframework.zip"
let releaseChecksum = "0000000000000000000000000000000000000000000000000000000000000000"

// TALK_OLM_LOCAL_BUILD=1 uses the output of scripts/build-xcframework.sh
let ffiTarget: Target = ProcessInfo.processInfo.environment["TALK_OLM_LOCAL_BUILD"] != nil
    ? .binaryTarget(name: "TalkOlmFFI", path: "build/TalkOlmFFI.xcframework")
    : .binaryTarget(name: "TalkOlmFFI", url: releaseURL, checksum: releaseChecksum)

let package = Package(
    name: "TalkOlm",
    platforms: [.iOS(.v16)],
    products: [
        .library(name: "TalkOlm", targets: ["TalkOlm"])
    ],
    targets: [
        ffiTarget,
        .target(name: "TalkOlm", dependencies: ["TalkOlmFFI"], path: "swift/Sources/TalkOlm"),
        .testTarget(name: "TalkOlmTests", dependencies: ["TalkOlm"], path: "swift/Tests/TalkOlmTests")
    ]
)
