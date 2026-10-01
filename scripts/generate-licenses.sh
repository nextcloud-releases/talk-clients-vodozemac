#!/bin/sh
# SPDX-FileCopyrightText: 2026 Nextcloud GmbH and Nextcloud contributors
# SPDX-License-Identifier: Apache-2.0

# Needs cargo-about: cargo install --locked cargo-about --features cli

set -eu
cd "$(dirname "$0")/.."

cargo about generate --locked --offline --fail about.hbs --output-file THIRD_PARTY_LICENSES.md
