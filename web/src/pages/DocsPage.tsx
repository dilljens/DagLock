import { useState } from "react";

const TABS = [
	{ id: "api", label: "API" },
	{ id: "cli", label: "CLI" },
	{ id: "bot", label: "Bot" },
	{ id: "integrate", label: "Integrate" },
] as const;

type TabId = (typeof TABS)[number]["id"];

const API_BASE = "https://api.daglock.com";

export function DocsPage() {
	const [tab, setTab] = useState<TabId>("api");

	return (
		<div>
			<div className="page-header">
				<h1>Developer Docs</h1>
				<p>Guides and references for building on DagLock</p>
			</div>

			<div className="tab-bar" role="tablist">
				{TABS.map((t) => (
					<button
						key={t.id}
						className={`tab-btn ${tab === t.id ? "tab-btn--active" : ""}`}
						onClick={() => setTab(t.id)}
						role="tab"
						aria-selected={tab === t.id}
					>
						{t.label}
					</button>
				))}
			</div>

			{tab === "api" && <ApiTab />}
			{tab === "cli" && <CliTab />}
			{tab === "bot" && <BotTab />}
			{tab === "integrate" && <IntegrateTab />}
		</div>
	);
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
	return (
		<section className="panel" style={{ marginTop: "16px" }}>
			<div className="panel-head"><h3>{title}</h3></div>
			{children}
		</section>
	);
}

function Code({ children }: { children: string }) {
	return (
		<pre style={{
			background: "rgba(0,0,0,0.3)", padding: "12px 16px", borderRadius: "8px",
			fontSize: "13px", overflowX: "auto", lineHeight: 1.5, margin: "8px 0",
		}}>
			<code>{children}</code>
		</pre>
	);
}

function ApiTab() {
	return (
		<>
			<Section title="Base URL">
				<p className="muted">All API endpoints are at <code>{API_BASE}</code></p>
				<p className="muted">OpenAPI spec: <code>{API_BASE}/v1/openapi.json</code> (19 endpoints)</p>
			</Section>

			<Section title="Authentication">
				<p className="muted">Escrow lifecycle actions require Schnorr signatures signed by your Kaspa wallet.</p>
				<p className="muted" style={{ marginTop: "8px" }}>Headers:</p>
				<Code>{`X-Daglock-Address: <your-kaspa-address>
X-Daglock-Signature: <64-byte-schnorr-sig>
X-Daglock-Message: <action>:<escrow_id>:<timestamp>:<nonce>`}</Code>
				<p className="muted">Supported actions: settle, refund, dispute, cancel, evidence, vote, vouch</p>
			</Section>

			<Section title="Key Endpoints">
				<Code>{`# Health
GET /v1/health

# Network info
GET /v1/network
GET /v1/network/price

# Escrows (CRUD + lifecycle)
POST /v1/escrows          # Create escrow
GET  /v1/escrows          # List by address
GET  /v1/escrows/:id      # Get by ID
POST /v1/escrows/:id/settle
POST /v1/escrows/:id/refund
POST /v1/escrows/:id/dispute
POST /v1/escrows/:id/cancel
POST /v1/escrows/:id/swap

# Offers
POST   /v1/offers         # Create offer
GET    /v1/offers          # List offers
POST   /v1/offers/:id/accept
POST   /v1/offers/:id/cancel

# Vaults
POST /v1/vaults
GET  /v1/vaults
POST /v1/vaults/:id/withdraw

# Reputation
GET /v1/reputation/:address

# Jury
POST /v1/jury/register
GET  /v1/jury/cases
POST /v1/jury/cases/:id/vote

# App registration (for integrators)
POST /v1/apps/register`}</Code>
			</Section>

			<Section title="Fees">
				<p className="muted">Both fees are enforced by the SilverScript covenant at the protocol level. DagLock cannot change or waive them.</p>
				<Code>{`Escrow settlement: 0.5% (1/200 of the deposited amount)
  → Paid by the seller at settlement
  → Treasury output enforced by covenant

Vault withdrawal: 0.1% (1/1000 of the vault amount)
  → Paid by the vault owner at withdrawal
  → Treasury output enforced by covenant`}</Code>
			</Section>

			<Section title="Reputation Formula">
				<p className="muted">DagLock uses the Beta reputation system (Josang 2002) with recency weighting.</p>
				<Code>{`Beta score = (settled + 1) / (trades + 2)
  → Recent trades (90d) weighted 2x vs older trades

Volume bonus = ln(volume_kas / 1000 + 1) × 0.12
Age bonus   = min(age_days / 365, 2) × 0.05
Score = 1 + (centered_beta × 4) + volume_bonus + age_bonus
  → Clamped to [1.0, 5.0]`}</Code>
				<p className="muted">Vouch score uses EigenTrust-lite: each vouch contributes <code>voucher_score / 5.0</code> weight. Vouchers with 0 trades get score=1.0, weight=0.2. Vouches expire after 6 months.</p>
			</Section>

			<Section title="Rate Limits">
				<p className="muted">30 req/min per IP without API key.</p>
				<p className="muted">300 req/min per IP with <code>X-Daglock-Api-Key</code> header.</p>
				<p className="muted">Register an app at <code>POST /v1/apps/register</code> to get an API key.</p>
			</Section>
		</>
	);
}

