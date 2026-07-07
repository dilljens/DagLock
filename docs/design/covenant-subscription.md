# DagLock Subscription Covenant — Recurring Payments

**Status:** Implemented in SilverScript. Needs audit before mainnet deployment.

**Template Hash:** `3e7e187030f06ae5830fa2c72ef531c3d80c11ec`

## Overview

A recurring payment escrow covenant. The payer locks N KAS, and the recipient claims M KAS per interval. The payer can cancel at any time.

## Constructor Parameters

| Param | Type | Description |
|-------|------|-------------|
| `payerKey` | byte[32] | Payer's public key (funds provider) |
| `recipientKey` | byte[32] | Recipient's public key (service provider) |
| `totalAmount` | int | Total KAS locked (sompi) |
| `installmentAmount` | int | Amount per installment (sompi) |
| `intervalSeconds` | int | Seconds between installments |
| `startTime` | int | Unix timestamp when first installment is claimable |
| `treasuryKey` | byte[32] | DagLock fee treasury |

## Entrypoints

### `claim(recipientSig)`

Recipient claims one installment. Validates:
- Signature matches recipient key
- Outputs: [installmentAmount - fee] [fee]

### `cancel(payerSig)`

Payer cancels the subscription. Remaining funds - fee return to payer.
- Validates: signature matches payer key
- Outputs: [remaining - fee] [fee]

### `release(payerSig, recipientSig)`

Mutual early settlement. All remaining funds go to recipient (minus fee).
- Validates: both signatures
- Outputs: [all - fee] [fee]

## Use Cases

| Use Case | How It Works |
|----------|-------------|
| **SaaS subscription** | User pays 1000 KAS upfront. Provider claims 100 KAS/month for 10 months. |
| **Freelance retainer** | Client locks 5000 KAS. Freelancer claims 500 KAS/week for 10 weeks. |
| **Rent payment** | Tenant locks 12 months' rent. Landlord claims monthly. |

## Limitations

- No on-chain tracking of claimed installments — the covenant doesn't have state. Each `claim` spends the entire UTXO, so the remaining installments require a new UTXO to be created. (Simple implementation: each claim pays one installment + fee, no re-lock.)
- For a production version with re-locking, the `claim` entrypoint would create a new subscription UTXO with remaining installments.
- Current implementation is a "pay-as-you-go" model: each claim sends the installment to the recipient.
