# Findings: UI Overhaul + Pre-Mainnet Hardening

## Requirements (User Answers)

| Question | Answer |
|----------|--------|
| UI approach | Group sidebar + consolidate tabs (recommendation accepted) |
| Priority | UI overhaul first |
| Scope | Both UI and hardening |

## Pre-resolved Decisions

### UI Architecture
- **Sidebar**: Collapsible groups (6 sections), not a flat list
  - Rationale: All 18 items remain accessible, zero feature loss, immediate UX improvement
- **Escrow tabs**: 10 → 4 (My Escrows, Create, Lookup, Receipt, Invoice)
  - Milestones, Multi-party, Swaps become escrow "types" with adaptive detail views
  - "+ Create" tabs become deal type presets in the unified Create flow
- **No new UI libraries**: Framer Motion and Radix already installed. No Tailwind, no shadcn.
  - Rationale: Keep bundle lean, no tooling churn before mainnet
- **Empty states**: Already have `EmptyState` component — just need to use it more intentionally

### Current Navigation Audit (Sidebar.tsx:11-29)

| # | Item | Type | Proposed Group | Notes |
|---|------|------|---------------|-------|
| 1 | Dashboard | Core | 📊 Overview | |
| 2 | Offers | Trade | 🔄 Trade | |
| 3 | Escrows | Trade | 🔄 Trade | Main action |
| 4 | Swap | Trade | 🔄 Trade | Atomic swap wizard |
| 5 | Vaults | Finance | 🔒 Finance | |
| 6 | Subscriptions | Finance | 🔒 Finance | |
| 7 | Reputation | Community | 👥 Community | |
| 8 | Jury | Community | 👥 Community | |
| 9 | Blog | Resource | 📚 Resources | |
| 10 | Security | Resource | 📚 Resources | |
| 11 | Merchant | Advanced | ⚙️ Advanced | |
| 12 | Stats | Overview | 📊 Overview | |
| 13 | Docs | Resource | 📚 Resources | |
| 14 | Tokens | Advanced | ⚙️ Advanced | |
| 15 | Create Token | Advanced | ⚙️ Advanced | Move into Tokens page |
| 16 | Testnet | Resource | 📚 Resources | |
| 17 | Settings | Community | 👥 Community | |
| 18 | Help | Resource | 📚 Resources | |

### Current Escrow Tab Audit (EscrowsPage.tsx:30-40)

| # | Tab | Type | Target |
|---|-----|------|--------|
| 1 | My Escrows | Core | Keep |
| 2 | My Swaps | Filter | Remove — merge into My Escrows with type filter |
| 3 | Create | Core | Keep |
| 4 | Lookup | Core | Keep |
| 5 | Receipt | Core | Keep |
| 6 | Invoice | Core | Keep |
| 7 | Milestones | Filter | Remove — merge into My Escrows |
| 8 | + Milestone | Create variant | Remove — become preset in Create flow |
| 9 | Multi | Filter | Remove — merge into My Escrows |
| 10 | + Multi | Create variant | Remove — become preset in Create flow |

### Security Audit Remaining Items

| ID | Severity | Area | Status | Effort |
|----|----------|------|--------|--------|
| H1 | HIGH | Subscription covenant | ❌ Unfixed | 1 day |
| U7 | LOW | Onboarding modal | ❌ Polished | 1 day |
| Q4 | LOW | TradeHash type | ❌ Newtype | 1 day |
| H2 | LOW | Minimum fee check | ❌ Verify | ½ day |

## Architecture Notes

### Sidebar Collapse State
- Group expand/collapse state stored in React state (not localStorage)
- Default: Overview + Trade expanded, others collapsed
- On first visit, show all expanded with a brief "here's what's new" pulse animation

### Escrow Type Detection
Each escrow already has `asset_type`, `trade_hash`, milestone count, or party count that identifies its type. The detail view can detect and adapt:
- `escrow.trade_hash` exists → show atomic swap UI + preimage input
- `escrow.milestone_statuses` exists → show milestone progress bar
- `escrow.parties` exists → show multi-party signature board

No new backend fields needed.

### Quick-Action Buttons
Add a "Quick actions" section at the top of the sidebar (always visible):
- "Create Escrow" → navigates to /escrows?action=create
- "Deposit" → navigates to /vaults?action=deposit

These bypass the nav entirely for power users.

## Open Questions → Resolved
- Q: Should "Swap" remain as a separate sidebar item? → A: Yes — it's a distinct enough workflow (atomic swap wizard) to warrant its own nav item within Trade. Users looking for it will find it there.
- Q: Should we add a search bar? → A: No — premature optimization with only 18 items. Revisit when there are 50+.
- Q: How do we handle old URLs? → A: Add redirect in App.tsx. `/milestones` → `/escrows`, `/multi` → `/escrows`, `/swap/create` → `/escrows?action=create`.
