# Deferred Items — Implementation Plan

> Deferred during July 8 audit sprint. These items need external access, infrastructure changes, or are larger efforts.

---

## 1. Rotate Cloudflare API Token & Encryption Key (C1/C3)

**Problem:** Two secrets are stored in plaintext on disk:
- `.env.cloudflare` — `CLOUDFLARE_API_TOKEN` (live CF API token)
- `.env` — `DAGLOCK_MESSAGE_KEY` (AES-256-GCM encryption key for chat messages)

Both are gitignored but live unencrypted on the filesystem. If the VPS is compromised, these secrets leak.

**Effort:** 30 minutes

**Steps:**

1. **Rotate the Cloudflare token:**
   - Go to https://dash.cloudflare.com/profile/api-tokens
   - Find the current token (starts with `cfut_` in `.env.cloudflare`)
   - Revoke it and create a new one with the same permissions:
     - Account scope: Pages → Read + Write (for the daglock project)
     - Zone scope: None (or daglock.com if needed)
   - Update `.env.cloudflare` with the new token:
     ```bash
     echo "CLOUDFLARE_API_TOKEN=<new-token>" > .env.cloudflare
     ```

2. **Rotate the message encryption key:**
   - Generate a new key:
     ```bash
     openssl rand -hex 32
     ```
   - Update `.env` with the new key:
     ```bash
     # Replace DAGLOCK_MESSAGE_KEY value in .env
     ```
   - **Note:** Rotating this key will make all existing encrypted chat messages undecryptable (they were encrypted with the old key). Only rotate between trading sessions or during low activity.

3. **Add to CI secret:**
   ```bash
   gh secret set CLOUDFLARE_API_TOKEN --repo dilljens/DagLock --body "<new-token>"
   ```

---

## 2. Bot Persistent Storage Migration (H3)

**Problem:** User data (`/tmp/daglock-users.json`) is stored at `/tmp/` which is wiped on system restart. Every VPS reboot forces all bot users to re-register their addresses.

**Effort:** 2-3 hours

**Options:**

### Option A: SQLite (Recommended)
Move from JSON file to SQLite database alongside the indexer's DB.

**Steps:**
1. Add `better-sqlite3` or `sql.js` dependency to `bot/package.json`:
   ```bash
   cd bot && npm install better-sqlite3
   ```
2. Create a `bot/src/db.js` module:
   ```javascript
   const Database = require('better-sqlite3');
   const path = require('path');
   
   const DB_PATH = process.env.BOT_DB_PATH || '/opt/daglock-bot/bot.db';
   const db = new Database(DB_PATH);
   
   db.exec(`
     CREATE TABLE IF NOT EXISTS users (
       telegram_id INTEGER PRIMARY KEY,
       address TEXT NOT NULL,
       created_at INTEGER NOT NULL
     );
     CREATE TABLE IF NOT EXISTS conversations (
       telegram_id INTEGER PRIMARY KEY,
       conv_data TEXT,
       updated_at INTEGER NOT NULL
     );
   `);
   
   module.exports = db;
   ```
3. Replace `loadUsers()`/`saveUsers()` with db queries:
   ```javascript
   const db = require('./db');
   
   function getUserAddress(telegramId) {
     const row = db.prepare('SELECT address FROM users WHERE telegram_id = ?').get(telegramId);
     return row ? row.address : null;
   }
   
   function setUserAddress(telegramId, address) {
     db.prepare('INSERT OR REPLACE INTO users (telegram_id, address, created_at) VALUES (?, ?, ?)')
       .run(telegramId, address, Math.floor(Date.now() / 1000));
   }
   ```
4. Migrate existing data:
   ```javascript
   function migrateFromJson() {
     const fs = require('fs');
     const jsonPath = '/tmp/daglock-users.json';
     if (!fs.existsSync(jsonPath)) return;
     const data = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));
     const insert = db.prepare('INSERT OR REPLACE INTO users (telegram_id, address, created_at) VALUES (?, ?, ?)');
     for (const [id, addr] of Object.entries(data.users || {})) {
       insert.run(parseInt(id), addr, Math.floor(Date.now() / 1000));
     }
     const convInsert = db.prepare('INSERT OR REPLACE INTO conversations (telegram_id, conv_data, updated_at) VALUES (?, ?, ?)');
     for (const [id, conv] of Object.entries(data.conversations || {})) {
       convInsert.run(parseInt(id), JSON.stringify(conv), Math.floor(Date.now() / 1000));
     }
     // Backup and remove old file
     fs.renameSync(jsonPath, jsonPath + '.bak');
   }
   ```

### Option B: Move JSON to Persistent Path
Simpler but less robust. Change the file path:
```javascript
const DB_PATH = process.env.BOT_DATA_DIR || '/opt/daglock-bot/data';
```
Then ensure the directory exists and is not in `/tmp/`.

**Files to modify:**
| File | Change |
|------|--------|
| `bot/src/index.js` | Replace `loadUsers`/`saveUsers` functions |
| `bot/src/db.js` | **New** — SQLite wrapper |
| `bot/package.json` | Add `better-sqlite3` dependency |
| `bot/Dockerfile` or systemd service | Set `BOT_DB_PATH` env var |

---

## 3. Chat Signature Verification (H4)

**Problem:** `--mock-chat-sig` is enabled in production. Anyone can forge chat messages on any escrow. The code has a `WARN` log:
```rust
tracing::warn!("chat_sig verification not yet implemented — accepting with --mock-chat-sig={}", state.mock_chat_sig);
```

**Effort:** 1-2 days

