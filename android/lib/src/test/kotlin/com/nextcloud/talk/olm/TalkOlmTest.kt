/*
 * SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
 * SPDX-License-Identifier: Apache-2.0
 */

package com.nextcloud.talk.olm

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

// Only checks the bindings, the Olm behaviour is tested in Rust
class TalkOlmTest {

    @Test
    fun exchangesMessagesInBothDirections() {
        val initiator = VodozemacAccount()
        val responder = VodozemacAccount()

        val oneTimeKey = initiator.createOneTimeKey()
        val outbound = responder.createOutboundSession(initiator.identityKey(), oneTimeKey)

        val first = outbound.encrypt("first 👋")
        assertEquals(VodozemacMessageKind.PRE_KEY, first.kind)

        val inbound = initiator.createInboundSession(first)
        assertEquals("first 👋", inbound.plaintext)

        val reply = inbound.session.encrypt("reply")
        assertEquals(VodozemacMessageKind.NORMAL, reply.kind)
        assertEquals("reply", outbound.decrypt(reply))
    }

    @Test
    fun throwsOnInvalidInput() {
        val account = VodozemacAccount()

        assertThrows(VodozemacException.InvalidKey::class.java) {
            account.createOutboundSession("invalid", "invalid")
        }

        assertThrows(VodozemacException.InvalidMessage::class.java) {
            account.createInboundSession(VodozemacMessage(VodozemacMessageKind.NORMAL, "AwoQ"))
        }
    }
}
