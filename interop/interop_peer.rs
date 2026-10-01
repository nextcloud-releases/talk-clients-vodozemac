// SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
// SPDX-License-Identifier: Apache-2.0

//! Peer for interop.mjs, one JSON command per stdin line, one JSON answer per stdout line.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use serde_json::{Value, json};
use talk_olm::{VodozemacAccount, VodozemacMessage, VodozemacMessageKind, VodozemacSession};

fn message(command: &Value) -> Result<VodozemacMessage, String> {
    let kind = match command["type"].as_u64() {
        Some(0) => VodozemacMessageKind::PreKey,
        Some(1) => VodozemacMessageKind::Normal,
        other => return Err(format!("invalid type {other:?}")),
    };
    let body = command["body"].as_str().ok_or("missing body")?.to_owned();

    Ok(VodozemacMessage { kind, body })
}

fn run(
    account: &VodozemacAccount,
    session: &mut Option<Arc<VodozemacSession>>,
    command: &Value,
) -> Result<Value, String> {
    let string = |name: &str| command[name].as_str().map(str::to_owned).ok_or(format!("missing {name}"));
    let current = || session.clone().ok_or("no session".to_owned());

    match command["op"].as_str().ok_or("missing op")? {
        "identity" => Ok(json!({ "key": account.identity_key() })),
        "one_time_key" => Ok(json!({ "key": account.create_one_time_key().map_err(|e| e.to_string())? })),
        "outbound" => {
            let created =
                account.create_outbound_session(string("identity")?, string("key")?).map_err(|e| e.to_string())?;
            *session = Some(created);
            Ok(json!({}))
        }
        "inbound" => {
            let created = account.create_inbound_session(message(command)?).map_err(|e| e.to_string())?;
            *session = Some(created.session);
            Ok(json!({ "plaintext": created.plaintext }))
        }
        "encrypt" => {
            let encrypted = current()?.encrypt(string("plaintext")?).map_err(|e| e.to_string())?;
            Ok(json!({ "type": encrypted.kind as u8, "body": encrypted.body }))
        }
        "decrypt" => {
            let plaintext = current()?.decrypt(message(command)?).map_err(|e| e.to_string())?;
            Ok(json!({ "plaintext": plaintext }))
        }
        op => Err(format!("unknown op {op}")),
    }
}

fn main() {
    let account = VodozemacAccount::new();
    let mut session = None;
    let mut stdout = io::stdout().lock();

    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin closed");
        let response = serde_json::from_str::<Value>(&line)
            .map_err(|e| e.to_string())
            .and_then(|command| run(&account, &mut session, &command))
            .unwrap_or_else(|error| json!({ "error": error }));

        writeln!(stdout, "{response}").expect("stdout closed");
        stdout.flush().expect("stdout closed");
    }
}