**Dependency:** Real wRPC node (needs Kaspa node for signature verification)

**Steps:**

1. **Implement chat signature verification in `messages.rs`:**
   The `send_message` handler currently accepts `body.sender_pubkey` as an optional string and doesn't verify it against the escrow's registered chat pubkeys.
   
   ```rust
   // In indexer/src/api/messages.rs, before inserting the message:
   if !state.mock_chat_sig {
       // Verify the sender's signature proves they own the claimed pubkey
       let escrow = queries::get_escrow(&state.db, &escrow_id)
           .await
           .map_err(|_e| internal_error())?
           .ok_or_else(|| not_found("escrow", &escrow_id))?;
       
       // Check the signature matches either buyer or seller
       // Message format: "daglock:chat:{escrow_id}:{sender_pubkey}:{nonce}"
       let expected_msg = format!("daglock:chat:{}:{}:{}", escrow_id, sender_pubkey, nonce);
       
       if !state.sig_verifier.verify_signature(&auth.address, &body.signature, &expected_msg)
           .map_err(|e| bad_request("sig_error", &e.to_string()))? {
           return Err(forbidden("invalid_signature", "Chat signature does not match sender"));
       }
       
       // Verify the pubkey is registered for this escrow
       let is_authorized = escrow.chat_pubkey_buyer.as_deref() == Some(&sender_pubkey)
           || escrow.chat_pubkey_seller.as_deref() == Some(&sender_pubkey);
       
       if !is_authorized {
           return Err(forbidden("unauthorized_pubkey", "Sender pubkey not registered for this escrow"));
       }
   }
   ```

2. **Add `sender_pubkey` and `signature` to `SendMessageRequest`** in types.rs

3. **Update the web UI** (`ChatPanel.tsx`) to include the sender's chat pubkey and a signature in message sends

4. **Remove `--mock-chat-sig` flag** once tested

**Files to modify:**
| File | Change |
|------|--------|
| `indexer/src/api/messages.rs` | Add signature verification |
| `indexer/src/types.rs` | Add `sender_pubkey`, `signature` to request |
| `web/src/components/ChatPanel.tsx` | Include pubkey + signature in sends |
| `indexer/src/config.rs` | Remove `mock_chat_sig` flag |

---

## 4. Covenant Test Coverage (C5)

**Problem:** 7 of 13 covenant `.sil` files have zero execution tests:
- `daglock_advanced.sil`
- `daglock_deposit.sil`
- `daglock_milestone.sil`
- `daglock_multi.sil`
- `daglock_subscription.sil`
- `daglock_vault_multisig.sil`
- `daglock_vault_softlock.sil`

**Effort:** 3-5 days total (spread across covenants)

**Priority order (by user-facing impact):**
1. `daglock_subscription.sil` — High (used by subscription flow)
2. `daglock_multi.sil` — High (multi-party escrows)
3. `daglock_milestone.sil` — Medium (milestone escrows)
4. `daglock_vault_multisig.sil` — Medium (multisig vaults)
5. `daglock_vault_softlock.sil` — Medium (softlock vaults)
6. `daglock_deposit.sil` — Low (security deposit, UI not fully wired)
7. `daglock_advanced.sil` — Low (advanced escrow features)

**Test pattern** (follow existing `daglock_execution_tests.rs`):
```rust
#[test]
fn subscription_claim_succeeds_after_interval() {
    let (mut session, _) = create_session();
    let covenant = compile_daglock_subscription(
        &payer_key, &recipient_key, TOTAL, INSTALLMENT,
        INTERVAL, START_TIME, 0, &treasury_key
    );
    // Fund the covenant
    session.create_output(100_000_000, &covenant, None);
    // Claim first installment
    let result = session.execute_entrypoint("claim", 
        vec![recipient_sig.clone(), covenant_change_script]);
    assert!(result.success);
    // Verify outputs: recipient gets installment - fee, treasury gets fee
    assert_eq!(result.outputs[0].value, INSTALLMENT - INSTALLMENT / 200);
}
```

---

## 5. Bot Test Suite (C6)

**Problem:** 3,073 lines of bot code with zero tests. The bot handles user funds, escrow operations, and wallet address management.

**Effort:** 2-3 days

**Approach:** Unit tests for command handlers + integration test for API client.

**Test structure:**
```
bot/
├── tests/
│   ├── unit/
│   │   ├── commands.test.js    — Test individual command handlers
│   │   └── api.test.js         — Test API client with mocked responses
│   └── integration/
│       └── lifecycle.test.js   — Full bot flow (requires test API)
├── src/
│   ├── index.js
│   └── lib/
│       └── api.js
```

**Key test scenarios:**
| Test | What it covers |
|------|---------------|
| `/setaddress` with valid address | Address storage |
| `/setaddress` with invalid address | Validation |
| `/create` wizard flow | Conversation state machine |
| `/settle` with mock API response | Escrow lifecycle |
| `/cancel` with invalid ID | Error handling |
| API retry/backoff | Network resilience |
| API client error parsing | Error messages |

---

## Priority Summary

| Priority | Item | Effort | Dependency |
|----------|------|--------|-----------|
| **P1** | Rotate secrets (C1/C3) | 30 min | Cloudflare dashboard access |
| **P2** | Bot persistent storage (H3) | 2-3 hrs | None |
| **P3** | Subscription covenant tests (C5) | 1 day | None |
| **P4** | Chat sig verification (H4) | 1-2 days | wRPC node |
| **P5** | Multi/Milestone covenant tests (C5) | 2-3 days | None |
| **P6** | Bot test suite (C6) | 2-3 days | None |
