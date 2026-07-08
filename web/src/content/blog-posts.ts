// Blog post data — each post is a simple object with slug, title, date, excerpt, content.
// Content is markdown-like (HTML subset). No external rendering library needed.

export type BlogPost = {
	slug: string;
	title: string;
	date: string;
	excerpt: string;
	content: string;
};

export const BLOG_POSTS: BlogPost[] = [
	{
		slug: "krc20-escrow",
		title: "KRC-20 Token Escrow is Here",
		date: "July 7, 2026",
		excerpt:
			"DagLock now supports escrow for KRC-20 tokens using native SilverScript covenants — the first platform on Kaspa to offer this.",
		content: `
<p>If you've traded KRC-20 tokens on Kaspa, you know the pain: find a buyer, agree on terms, then trust a chat-group guarantor with your money. It works — until it doesn't. Guarantors charge 3-10%, hold your funds, and their honesty is the only thing protecting you.</p>

<p>Today, there's another option.</p>

<h2>KRC-20 Escrow, Native on Kaspa</h2>

<p>DagLock now supports escrow for KRC-20 tokens using the same SilverScript covenant model as our KAS escrow. The difference? The covenant validates token ownership directly through Kaspa's Inter-Covenant Communication (ICC) pattern — no wrappers, no bridges, no trusted third parties.</p>

<p>When you create a KRC-20 escrow on DagLock:</p>

<ul>
  <li>The <strong>KRC-20 tokens</strong> are held by the KCC-20 covenant under DagLock's control</li>
  <li>The <strong>KAS fee</strong> goes to the DagLock treasury covenant</li>
  <li>Both are governed by the same SilverScript spending rules: release (both sign), swap (hash preimage), refund (timeout), or auto-settle (post-timeout)</li>
  <li>The ICC pattern means the covenant verifies its own ownership of the tokens at spend time — it cannot be tricked into releasing tokens it doesn't control</li>
</ul>

<h2>What This Enables</h2>

<table>
  <tr><th>Use Case</th><th>How It Works</th></tr>
  <tr><td>OTC token trades</td><td>Lock tokens in escrow, buyer sends KAS, both sign to release</td></tr>
  <tr><td>Token-for-token swaps</td><td>Atomic swap with hash preimage — neither side can cheat</td></tr>
  <tr><td>Milestone-based token sales</td><td>Release tokens in stages as milestones are met</td></tr>
  <tr><td>Subscription distributions</td><td>Pre-fund and let recipients claim periodically</td></tr>
</table>

<h2>The Fee Model</h2>

<p>The KRC-20 escrow uses the same 0.5% protocol fee as our KAS escrow. The fee is deducted from the KAS side of the transaction (not the tokens). On refund, no fee is charged.</p>

<h2>Security</h2>

<p>The KRC-20 covenant underwent internal security review alongside the rest of DagLock's codebase. The ICC validation ensures that the covenant can only spend KCC-20 branches it actually owns. A misconfigured covenant with zero ICC parameters is rejected at compile time.</p>

<h2>Getting Started</h2>

<ol>
  <li><strong>Deploy a KRC-20 token</strong> or use an existing one on testnet</li>
  <li><strong>Create an escrow</strong> with <code>asset_type: "KRC20:YOUR_TICKER"</code></li>
  <li><strong>Share the link</strong> with your counterparty</li>
  <li><strong>Settle</strong> when both parties agree — or dispute if something's wrong</li>
</ol>

<p>The compile API now supports KRC-20 covenants at <code>POST /v1/compile</code> with <code>template: "daglock_krc20"</code>. All constructor parameters are accepted, including optional KCC-20 ICC metadata for production use.</p>

<p style="margin-top: 24px; font-size: 13px; color: #888;">
  DagLock is open source and has undergone internal security review. No admin keys. No backdoors. The covenant enforces the rules — not us.
</p>`,
	},
	{
		slug: "ai-mediation",
		title: "AI Mediation for Escrow Disputes",
		date: "July 7, 2026",
		excerpt:
			"Our AI mediator resolves escrow disputes in minutes by reading encrypted chat evidence and proposing fair outcomes — before a human jury is ever needed.",
		content: `
<p>Escrow disputes are the worst part of P2P trading. Someone doesn't deliver. The chat logs are ambiguous. Now you need a third party to sort it out — and that takes days, costs money, and requires trusting someone with your evidence.</p>

<p>We built a better way.</p>

<h2>AI Mediation, Built Into the Dispute Flow</h2>

<p>When a DagLock escrow is disputed, the flow is:</p>

<ol>
  <li><strong>Amicable negotiation</strong> — Both parties can still release or refund at any time during the dispute. Most issues resolve here.</li>
  <li><strong>AI mediator</strong> — One party reveals the encrypted chat to the AI. The AI reads the evidence, analyzes both sides' claims, and proposes a fair outcome within minutes.</li>
  <li><strong>Human jury</strong> — If neither party accepts the AI's recommendation within 24 hours, a randomly selected jury votes on the outcome.</li>
</ol>

<p>The AI step is optional, non-binding, and free. It's designed to resolve the 80% of disputes that don't need a human — quickly and fairly.</p>

<h2>Privacy First</h2>

<p>The deal chat is end-to-end encrypted. During normal operation, no one — including us — can read it. When a dispute is opened, one party deliberately reveals the chat key to the mediator. The key is read-only: it can decrypt messages but cannot move funds.</p>

<p>After the dispute is resolved, the decrypted evidence is wiped. We never store your private chat keys.</p>

<h2>Why It's Safe</h2>

<ul>
  <li><strong>AI never touches money</strong> — The mediator proposes outcomes, but only the parties can execute them with their signatures</li>
  <li><strong>The covenant caps everything</strong> — Funds can only go to the buyer, seller, or the protocol fee address. The AI cannot redirect funds anywhere else</li>
  <li><strong>Chat key is separate</strong> — The key used to read messages physically cannot sign covenant spends. Different key type, different purpose</li>
</ul>

<h2>Try It</h2>

<p>Create an escrow on testnet, send a few messages, then open a dispute. You'll see the AI mediation option before the jury is empaneled.</p>
`,
	},
	{
		slug: "full-feature-set",
		title: "What We Built: The Full DagLock Feature Set",
		date: "July 7, 2026",
		excerpt:
			"One platform, 12+ covenant types, AI mediation, E2E encrypted chat, a Telegram bot with 35 commands, and an embeddable payment widget.",
		content: `
<p>DagLock started as a simple idea: trustless escrow on Kaspa using SilverScript covenants. It has grown into something much larger — a full platform for P2P trading, payments, and dispute resolution on Kaspa L1.</p>

<p>Here's everything we've built.</p>

<h2>Escrow Types</h2>

<table>
  <tr><th>Type</th><th>Description</th></tr>
  <tr><td>Standard escrow</td><td>Basic KAS or KRC-20 escrow with release, refund, swap, and auto-settle</td></tr>
  <tr><td>Milestone payments</td><td>Up to 5 stages, each released by time or buyer approval</td></tr>
  <tr><td>Recurring subscriptions</td><td>Pre-fund and draw periodically with auto-draw service</td></tr>
  <tr><td>Multi-party escrow</td><td>Up to 4 parties with custom split ratios (e.g., 70/20/10)</td></tr>
  <tr><td>Security deposits</td><td>Both parties stake a bond; jury can forfeit on bad behavior</td></tr>
  <tr><td>Atomic swaps</td><td>Hash preimage swaps with a 6-step guided wizard</td></tr>
</table>

<h2>Vaults</h2>

<ul>
  <li><strong>Time-locked vaults</strong> — Lock KAS for a duration with DAA-block maturity</li>
  <li><strong>Check-in vaults</strong> — Reset the lock timer to extend storage</li>
  <li><strong>Inheritance vaults</strong> — Designate an heir who can claim after a longer timeout</li>
  <li><strong>Multisig vaults</strong> — Require multiple signatures to withdraw</li>
  <li><strong>Softlock vaults</strong> — Recover with a password if the key is lost</li>
</ul>

<h2>Dispute Resolution</h2>

<ul>
  <li><strong>AI mediator</strong> — Non-binding proposal within minutes using OpenAI</li>
  <li><strong>Jury system</strong> — Randomly selected community members vote on outcomes</li>
  <li><strong>Escalation tiers</strong> — Mediation → jury → admin override with time-based auto-escalation</li>
  <li><strong>Arbitrate split</strong> — The arbiter can split funds at any ratio, not just all-or-nothing</li>
</ul>

<h2>Chat &amp; Evidence</h2>

<ul>
  <li><strong>E2E encrypted messaging</strong> — Ed25519 keypairs, X25519 ECDH key exchange, client-side encrypt/decrypt</li>
  <li><strong>On-chain hash anchoring</strong> — Message hashes are committed to Kaspa transactions as tamper-proof evidence</li>
  <li><strong>Dispute reveal</strong> — Party can reveal chat key to jury (read-only, cannot move funds)</li>
  <li><strong>Post-resolution wipe</strong> — Decrypted evidence is deleted after the case closes</li>
</ul>

<h2>Platform Surfaces</h2>

<ul>
  <li><strong>Web dashboard</strong> — Full React UI at daglock.com</li>
  <li><strong>Telegram bot</strong> — 35+ commands (@DagLock_bot)</li>
  <li><strong>CLI tool</strong> — Power-user terminal interface</li>
  <li><strong>REST API</strong> — Full API for integrators</li>
  <li><strong>WASM SDK</strong> — Browser-side covenant compilation</li>
  <li><strong>Embeddable widget</strong> — <code>&lt;daglock-pay&gt;</code> tag for any website</li>
</ul>

<h2>Developer Features</h2>

<ul>
  <li><strong>Analytics dashboard</strong> — Volume, escrow counts, KAS/USD price charts at /stats</li>
  <li><strong>Trading bot API</strong> — Rate-limited API key tiers (Free/Pro/Whale)</li>
  <li><strong>Price alerts</strong> — Get notified when KAS crosses your target price</li>
  <li><strong>CoinGecko integration</strong> — Real-time KAS/USD prices throughout the UI</li>
  <li><strong>CSV export</strong> — One-click download for tax reporting</li>
</ul>

<h2>Security</h2>

<ul>
  <li><strong>All covenants reviewed</strong> — 12 SilverScript contracts with dust protection, destination validation, and ICC ownership checks</li>
  <li><strong>No admin keys</strong> — The covenant defines every possible outcome. There is no "send funds to admin" path</li>
  <li><strong>Open source</strong> — Everything on GitHub</li>
  <li><strong>Recovery sheets</strong> — Downloadable key backup for chat keys</li>
</ul>

<p style="margin-top: 24px; font-size: 13px; color: #888;">
  Try it on testnet: <a href="https://daglock.com" style="color: var(--color-primary);">daglock.com</a><br>
  Source code: <a href="https://github.com/dilljens/DagLock" style="color: var(--color-primary);" target="_blank">github.com/dilljens/DagLock</a>
</p>`,
	},
	{
		slug: "how-silverscript-covenants-work",
		title: "How SilverScript Covenants Enable Trustless Trading",
		date: "July 7, 2026",
		excerpt:
			"A technical deep-dive into how DagLock uses SilverScript covenants on Kaspa L1 to create self-executing escrows that cannot be tampered with.",
		content: `
<p>DagLock is built on SilverScript — a high-level covenant language for the Kaspa blockDAG. Covenants are UTXO-based smart contracts that define how a specific output can be spent. Unlike account-based smart contracts (Ethereum), covenants are purely functional: given an input UTXO and a spending transaction, the covenant either accepts or rejects the spend.</p>

<p>This article explains how the DagLock escrow covenant works, line by line.</p>

<h2>The Core Covenant: daglock.sil</h2>

<p>The main escrow covenant has five constructor parameters:</p>

<pre><code>contract DagLock(
    byte[32] buyerKey,
    byte[32] sellerKey,
    byte[32] tradeHash,
    int timeout,
    byte[32] treasuryKey
)</code></pre>

<p>These are set when the covenant is compiled and become part of the script hash. They cannot be changed after deployment — the UTXO is locked to this specific configuration.</p>

<h2>Spending Paths</h2>

<p>The covenant defines six entrypoints, each representing a valid way to spend the UTXO:</p>

<h3>1. Release (mutual settlement)</h3>

<pre><code>entrypoint function release(sig buyerSig, sig sellerSig) {
    require(checkSig(buyerSig, pubkey(buyerKey)));
    require(checkSig(sellerSig, pubkey(sellerKey)));

    int inputValue = tx.inputs[this.activeInputIndex].value;
    int feeAmount = inputValue / 200;
    int sendAmount = inputValue - feeAmount;

    require(sendAmount >= 1000);
    require(tx.outputs[0].value == sendAmount);
    // ... treasury output check
}</code></pre>

<p>Both parties must sign. The fee is fixed at 0.5% (1/200) and is hardcoded in the covenant — it cannot be changed. The output is sent to the seller's P2PK address.</p>

<h3>2. Split (proportional distribution)</h3>

<p>The split path lets both parties agree to divide the funds at any ratio, from 0/100 to 100/0, in basis points. This is useful for partial refunds, fee sharing, or compromise settlements.</p>

<h3>3. Swap (atomic swap with hash preimage)</h3>

<pre><code>entrypoint function swap(byte[] secret) {
    require(sha256(secret) == tradeHash);
    // ... fee to treasury, remainder to output[0]
}</code></pre>

<p>Anyone with the correct preimage can claim the funds. This enables cross-party atomic swaps: buyer generates a secret, shares the hash, locks KAS in the covenant. Seller locks their asset elsewhere with the same hash. Buyer claims seller's asset by revealing the secret — seller then claims the KAS with the same secret.</p>

<h3>4. Refund (timeout)</h3>

<p>After the timeout, the buyer can reclaim their funds with their signature. No fee is charged on refund (the service was not completed).</p>

<h3>5. Emergency Refund (no-signature timeout)</h3>

<p>After timeout + 30 days, anyone can trigger a refund to the buyer with no signature required. This prevents funds from being stuck if the buyer loses their key or goes offline. The output is hardcoded to the buyer's P2PK address.</p>

<h3>6. Auto-Settle (seller protection)</h3>

<p>After timeout, anyone can trigger settlement to the seller. This prevents the buyer from receiving goods and going silent — the seller gets paid automatically when the dispute window closes.</p>

<h2>ICC Pattern for KRC-20 Tokens</h2>

<p>KRC-20 escrows use the Inter-Covenant Communication (ICC) pattern, where the DagLock covenant validates that it controls the KCC-20 token branch it's spending:</p>

<pre><code>function validateKcc20Input() {
    if (kcc20TemplatePrefixLen != 0) {
        require(OpCovInputCount(kcc20CovenantId) == 1);
        int kcc20Idx = OpCovInputIdx(kcc20CovenantId, 0);
        KCC20State prevState = readInputStateWithTemplate(
            kcc20Idx, kcc20TemplatePrefixLen,
            kcc20TemplateSuffixLen, kcc20ExpectedTemplateHash
        );
        require(prevState.ownerIdentifier == OpInputCovenantId(this.activeInputIndex));
        require(prevState.identifierType == IDENTIFIER_COVENANT_ID);
        require(!prevState.isMinter);
    }
}</code></pre>

<p>This is the fix for audit finding S3: the covenant verifies that the KCC-20 branch's <code>ownerIdentifier</code> matches its own covenant ID. Without this check, a malicious actor could create a fake KCC-20 branch and trick the DagLock covenant into authorizing its transfer.</p>

<h2>Security Properties</h2>

<ul>
  <li><strong>No admin keys.</strong> There is no "emergency withdraw" path that bypasses the rules. The covenant defines every possible outcome.</li>
  <li><strong>Fixed fee.</strong> The 0.5% fee is hardcoded as <code>inputValue / 200</code>. Neither the indexer operator nor anyone else can change it.</li>
  <li><strong>Destination validation.</strong> All no-signature paths (auto-settle, emergency refund) hardcode the output script to the intended recipient's P2PK address. A third party cannot redirect funds.</li>
  <li><strong>Dust protection.</strong> All outputs must be at least 1,000 sompi, preventing UTXO spam.</li>
  <li><strong>No fund locking.</strong> Even if every party disappears, the emergency timeout returns funds after timeout + 30 days. Funds can never be permanently stuck.</li>
</ul>

<h2>Building With SilverScript</h2>

<p>DagLock's compile API at <code>POST /v1/compile</code> lets anyone compile any DagLock covenant template with their own parameters, without running the SilverScript compiler locally. The indexer handles compilation and returns ready-to-deploy bytecode.</p>

<p>All 12 covenant source files are on GitHub under <code>contracts/src/</code>. Each one follows the same pattern: constructor parameters define the rules, entrypoints define the valid spending paths, and <code>require()</code> statements enforce constraints.</p>

<p style="margin-top: 24px; font-size: 13px; color: #888;">
  Explore the code: <a href="https://github.com/dilljens/DagLock" style="color: var(--color-primary);" target="_blank">github.com/dilljens/DagLock</a><br>
  Try it on testnet: <a href="https://daglock.com" style="color: var(--color-primary);">daglock.com</a>
</p>`,
	},
];
