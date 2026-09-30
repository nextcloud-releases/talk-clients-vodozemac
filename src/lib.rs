// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0

//! What the Talk call key exchange needs from vodozemac, compatible with the libolm of the web client.

use std::sync::{Arc, Mutex, MutexGuard};

use vodozemac::Curve25519PublicKey;
use vodozemac::olm::{self, OlmMessage, SessionConfig};

uniffi::setup_scaffolding!();

// libolm only speaks version 1
const SESSION_CONFIG: SessionConfig = SessionConfig::version_1();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum VodozemacError {
    #[error("Invalid key: {reason}")]
    InvalidKey { reason: String },
    #[error("Invalid message: {reason}")]
    InvalidMessage { reason: String },
    #[error("Session creation failed: {reason}")]
    SessionCreation { reason: String },
    #[error("Decryption failed: {reason}")]
    Decryption { reason: String },
    #[error("Encryption failed: {reason}")]
    Encryption { reason: String },
}

/// The raw values are the Olm message types on the wire
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum VodozemacMessageKind {
    PreKey = 0,
    Normal = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct VodozemacMessage {
    pub kind: VodozemacMessageKind,
    pub body: String,
}

#[derive(uniffi::Record)]
pub struct VodozemacInboundSession {
    pub session: Arc<VodozemacSession>,
    pub plaintext: String,
}

#[derive(uniffi::Object)]
pub struct VodozemacAccount {
    inner: Mutex<olm::Account>,
}

#[uniffi::export]
impl VodozemacAccount {
    #[uniffi::constructor]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { inner: Mutex::new(olm::Account::new()) }
    }

    /// Curve25519
    pub fn identity_key(&self) -> String {
        lock(&self.inner).curve25519_key().to_base64()
    }

    /// Creates a one-time key and marks it as published
    pub fn create_one_time_key(&self) -> Result<String, VodozemacError> {
        let mut account = lock(&self.inner);
        let result = account.generate_one_time_keys(1);
        account.mark_keys_as_published();

        result
            .created
            .first()
            .map(Curve25519PublicKey::to_base64)
            .ok_or_else(|| VodozemacError::InvalidKey { reason: "no one-time key created".into() })
    }

    pub fn create_outbound_session(
        &self,
        their_identity_key: String,
        their_one_time_key: String,
    ) -> Result<Arc<VodozemacSession>, VodozemacError> {
        let identity_key = parse_key(&their_identity_key)?;
        let one_time_key = parse_key(&their_one_time_key)?;

        let session = lock(&self.inner)
            .create_outbound_session(SESSION_CONFIG, identity_key, one_time_key)
            .map_err(|e| VodozemacError::SessionCreation { reason: e.to_string() })?;

        Ok(Arc::new(VodozemacSession::new(session)))
    }

    /// Also decrypts the message, vodozemac cannot decrypt it again afterwards
    pub fn create_inbound_session(
        &self,
        pre_key_message: VodozemacMessage,
    ) -> Result<VodozemacInboundSession, VodozemacError> {
        let OlmMessage::PreKey(message) = decode_message(&pre_key_message)? else {
            return Err(VodozemacError::InvalidMessage { reason: "not a pre-key message".into() });
        };

        // Talk sends no identity key along with the message, so the one inside it is used
        let result = lock(&self.inner)
            .create_inbound_session(SESSION_CONFIG, message.identity_key(), &message)
            .map_err(|e| VodozemacError::SessionCreation { reason: e.to_string() })?;

        Ok(VodozemacInboundSession {
            session: Arc::new(VodozemacSession::new(result.session)),
            plaintext: into_string(result.plaintext)?,
        })
    }
}

#[derive(uniffi::Object)]
pub struct VodozemacSession {
    inner: Mutex<olm::Session>,
}

impl VodozemacSession {
    fn new(session: olm::Session) -> Self {
        Self { inner: Mutex::new(session) }
    }
}

#[uniffi::export]
impl VodozemacSession {
    pub fn encrypt(&self, plaintext: String) -> Result<VodozemacMessage, VodozemacError> {
        let message =
            lock(&self.inner).encrypt(plaintext).map_err(|e| VodozemacError::Encryption { reason: e.to_string() })?;

        let (message_type, bytes) = message.to_parts();
        let kind = if message_type == 0 { VodozemacMessageKind::PreKey } else { VodozemacMessageKind::Normal };

        Ok(VodozemacMessage { kind, body: vodozemac::base64_encode(bytes) })
    }

    pub fn decrypt(&self, message: VodozemacMessage) -> Result<String, VodozemacError> {
        let message = decode_message(&message)?;
        let plaintext =
            lock(&self.inner).decrypt(&message).map_err(|e| VodozemacError::Decryption { reason: e.to_string() })?;

        into_string(plaintext)
    }
}

