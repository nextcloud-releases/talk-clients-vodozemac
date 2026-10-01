// swift-tools-version: 5.9
//
// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0
//

import Foundation
import PackageDescription

// Set on the release tag by the release workflow
let releaseURL = "https://github.com/nextcloud-releases/talk-clients-vodozemac/releases/download/0.0.1/TalkOlmFFI.xcframework.zip"
let releaseChecksum = "a14613cd49fbbe2484b274563b7d6f9b48a19c21dba51281e85d716b4ac93b2a"

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
