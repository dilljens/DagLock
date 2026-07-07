# DagLock Wallet — Architecture & Build Plan

> A Kaspa-native browser wallet with hardware signing support, purpose-built for DagLock escrows.
> Target: KasWare API-compatible, hardware-wallet-ready, covenant-aware.

---

## Table of Contents

1. [Why Build a DagLock Wallet](#1-why-build-a-daglock-wallet)
2. [Architecture Overview](#2-architecture-overview)
3. [Wallet API (KasWare Compatible)](#3-wallet-api-kasware-compatible)
4. [Software Wallet Build Plan (Phase 1)](#4-software-wallet-build-plan-phase-1)
5. [Hardware Wallet Integration (Phase 2)](#5-hardware-wallet-integration-phase-2)
6. [Covenant-Aware Display (Phase 3)](#6-covenant-aware-display-phase-3)
7. [Features That Matter](#7-features-that-matter)
8. [FAQ](#8-faq)

---

## 1. Why Build a DagLock Wallet

### The Gap

KasWare is the only Kaspa browser wallet. It's good, but it's generic — no DagLock awareness, no covenant display, no hardware wallet support. Users sign blind.

### What a DagLock Wallet Would Do Differently

| Feature | KasWare | DagLock Wallet |
|---------|---------|----------------|
| Covenant display | Raw hex | "Escrow: 5000 KAS settle" |
| Auth signing | Generic | Shows action + escrow ID |
| Hardware wallet | ❌ | Trezor / Ledger / Hybrid QR |
| Transaction history | Basic | Labels, search, export CSV |
| KRC-20 tokens | Basic | Native list + balances |
| Fee control | Default only | Custom fee rate + RBF |

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser Extension / Web App              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  window.daglockWallet  (KasWare-compatible API)       │  │
│  │  ├─ requestAccounts / getAccounts                     │  │
│  │  ├─ getBalance / getNetwork / getVersion               │  │
│  │  ├─ signMessage (schnorr)                             │  │
│  │  └─ sendKaspa (build + sign + broadcast tx)           │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                          │                                   │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │  Key Manager                                         │  │
│  │  ├─ Software: seed → BIP39 → BIP44 (m/44'/111111')  │  │
│  │  ├─ Hardware: transport abstraction                  │  │
│  │  └─ Watch-only: import pubkey only                   │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                          │                                   │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │  Transaction Builder                                 │  │
│  │  ├─ UTXO selection (coin selection)                  │  │
│  │  ├─ Fee estimation + custom fee rate                 │  │
│  │  ├─ Covenant output construction                     │  │
│  │  └─ Broadcast via wRPC                               │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Transport Abstraction (Hardware)

```
KeyManager
  │
  ├─ SoftwareKeySource
  │   └─ sign(hash) → sig          // seed-derived, in-memory
  │
  ├─ LedgerKeySource
  │   └─ sign(hash) → sig          // WebHID → Ledger device
  │
  ├─ TrezorKeySource
  │   └─ sign(hash) → sig          // TrezorConnect → WebUSB
  │
  └─ QRKeySource (hybrid mobile)
      └─ sign(hash) → sig          // QR → mobile app → hardware
```

---

## 3. Wallet API (KasWare Compatible)

This is the interface DagLock's web UI calls. Build this first for instant compatibility.

```typescript
// window.daglockWallet — injected into page
interface DaglockWalletProvider {
  // Connection
  requestAccounts(): Promise<string[]>;
  getAccounts(): Promise<string[]>;

  // Identity
  getPublicKey(): Promise<string>;            // x-only 32-byte hex
  getBalance(): Promise<{confirmed: number; pending: number}>;  // sompi
  getNetwork(): Promise<string>;              // "mainnet" | "testnet-11"

  // Signing
  signMessage(message: string, type?: "ecdsa" | "schnorr"): Promise<string>;
  sendKaspa(to: string, sompi: number, opts?: { feeRate?: number }): Promise<string>;

  // Metadata
  getVersion(): Promise<string>;

  // Events
  on(event: "accountsChanged" | "networkChanged" | "disconnect", handler: Function): void;
  removeListener(event: string, handler: Function): void;
}
```

**To integrate with DagLock:** Replace `window.kasware` check in `web/src/kasware.ts` with:

```typescript
declare global {
  interface Window {
    kasware?: KaswareProvider;
    daglockWallet?: KaswareProvider;  // fallback
  }
}

export async function getProvider() {
  return window.daglockWallet ?? window.kasware;
}
```

---

## 4. Software Wallet Build Plan (Phase 1)

**Duration: 2-3 weeks · 1 engineer**

### Week 1 — Key Management + UTXO

| Task | Files |
|------|-------|
| Seed generation (BIP39, 12/24 words) | `src/crypto/mnemonic.ts` |
| BIP44 derivation m/44'/111111' | `src/crypto/derivation.ts` |
| x-only public key → bech32m address | `src/crypto/address.ts` |
| secp256k1 Schnorr signing | `src/crypto/sign.ts` |
| UTXO fetch via wRPC | `src/rpc/utxos.ts` |
| UTXO store + cache | `src/storage/utxos.ts` |

### Week 2 — Transaction Building + DagLock Integration

| Task | Files |
|------|-------|
| Input selection (largest-first) | `src/tx/coinselect.ts` |
| Fee estimation | `src/tx/fees.ts` |
| Transaction serialization (borsh) | `src/tx/serialize.ts` |
| Broadcast via wRPC | `src/rpc/broadcast.ts` |
| Covenant output construction | `src/tx/covenant.ts` |
| `window.daglockWallet` API | `src/provider/index.ts` |
| End-to-end: create escrow flow | `src/flows/createEscrow.ts` |

### Week 3 — History + UX + Testing

| Task | Files |
|------|-------|
| Transaction history (local storage) | `src/storage/history.ts` |
| Address label + notes | `src/storage/labels.ts` |
| fiat price display (CoinGecko) | `src/prices.ts` |
| UI: popup, onboarding, settings | `src/ui/` |
| Extension packaging (Chrome) | `manifest.json` |
| Test against DagLock e2e | `tests/e2e.test.ts` |

### Key Libraries

| Package | Purpose |
|---------|---------|
| `kaspa-wasm` (npm) | Address derivation, tx building, Schnorr |
| `ws` (Node.js shim) | WebSocket for wRPC in Node |
| `idb-keyval` | Local storage (IndexedDB) |
| `zustand` | Lightweight state management |

---

## 5. Hardware Wallet Integration (Phase 2)

**Duration: 4-6 weeks · 1 embedded engineer + 1 frontend engineer**

### 5.1 Ledger (3-4 weeks)

```
┌──────────────┐     WebHID      ┌────────────┐
│  Browser      │ ───────────────▶│  Ledger     │
│  Extension    │ ◀───────────────│  Nano S/X   │
│  (WebHID API) │                  └────────────┘
└──────────────┘
```

**Implementation:**

```typescript
// transports/ledger.ts
import TransportWebHID from "@ledgerhq/hw-transport-webhid";
import { schnorrSign, getPublicKey } from "./ledger-kaspa-app";

export class LedgerKeySource {
  private transport: Transport;

  async connect() {
    this.transport = await TransportWebHID.request();
  }

  async getAddress(path: string): Promise<string> {
    const { publicKey } = await getPublicKey(this.transport, path);
    return pubkeyToAddress(publicKey, "kaspa");
  }

  async sign(hash: Uint8Array, path: string): Promise<Uint8Array> {
    return schnorrSign(this.transport, path, hash);
  }
}
```

**Files needed:**
| File | Purpose |
|------|---------|
| `src/hardware/ledger/transport.ts` | WebHID connection management |
| `src/hardware/ledger/kaspa-app.ts` | Kaspa-specific APDU commands |
| `src/hardware/ledger/derivation.ts` | Path derivation |
| `src/hardware/types.ts` | Common interface |

**Kaspa Ledger App (C firmware):**
- Fork the Bitcoin app from `ledger/app-bitcoin-new`
- Replace curve: secp256k1 Schnorr (not ECDSA)
- Replace derivation: `m/44'/111111'` (Kaspa coin type)
- Screen: show address + amount + "Sign transaction?"
- Build with `pixie` / `clang` for STM32 (Ledger's SDK)

**Build & Deploy:**
```bash
git clone https://github.com/LedgerHQ/app-bitcoin-new
# Modify for Kaspa — curve, paths, screen text
# Build with Ledger's SDK
make
# Test with Speculos
docker run --rm -v $(pwd)/app.elf:/app.elf speculos:latest /app.elf
# Submit for Ledger review (2-3 months queue)
```

### 5.2 Trezor (2-3 weeks)

```typescript
// transports/trezor.ts
import TrezorConnect from "@trezor/connect";

export class TrezorKeySource {
  async getAddress(path: string): Promise<string> {
    const r = await TrezorConnect.getAddress({
      path: `m/44'/111111'/0'/0/0`,
      coin: "kaspa",
      showOnTrezor: true,
    });
    return r.payload.address;
  }

  async sign(hash: Uint8Array, path: string): Promise<Uint8Array> {
    // For Trezor without native Kaspa app: use signMessage
    const r = await TrezorConnect.signMessage({
      path,
      message: Buffer.from(hash).toString("hex"),
      coin: "kaspa",
    });
    return Buffer.from(r.payload.signature, "hex");
  }
}
```

**Limitations without a Kaspa Trezor app:**
- `signMessage` works today (auth: settle/refund/vouch)
- Full tx signing needs custom firmware (wait for community)

**Approach:** Ship auth-only first, add full tx signing when Trezor releases a Kaspa app.

### 5.3 Hybrid Mobile QR (Solves Both)

```
Desktop (no USB/BT)           Mobile (has USB/BT)        
┌─────────────────────┐       ┌─────────────────────┐    
│ DagLock Web UI      │       │ Your Companion App   │    
│ → Shows unsigned QR │ ◄───▶ │ ← Scans QR          │    
│ ← User types sig    │       │ → Signs via HW       │    
│ → Broadcasts tx     │       │ → Returns signed QR  │    
└─────────────────────┘       └─────────────────────┘    
```

This avoids needing Trezor/Ledger firmware entirely. The mobile companion app handles the hardware connection (USB-C/Bluetooth on phone is native), and the desktop gets the result via QR or copy-paste.

**Build:**
| Platform | Framework | Effort |
|----------|-----------|--------|
| iOS | SwiftUI + CoreBluetooth | 2-3 weeks |
| Android | Kotlin + USB serial | 2-3 weeks |
| Web/pwa | Not needed — desktop can't do USB/BT to phone | N/A |

---

## 6. Covenant-Aware Display (Phase 3)

**Duration: 1 week**

When a user signs a DagLock action, show them what they're actually signing:

```typescript
// src/parsers/covenant.ts
export function parseDaglockCovenant(txHex: string): DaglockAction | null {
  // Detect DagLock template hash in output script
  // Extract: action type (release/swap/refund), amount, buyer, seller, fee
  // Return parsed object for screen display
}

// Display on screen:
// ┌──────────────────────────────────────┐
// │  💠 DagLock Action                  │
// │                                      │
// │  Settle Escrow esc_abc123            │
// │  5000 KAS → seller                   │
// │  Fee: 25 KAS (0.5%)                 │
// │                                      │
// │  [Cancel]  [Confirm with Ledger]     │
// └──────────────────────────────────────┘
```

**Template hashes to detect:**
| Covenant | Template Hash |
|----------|--------------|
| KAS escrow | `30876e3ea42d0e23bb0980f3fd97ae8807e9c70f` |
| Arbiter | `d6aea010040d361049483c62da2e6b35f6dc256c` |
| KRC-20 | `ae0946e4a9bd4a7585e6bf9135de38083cb11c85` |
| Vault | `b338c514b1ef79bf1b0739814bc0d567e8461cfb` |
| Softlock | `ed57b9da957beaac387a0baa9a23c8c54d186964` |
| Multisig | `caf0b46ea425159b80af81436fc8f8cfd4e62afa` |

---

## 7. Features That Matter

### Priority 1 — Must Have (Phase 1)
- [ ] Schnorr x-only signing
- [ ] BIP44 derivation `m/44'/111111'`
- [ ] `window.daglockWallet` API (KasWare compatible)
- [ ] UTXO management + coin selection
- [ ] Transaction history + labels
- [ ] Fee estimation
- [ ] Testnet/mainnet switch

### Priority 2 — Hardware Support (Phase 2)
- [ ] Ledger WebHID transport
- [ ] Trezor Connect (auth-only)
- [ ] Hybrid QR mobile bridge
- [ ] Covenant-aware display

### Priority 3 — Polish (Post-Launch)
- [ ] KRC-20 token list
- [ ] Fiat price display
- [ ] Watch-only accounts
- [ ] Multi-account (BIP44 accounts)
- [ ] CSV export (tax)
- [ ] UTXO consolidation

---

## 8. FAQ

### Q: Is KasWare required for DagLock to work?

No. DagLock checks `window.kasware`, but any provider with the same interface works. You can build `window.daglockWallet` and the web UI will use it.

### Q: Can I build this without a hardware wallet?

Yes. Phase 1 (software wallet) works standalone. Hardware support is Phase 2. Ship the software wallet first, add hardware when users ask.

### Q: Do I need to write C firmware for Trezor?

For full tx signing, yes. For auth-only (settle/refund/dispute), `TrezorConnect.signMessage` works today.

### Q: How long does Ledger/Trezor review take?

| Platform | Review Time | Notes |
|----------|------------|-------|
| Ledger | 2-3 months | Must provide source + docs |
| Trezor | 1-2 months | Community-driven, open PR |

### Q: Can I skip the firmware and use hybrid mobile?

Yes. Hybrid QR is the fastest path to hardware support — no firmware, no review queue. Build a mobile companion app + QR bridge, ship in 2-3 weeks.

### Q: What's the budget for an embedded engineer?

| Role | Rate | Duration | Total |
|------|------|----------|-------|
| Rust/TS full-stack dev | $80-150/hr | 3 weeks | $10-30k |
| Embedded C dev (firmware) | $100-200/hr | 4 weeks | $15-35k |
| Total | | 7 weeks | $25-65k |

### Q: Where does this integrate with DagLock?

```typescript
// web/src/kasware.ts — one change:
export async function getProvider() {
  return window.daglockWallet ?? window.kasware;
}
```

Everything else in the DagLock web UI works automatically.

---

## Appendix: Quick Start

```bash
# 1. Scaffold extension
npx create-chrome-extension daglock-wallet --template typescript
cd daglock-wallet

# 2. Install crypto deps
npm install kaspa-wasm kaspa-wrpc-client

# 3. Implement the provider
cat > src/provider/index.ts << PROVIDER_EOF
import { mnemonicToSeed, deriveKey, signSchnorr } from "kaspa-wasm";

let state = { seed: null, accounts: [] };

export const provider = {
  requestAccounts: async () => {
    if (!state.seed) state.seed = await mnemonicToSeed(prompt("Seed phrase:"));
    const key = deriveKey(state.seed, "m/44'/111111'/0'/0/0");
    state.accounts = [key.address];
    return state.accounts;
  },
  signMessage: async (msg) => {
    if (!state.seed) throw new Error("Not connected");
    return signSchnorr(state.seed, msg);
  },
  // ... rest of API
};
PROVIDER_EOF

# 4. Inject into page
cat > src/content.ts << CONTENT_EOF
import { provider } from "./provider";
window.daglockWallet = provider;
CONTENT_EOF

# 5. Load unpacked extension in Chrome → daglock.io works
```