// A panic while holding the lock is caught by UniFFI, the session is unusable afterwards anyway
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn parse_key(key: &str) -> Result<Curve25519PublicKey, VodozemacError> {
    Curve25519PublicKey::from_base64(key).map_err(|e| VodozemacError::InvalidKey { reason: e.to_string() })
}

fn decode_message(message: &VodozemacMessage) -> Result<OlmMessage, VodozemacError> {
    let bytes = vodozemac::base64_decode(&message.body)
        .map_err(|e| VodozemacError::InvalidMessage { reason: e.to_string() })?;

    OlmMessage::from_parts(message.kind as usize, &bytes)
        .map_err(|e| VodozemacError::InvalidMessage { reason: e.to_string() })
}

fn into_string(plaintext: Vec<u8>) -> Result<String, VodozemacError> {
    String::from_utf8(plaintext).map_err(|e| VodozemacError::Decryption { reason: e.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_session(
        initiator: &VodozemacAccount,
        responder: &VodozemacAccount,
    ) -> (Arc<VodozemacSession>, Arc<VodozemacSession>) {
        let one_time_key = initiator.create_one_time_key().unwrap();
        let outbound = responder.create_outbound_session(initiator.identity_key(), one_time_key).unwrap();

        let first = outbound.encrypt("first".into()).unwrap();
        assert_eq!(first.kind, VodozemacMessageKind::PreKey);

        let inbound = initiator.create_inbound_session(first).unwrap();
        assert_eq!(inbound.plaintext, "first");

        (outbound, inbound.session)
    }

    #[test]
    fn exchanges_messages_in_both_directions() {
        let initiator = VodozemacAccount::new();
        let responder = VodozemacAccount::new();
        let (outbound, inbound) = start_session(&initiator, &responder);

        let reply = inbound.encrypt("reply".into()).unwrap();
        assert_eq!(reply.kind, VodozemacMessageKind::Normal);
        assert_eq!(outbound.decrypt(reply).unwrap(), "reply");

        // Once a reply arrived, the outbound side sends normal messages too
        let next = outbound.encrypt("next".into()).unwrap();
        assert_eq!(next.kind, VodozemacMessageKind::Normal);
        assert_eq!(inbound.decrypt(next).unwrap(), "next");
    }

    #[test]
    fn keys_are_unpadded_base64() {
        let account = VodozemacAccount::new();

        for key in [account.identity_key(), account.create_one_time_key().unwrap()] {
            assert_eq!(key.len(), 43);
            assert!(!key.ends_with('='));
        }
    }

    #[test]
    fn one_time_key_is_used_once() {
        let initiator = VodozemacAccount::new();
        let responder = VodozemacAccount::new();
        let one_time_key = initiator.create_one_time_key().unwrap();

        let first = responder.create_outbound_session(initiator.identity_key(), one_time_key.clone()).unwrap();
        let second = responder.create_outbound_session(initiator.identity_key(), one_time_key).unwrap();

        initiator.create_inbound_session(first.encrypt("first".into()).unwrap()).unwrap();
        assert!(matches!(
            initiator.create_inbound_session(second.encrypt("second".into()).unwrap()),
            Err(VodozemacError::SessionCreation { .. })
        ));
    }

    #[test]
    fn forged_message_keeps_one_time_key() {
        let initiator = VodozemacAccount::new();
        let responder = VodozemacAccount::new();
        let one_time_key = initiator.create_one_time_key().unwrap();
        let outbound = responder.create_outbound_session(initiator.identity_key(), one_time_key).unwrap();
        let message = outbound.encrypt("first".into()).unwrap();

        // Flipping a bit of the ciphertext breaks the MAC, but not the pre-key header
        let mut bytes = vodozemac::base64_decode(&message.body).unwrap();
        let last = bytes.len() - 9;
        bytes[last] ^= 1;
        let forged = VodozemacMessage { kind: message.kind, body: vodozemac::base64_encode(bytes) };

        assert!(initiator.create_inbound_session(forged).is_err());
        assert_eq!(initiator.create_inbound_session(message).unwrap().plaintext, "first");
    }

    #[test]
    fn rejects_invalid_input() {
        let account = VodozemacAccount::new();

        assert!(matches!(
            account.create_outbound_session("invalid".into(), account.identity_key()),
            Err(VodozemacError::InvalidKey { .. })
        ));

        let normal = VodozemacMessage { kind: VodozemacMessageKind::Normal, body: "AwoQ".into() };
        assert!(matches!(account.create_inbound_session(normal), Err(VodozemacError::InvalidMessage { .. })));

        let invalid = VodozemacMessage { kind: VodozemacMessageKind::PreKey, body: "not base64!".into() };
        assert!(matches!(account.create_inbound_session(invalid), Err(VodozemacError::InvalidMessage { .. })));
    }
}
