//
// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0
//

import XCTest
import TalkOlm

// Only checks the bindings, the Olm behaviour is tested in Rust
final class TalkOlmTests: XCTestCase {

    func testExchangesMessagesInBothDirections() throws {
        let initiator = VodozemacAccount()
        let responder = VodozemacAccount()

        let oneTimeKey = try initiator.createOneTimeKey()
        let outbound = try responder.createOutboundSession(theirIdentityKey: initiator.identityKey(), theirOneTimeKey: oneTimeKey)

        let first = try outbound.encrypt(plaintext: "first 👋")
        XCTAssertEqual(first.kind, .preKey)

        let inbound = try initiator.createInboundSession(preKeyMessage: first)
        XCTAssertEqual(inbound.plaintext, "first 👋")

        let reply = try inbound.session.encrypt(plaintext: "reply")
        XCTAssertEqual(reply.kind, .normal)
        XCTAssertEqual(try outbound.decrypt(message: reply), "reply")
    }

    func testThrowsOnInvalidInput() {
        let account = VodozemacAccount()

        XCTAssertThrowsError(try account.createOutboundSession(theirIdentityKey: "invalid", theirOneTimeKey: "invalid")) { error in
            guard case .InvalidKey? = error as? VodozemacError else { return XCTFail("Unexpected error \(error)") }
        }

        XCTAssertThrowsError(try account.createInboundSession(preKeyMessage: VodozemacMessage(kind: .normal, body: "AwoQ"))) { error in
            guard case .InvalidMessage? = error as? VodozemacError else { return XCTFail("Unexpected error \(error)") }
        }
    }
}
