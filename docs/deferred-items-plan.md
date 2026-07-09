# Deferred Items — Implementation Plan

> Deferred during July 8 audit sprint. These items need external access, infrastructure changes, or are larger efforts.
>
> **Status:** Implementation started July 8, 2026. Items with no external dependencies are being actively worked.

---

## Status Overview

| Priority | Item | Status | Who |
|----------|------|--------|-----|
| **P1** | Rotate secrets (C1/C3) | ❌ Blocked — needs Cloudflare dashboard | User |
| **P2** | Bot persistent storage (H3) | ✅ Done — SQLite migration completed | Agent |
| **P3** | Subscription covenant tests (C5) | ✅ Done | Agent |
| **P4** | Chat sig verification (H4) | ❌ Blocked — needs wRPC node | User |
| **P5** | Milestone/Advanced covenant tests (C5) | ✅ Done | Agent |
| **P6** | Bot test suite (C6) | ✅ Done — index.js command handler tests | Agent |

---

## 1. Rotate Cloudflare API Token & Encryption Key (P1)

**Tracking codes:** C1/C3 (project-internal tracking, unrelated to security audit C1/C3)

**Problem:** Two secrets are stored in plaintext on disk:
- `.env.cloudflare` — `CLOUDFLARE_API_TOKEN` (live CF API token)
- `.env` — `DAGLOCK_MESSAGE_KEY` (AES-256-GCM encryption key for chat messages)

Both are gitignored but live unencrypted on the filesystem. If the VPS is compromised, these secrets leak.

**Effort:** 30 minutes

**Dependency:** You need Cloudflare dashboard access.

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

## 2. Bot Persistent Storage Migration (P2) ✅ Done

**Tracking code:** H3 (project-internal tracking, unrelated to audit H3)

**Problem:** User data (`/tmp/daglock-users.json`) was stored at `/tmp/` which is wiped on system restart. Every VPS reboot forced all bot users to re-register their addresses.

**Solution:** Migrated to SQLite via `better-sqlite3`.

**What was done:**
1. Added `better-sqlite3` dependency to `bot/package.json`
2. Created `bot/src/db.js` — SQLite wrapper with two tables:
   - `users` (telegram_id, address, created_at)
   - `conversations` (telegram_id, conv_data, updated_at)
3. Replaced `loadUsers()`/`saveUsers()` in `bot/src/index.js` with SQLite queries
4. Added migration from existing `/tmp/daglock-users.json` on startup
5. Encryption-at-rest preserved (addresses still encrypted via `BOT_ENCRYPTION_KEY`)
6. Systemd service updated with `BOT_DB_PATH=/opt/daglock-bot/bot.db`

**Effort:** 2-3 hours

---

## 3. Chat Signature Verification (P4)

**Tracking code:** H4

**Problem:** `--mock-chat-sig` is enabled in production. Anyone can forge chat messages on any escrow. The code has a `WARN` log:
```rust
tracing::warn!("chat_sig verification not yet implemented — accepting with --mock-chat-sig={}", state.mock_chat_sig);
```

**Effort:** 1-2 days

**Dependency:** Real wRPC node (needs Kaspa node for signature verification) — blocked until local testnet node is set up.

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

## 4. Covenant Test Coverage (P3/P5) ✅ Done

**Tracking code:** C5

**Initial problem statement (outdated):** 7 of 13 covenant `.sil` files have zero execution tests.

**Correction (July 8):** Only **3** of 13 covenants were untested. The following already had execution tests:

| Covenant | Test file |
|----------|-----------|
| `daglock.sil` | `daglock_execution_tests.rs` (742 lines) |
| `daglock_krc20.sil` | `daglock_krc20_execution_tests.rs` (418 lines) + `daglock_krc20_tests.rs` (185 lines) |
| `daglock_arbiter.sil` | `daglock_arbiter_tests.rs` (981 lines) |
| `daglock_reputation.sil` | `daglock_reputation_tests.rs` (193 lines) |
| `daglock_vault.sil` | `daglock_vault_tests.rs` (895 lines) |
| `daglock_vault_multisig.sil` | `daglock_vault_tests.rs` (lines 748+) |
| `daglock_vault_softlock.sil` | `daglock_vault_tests.rs` (lines 284+) |
| `daglock_deposit.sil` | `daglock_execution_tests.rs` (line 667) |
| `daglock_multi.sil` | `daglock_execution_tests.rs` (line 720) |

**Untested (all now completed):**
1. `daglock_subscription.sil` — P3, High priority
2. `daglock_milestone.sil` — P5, Medium priority
3. `daglock_advanced.sil` — P5, Low priority

**Tests added:**
- `contracts/tests/daglock_subscription_tests.rs` — New file: subscription claim timing, fee calculation, re-lock behavior
- `contracts/tests/daglock_milestone_tests.rs` — New file: milestone release, partial release, full release
- Extended `daglock_execution_tests.rs` — Added `daglock_advanced.sil` release/swap/refund path tests

**Test pattern** (follows existing conventions):
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

## 5. Bot Test Suite (P6) ✅ Done

**Tracking code:** C6

**Initial problem statement (outdated):** 3,073 lines of bot code with zero tests.

**Correction (July 8):** Bot already had tests for library modules:
- `bot/src/lib/api.test.js` (206 lines) — API client with mocked fetch responses
- `bot/src/crypto.test.js` (105 lines) — Encryption/decryption round-trips

**Gap filled:** `bot/src/index.js` (3,058 lines) — the main command handler file — had zero tests.

**Tests added:**
- `bot/tests/unit/commands.test.js` — Tests for key command handlers:
  - `/setaddress` with valid/invalid addresses
  - `/create` wizard flow (conversation state machine)
  - `/settle` with mock API response
  - `/cancel` with invalid escrow ID
  - `/status` with active/completed escrow
  - Helper functions (`getUserAddress`, `setUserAddress`, conversation state)
- `bot/tests/unit/db.test.js` — SQLite-backed address storage CRUD

**Key test scenarios covered:**
| Test | What it covers |
|------|---------------|
| `/setaddress` with valid address | Address storage via SQLite |
| `/setaddress` with invalid address | Validation rejection |
| `/create` wizard flow | Conversation state machine with grammY sessions |
| `/settle` with mock API response | Escrow lifecycle dispatch |
| `/cancel` with invalid ID | Error handling |
| `getUserAddress` / `setUserAddress` | SQLite CRUD |
| Conversation start/advance/end | State machine helpers |
| DB migration from JSON | Legacy data migration |

**Test structure:**
```
bot/
├── tests/
│   ├── unit/
│   │   ├── commands.test.js    — Command handler tests
│   │   └── db.test.js          — SQLite CRUD + migration tests
│   └── integration/
│       └── lifecycle.test.js   — (future: requires test API)
├── src/
│   ├── index.js
│   ├── db.js                   — SQLite module
│   └── lib/
│       ├── api.js
│       └── api.test.js         — Pre-existing
```

---

## Future (After wRPC Node is Available)

Once the local testnet node is set up (see `docs/local-testnet-node.md`), the `--mock-chat-sig` flag can be removed and real signature verification enabled. This unlocks P4.
