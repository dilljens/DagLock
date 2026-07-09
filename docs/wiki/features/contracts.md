# contracts

12 SilverScript covenants compiled via `silverscript-lang`. Rusty-kaspa SDK v2.0.1 (Toccata). The `lib.rs` crate provides Rust API for compilation and template hash extraction.

## Covenant Inventory

| Covenant | File | Entrypoints | Purpose |
|----------|------|-------------|---------|
| KAS Escrow | `daglock.sil` | 6 | Release, split, swap, refund, emergency_refund, auto_settle. Core escrow with mutual settlement, atomic swaps, timeout refund, and no-sig emergency paths. MIN_OUT=1000 dust protection. |
| KRC-20 Escrow | `daglock_krc20.sil` | 4 | Release, swap, refund, auto_settle. ICC pattern for KCC-20 token ownership validation. |
| Arbiter | `daglock_arbiter.sil` | 7 | Release, swap, refundAfterTimeout, disputeSellerWins, disputeBuyerWins, arbitrateSplit, emergencyRefund. Mediated dispute resolution with proportional split. |
| Advanced | `daglock_advanced.sil` | 8 | Release, swap, swap_partial, extendTimeout, refund, auto_settle, split, emergency_refund. Time extension and partial atomic swaps. |
| Vault | `daglock_vault.sil` | 5 | Withdraw, sweep, relock (check-in), early_exit (cancel), heir_withdraw (inheritance). Dual-key model with DAA-block maturity. |
| Vault Multisig | `daglock_vault_multisig.sil` | 2 | Withdraw (2 key threshold), sweep. 2-of-3 multisig vault. |
| Vault Softlock | `daglock_vault_softlock.sil` | 2 | Withdraw password, withdraw timeout. Password-recoverable vault. |
| Milestone | `daglock_milestone.sil` | 5 | Release_milestone, approve_milestone, dispute, refund_remaining, complete. Up to 5 stages. |
| Subscription | `daglock_subscription.sil` | 3 | Claim (re-lock with currentPeriod+1), cancel, release. Timing-enforced installment draws. |
| Multi-Party | `daglock_multi.sil` | 3 | Release (all-party sig), swap (hash→party2), refund (buyer timeout). Up to 4 parties with basis-point shares. |
| Deposit | `daglock_deposit.sil` | 3 | Forfeit (jury sig + losingParty), release (both sigs), sweep (timeout). Security bonds. |
| Reputation | `daglock_reputation.sil` | 1 | Record trade. On-chain trade outcome recording. |

## Security Properties (Common Across All Covenants)

- **MIN_OUT = 1000**: All outputs must carry at least 1000 sompi (dust protection)
- **Destination validation**: No-signature paths hardcode output scripts to intended recipient (prevents third-party theft)
- **Fixed fee**: 0.5% (value/200) for escrows, 0.1% (value/1000) for vaults — hardcoded in covenant
- **No admin keys**: No "emergency withdraw" path that bypasses rules
- **Timeout safety**: Even if all parties disappear, emergency timeout returns funds

## Entrypoint Constants

All entrypoint name constants defined in `lib.rs::entrypoints` module for use by indexer and tests.
Added July 8: `CLAIM`, `CANCEL` (subscription), `SWAP_PARTIAL`, `EXTEND_TIMEOUT` (advanced).

## Test Files (July 8, 2026)

| Test file | Tests | Covers |
|-----------|-------|--------|
| `daglock_execution_tests.rs` | 21 | DagLock base + Advanced + Deposit + Multi |
| `daglock_arbiter_tests.rs` | 11 | Arbiter all entrypoints |
| `daglock_vault_tests.rs` | 17 | Vault + VaultMultisig + VaultSoftlock |
| `daglock_krc20_execution_tests.rs` | 9 | KRC-20 release/swap/refund |
| `daglock_krc20_tests.rs` | 6 | KRC-20 compilation + ABI + fee math |
| `daglock_milestone_tests.rs` | 7 | Milestone all 5 entrypoints |
| `daglock_subscription_tests.rs` | 6 | Subscription claim/cancel/release |
| `daglock_reputation_tests.rs` | 4 | Reputation record + template hash |
| **Total** | **104** | |

---
*Confidence: 0.95 · Last updated: 7/8/2026*