function CliTab() {
	return (
		<>
			<Section title="Installation">
				<p className="muted">The CLI tool lets you interact with DagLock from the terminal.</p>
				<Code>{`# Build from source
git clone https://github.com/dilljens/DagLock
cd daglock
cargo build --release -p daglock-cli
./target/release/daglock-cli --help`}</Code>
			</Section>

			<Section title="Configuration">
				<p className="muted">Set your API endpoint:</p>
				<Code>{`daglock-cli config --api-url https://api.daglock.com`}</Code>
			</Section>

			<Section title="Commands">
				<Code>{`# Check reputation
daglock-cli reputation <kaspa-address>

# Browse offers
daglock-cli offer list

# Create an escrow (requires kaspawallet for signing)
daglock-cli create --amount 100 --counterparty <address>

# Check escrow status
daglock-cli status <escrow-id>

# Vault management
daglock-cli vault list
daglock-cli vault create --amount 500 --timeout 30`}</Code>
			</Section>
		</>
	);
}

function BotTab() {
	return (
		<>
			<Section title="Getting Started">
				<p className="muted">Open Telegram and search for <strong>@DagLock_bot</strong>. Send <code>/start</code> to begin.</p>
				<p className="muted">First, set your address with <code>/setaddress &lt;kaspa-address&gt;</code>.</p>
			</Section>

			<Section title="Commands">
				<Code>{`/start        — Welcome + deep link handling
/setaddress  — Set your Kaspa address
/create      — 4-step wizard: amount, counterparty, timeout, dispute mode
/claim <id>  — Claim a pending escrow
/list        — List your escrows
/offers      — Browse open offers with inline keyboard
/status <id> — Check escrow lifecycle state
/swap <id> <hex> — Atomic swap settle via preimage
/vaults      — List your time-locked vaults
/receipt <id> — Export settlement receipt
/dispute <id> <reason> — Dispute an escrow
/cancel <id> — Cancel an escrow
/reputation <address> — Check counterparty stats
/msg <id> <text> — Send message on an escrow
/messages <id> — Read escrow thread
/evidence <id> — List dispute evidence
/help        — All commands`}</Code>
			</Section>

			<Section title="Trade Links">
				<p className="muted">Share this link to let someone claim an escrow:</p>
				<Code>{`https://t.me/DagLock_bot?start=claim_<escrow-id>`}</Code>
			</Section>
		</>
	);
}

function IntegrateTab() {
	return (
		<>
			<Section title="TypeScript SDK">
				<p className="muted">Use the DagLock SDK in your own web app to compile covenants and interact with the API.</p>
				<Code>{`npm install @daglock/sdk`}</Code>
				<p className="muted" style={{ marginTop: "8px" }}>Example — check an address reputation:</p>
				<Code>{`import { DagLockSDK } from "@daglock/sdk";

const daglock = new DagLockSDK({ apiUrl: "https://api.daglock.com" });
const rep = await daglock.getReputation("kaspa:...");
console.log(rep.score, rep.trade_count);`}</Code>
			</Section>

			<Section title="KasWare Integration">
				<p className="muted">DagLock uses KasWare browser extension for wallet operations.</p>
				<p className="muted" style={{ marginTop: "8px" }}>KasWare exposes:</p>
				<Code>{`window.kasware.getPublicKey()  → coin type + pubkey bytes
window.kasware.getBalance()   → { confirmed, unconfirmed }
window.kasware.signMessage()  → Schnorr signature (BIP-340)
window.kasware.sendKaspa()    → broadcast transaction → tx_id`}</Code>
			</Section>

			<Section title="Webhooks">
				<p className="muted">Subscribe to lifecycle events via HTTP callbacks. Register an app first, then add webhooks.</p>
				<Code>{`# Register an app
curl -X POST https://api.daglock.com/v1/apps/register \\
  -H "Content-Type: application/json" \\
  -d '{"name": "MyApp", "owner_address": "kaspa:..."}'

# Add a webhook
curl -X POST https://api.daglock.com/v1/apps/:id/webhooks \\
  -H "X-Daglock-Api-Key: <key>" \\
  -d '{"url": "https://myapp.com/webhook", "events": ["escrow.settled", "escrow.disputed"]}'`}</Code>
				<p className="muted" style={{ marginTop: "8px" }}>Available events:</p>
				<Code>{`escrow.created
escrow.settled
escrow.refunded
escrow.disputed
escrow.cancelled
escrow.expired
offer.created
offer.accepted`}</Code>
				<p className="muted">Delivery: HTTP POST with 3 retries (1s, 4s, 10s backoff). Two header IDs for idempotency.</p>
			</Section>

			<Section title="Encrypted Messaging">
				<p className="muted">Each escrow has a threaded, encrypted chat attached. Messages are encrypted with AES-256-GCM using the server-side <code>DAGLOCK_MESSAGE_KEY</code>. Only the escrow participants can read them.</p>
				<Code>{`# Send a message
POST /v1/escrows/:id/messages
Headers: X-Daglock-Address, X-Daglock-Signature
Body: { "content": "message (max 1024 chars)" }

# List messages
GET /v1/escrows/:id/messages
Headers: X-Daglock-Address, X-Daglock-Signature`}</Code>
			</Section>

			<Section title="Bug Reports & Feedback">
				<p className="muted">Found a bug or have a feature request? Open an issue on GitHub:</p>
				<p className="muted"><a href="https://github.com/dilljens/DagLock/issues" target="_blank" rel="noopener noreferrer" style={{ color: "var(--color-primary)", textDecoration: "underline" }}>github.com/dilljens/DagLock/issues</a></p>
				<p className="muted">Or message the team on Telegram: <a href="https://t.me/DagLock_bot" target="_blank" rel="noopener noreferrer" style={{ color: "var(--color-primary)", textDecoration: "underline" }}>@DagLock_bot</a></p>
			</Section>
		</>
	);
}
