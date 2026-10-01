// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0

// The Talk key exchange between this crate and @matrix-org/olm, called like spreed src/utils/e2ee/encryption.js does

import assert from 'node:assert/strict'
import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'
import Olm from '@matrix-org/olm'

const PRE_KEY = 0
const NORMAL = 1

class Peer {
	constructor(binary) {
		this.process = spawn(binary, [], { stdio: ['pipe', 'pipe', 'inherit'] })
		this.responses = createInterface({ input: this.process.stdout })[Symbol.asyncIterator]()
	}

	async send(command) {
		this.process.stdin.write(JSON.stringify(command) + '\n')
		const { value, done } = await this.responses.next()
		assert.ok(!done, 'peer exited')

		const response = JSON.parse(value)
		assert.equal(response.error, undefined, `peer failed on ${command.op}`)
		return response
	}

	close() {
		this.process.stdin.end()
	}
}

function createWebAccount() {
	const account = new Olm.Account()
	account.create()
	return account
}

function webOneTimeKey(account) {
	account.generate_one_time_keys(1)
	const key = Object.values(JSON.parse(account.one_time_keys()).curve25519)[0]
	account.mark_keys_as_published()
	return key
}

// The key payload as encryption.js sends it
function keyPayload(index) {
	return JSON.stringify({ key: Buffer.alloc(32, index).toString('base64'), index })
}

async function exchangeMore(peer, webSession) {
	for (let index = 1; index <= 3; index++) {
		const fromWeb = webSession.encrypt(keyPayload(index))
		assert.equal(fromWeb.type, NORMAL)
		assert.equal((await peer.send({ op: 'decrypt', ...fromWeb })).plaintext, keyPayload(index))

		const fromPeer = await peer.send({ op: 'encrypt', plaintext: keyPayload(index + 10) })
		assert.equal(fromPeer.type, NORMAL)
		assert.equal(webSession.decrypt(fromPeer.type, fromPeer.body), keyPayload(index + 10))
	}
}

async function webStarts(binary) {
	const peer = new Peer(binary)
	const account = createWebAccount()
	const session = new Olm.Session()

	try {
		// encryption.start
		await peer.send({ op: 'outbound', identity: JSON.parse(account.identity_keys()).curve25519, key: webOneTimeKey(account) })

		// encryption.finish
		const finish = await peer.send({ op: 'encrypt', plaintext: keyPayload(0) })
		assert.equal(finish.type, PRE_KEY)

		session.create_inbound(account, finish.body)
		account.remove_one_time_keys(session)
		assert.equal(session.decrypt(finish.type, finish.body), keyPayload(0))

		await exchangeMore(peer, session)
	} finally {
		session.free()
		account.free()
		peer.close()
	}
}

async function peerStarts(binary) {
	const peer = new Peer(binary)
	const account = createWebAccount()
	const session = new Olm.Session()

	try {
		// encryption.start
		const identity = (await peer.send({ op: 'identity' })).key
		const oneTimeKey = (await peer.send({ op: 'one_time_key' })).key
		session.create_outbound(account, identity, oneTimeKey)

		// encryption.finish, non-ASCII to check the string conversion
		const plaintext = keyPayload(0) + ' ü 👋'
		const finish = session.encrypt(plaintext)
		assert.equal(finish.type, PRE_KEY)
		assert.equal((await peer.send({ op: 'inbound', ...finish })).plaintext, plaintext)

		const reply = await peer.send({ op: 'encrypt', plaintext: keyPayload(5) })
		assert.equal(reply.type, NORMAL)
		assert.equal(session.decrypt(reply.type, reply.body), keyPayload(5))

		await exchangeMore(peer, session)
	} finally {
		session.free()
		account.free()
		peer.close()
	}
}

const binary = process.argv[2]
assert.ok(binary, 'usage: node interop.mjs <interop_peer binary>')

await Olm.init()

for (const test of [webStarts, peerStarts]) {
	await test(binary)
	console.log(`ok ${test.name}`)
}
