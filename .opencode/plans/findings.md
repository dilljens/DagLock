# UI/UX Findings

## Tool Stack Evaluation

| Tool | Current | Verdict | Action |
|------|---------|---------|--------|
| React 18 + Vite 7 | ✅ In use | Good for SPA | Keep |
| Custom CSS (1280 lines) | 🟡 Single file | Works, needs splitting | Split into CSS modules |
| Tailwind CSS | ❌ Not used | 2-week migration, no functional gain | **Skip** |
| No component library | 🔴 Hurts a11y | Dialog, Tabs, Tooltip needed | Add Radix UI |
| No animation library | 🔴 Static transitions | Micro-interactions add polish | Add Framer Motion (`motion`) |
| Manual data fetching | 🔴 No caching, no re-fetch | Every page re-fetches on mount | Add TanStack Query |
| Manual form validation | 🔴 Repeated boilerplate | Each form re-implements the same patterns | Add React Hook Form + Zod |
| Custom hash router | 🟡 Works fine | Not worth changing | Keep |
| Biome linting | ✅ Fast, good | Best-in-class | Keep |
| Vitest + RTL | ✅ Testing | Good setup | Keep |

## Design Decisions

### Color Theme: Dark Green (Keep and expand)
The current `#0a1a0a` background with `#53d769` accent is thematically correct for Kaspa. Expand with:
- Semantic colors (success/error/info/warning) already exist but aren't used consistently
- Add text hierarchy (primary/secondary/muted) with specific values
- Add elevation colors for modal overlays

### Typography: Inter (Keep)
Inter is the correct choice for DeFi UIs (used by Uniswap, Aave, etc.). Already in use.

### Empty States
Current "No offers found" text is unhelpful. Each empty state needs:
1. Icon/illustration
2. Friendly heading
3. Explanation text
4. Clear call-to-action button

### Skeleton Loading
Use animated placeholder shapes that mirror the final layout. Not just spinners. Not just "Loading...".
Reference: react-loading-skeleton library pattern, but custom CSS to match our design tokens.

### Transaction Feedback
Users need confirmation that their wallet action succeeded. Current toast-only approach misses the opportunity for a celebratory moment. Brief (2s) animation after successful settle/create/refund.

### Onboarding
First-time users on `daglock.com` see a hero with "Connect Wallet". No guidance on what DagLock is or what to do next. A 4-step wizard on first visit would dramatically improve conversion.

## Bundle Impact

Adding 6 packages adds ~80KB gzipped. Current JS bundle: 205KB → ~285KB total. Acceptable for the UX improvement.

## What NOT to Do
- Don't rewrite in Tailwind (2 weeks, no functional gain)
- Don't add react-router (custom hash router works fine)
- Don't add heavy component library (MUI, Chakra, Ant Design)
- Don't add SSR/Next.js (SPA is correct architecture for wallet apps)
- Don't add i18n (English-only is sufficient)
