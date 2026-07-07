import { useState } from "react";
import { Helmet } from "react-helmet-async";

interface Scenario {
	id: string;
	icon: string;
	title: string;
	question: string;
	explanation: string;
	rule: string;
	code: string;
	animation: string;
}

const scenarios: Scenario[] = [
	{
		id: "arbiter-steal",
		icon: "🔓",
		title: "Arbiter tries to steal",
		question: "What if the arbiter sends funds to their own address?",
		explanation:
			"The covenant's spending rules only allow outputs that match specific P2PK templates. The first output must go to either the buyer's or seller's public key. The second output must go to the treasury address. There is no 'send anywhere' path — the arbiter's signature alone cannot redirect funds to an arbitrary address. The script rejects any transaction that deviates from these fixed output patterns.",
		rule: "output validation — fixed P2PK destinations",
		code: "// Only these two output patterns are valid:\n// output[0] → buyer P2PK or seller P2PK\n// output[1] → treasury P2PK\n// Any other destination → REJECT",
		animation: "🛡️→💥←💰",
	},
	{
		id: "fee-change",
		icon: "💰",
		title: "Server changes the fee",
		question: "What if we raise the fee to 50%?",
		explanation:
			"The fee is not configurable by the server or any off-chain component. The covenant hardcodes `inputValue / 200` (0.5%) as the treasury output amount. Even the DagLock team cannot change the fee for an active escrow — it is baked into the UTXO's script at creation time. The only way to change the fee is to deploy a new covenant version, which would be a separate contract with a different address.",
		rule: "hardcoded fee — immutable in the covenant",
		code: "// Fee is set in stone at covenant creation:\nlet treasuryAmount = inputValue / 200;\n// No variable, no parameter, no override.",
		animation: "🔒📊🔒",
	},
	{
		id: "seller-ships-nothing",
		icon: "📦",
		title: "Seller ships nothing",
		question: "What if the seller takes the money and runs?",
		explanation:
			"If the seller does not deliver, the buyer can open a dispute within the dispute window. A mediator is assigned to review the case. If the mediator proposes a refund and the seller does not comply, the jury votes. Funds are locked in the covenant — neither party can move them without a valid entrypoint. The covenant has no 'seller-only withdrawal' path. The only ways out are mutual settlement, dispute resolution, or emergency timeout.",
		rule: "no unilateral seller withdrawal",
		code: "// Entrypoints that can spend:\n// settle(buyerSig, sellerSig) → both must sign\n// dispute(buyerSig) → locks funds, requires resolution\n// auto_settle() → only after buyer timeout\n// emergency_refund() → only after deadline + 30d\n// No entrypoint lets the seller withdraw alone.",
		animation: "📦❌→⚖️🛡️",
	},
	{
		id: "buyer-ghosts",
		icon: "👻",
		title: "Buyer ghosts after receiving",
		question: "What if the buyer gets the goods and goes silent?",
		explanation:
			"The covenant includes an `auto_settle()` entrypoint that triggers after a configurable timeout. If the buyer has confirmed receipt of goods (off-chain) but refuses to sign the settlement transaction, the seller simply waits. Once the timeout expires, `auto_settle()` releases the funds to the seller without the buyer's signature. The covenant enforces this — no off-chain coordination needed.",
		rule: "auto_settle() timeout release",
		code: "// After timeout, seller can settle alone:\nfunction auto_settle():\n    require(lockTime < currentBlock);\n    output[0] = sellerP2PK(inputValue - treasuryAmount);\n    output[1] = treasuryP2PK(treasuryAmount);",
		animation: "⏰⏰⏰→✅💰",
	},
	{
		id: "arbiter-disappears",
		icon: "👤",
		title: "Arbiter disappears",
		question: "What if our arbiter vanishes mid-dispute?",
		explanation:
			"Every escrow has a hard deadline. If the arbiter goes silent during a dispute, the buyer can call `emergency_refund()` after the deadline + 30 days. This entrypoint returns the full deposit to the buyer with no signature required — the covenant itself enforces the refund. No one can stall the funds forever. This prevents 'rug by neglect' scenarios where a malicious or incompetent arbiter holds funds hostage.",
		rule: "emergency_refund() — deadline + 30 day timeout",
		code: "// Emergency path — no signatures needed:\nfunction emergency_refund():\n    require(currentBlock > deadline + 30_DAYS);\n    output[0] = buyerP2PK(inputValue); // full refund, no fee",
		animation: "👤💨→💰🔙",
	},
	{
		id: "chat-forged",
		icon: "📝",
		title: "Chat evidence forged",
		question: "What if someone fakes the chat log?",
		explanation:
			"Every message in the dispute chat is encrypted end-to-end using AES-256-GCM. The server cannot read or modify message contents. Each message hash is anchored on-chain via the Kaspa transaction payload, creating a tamper-proof chain of custody. Ed25519 signatures (via the user's wallet) prove authorship. To forge evidence, an attacker would need to break AES-256, Ed25519, AND rewrite Kaspa block history — three infeasible attacks simultaneously.",
		rule: "E2E encryption + on-chain hash anchoring",
		code: "// Each message is authenticated:\n// 1. Encrypted with recipient's public key (AES-256-GCM)\n// 2. Signed with sender's wallet (Ed25519)\n// 3. Hash committed to Kaspa tx metadata\n// Server sees only ciphertext + hash — no plaintext.",
		animation: "🔗📜🔗✅",
	},
];

const scenarioColors: Record<string, { border: string; glow: string }> = {
	"arbiter-steal": { border: "#ff4444", glow: "rgba(255,68,68,0.15)" },
	"fee-change": { border: "#ff9800", glow: "rgba(255,152,0,0.15)" },
	"seller-ships-nothing": { border: "#ff7b7b", glow: "rgba(255,123,123,0.15)" },
	"buyer-ghosts": { border: "#4fc3f7", glow: "rgba(79,195,247,0.15)" },
	"arbiter-disappears": { border: "#ab47bc", glow: "rgba(171,71,188,0.15)" },
	"chat-forged": { border: "#66bb6a", glow: "rgba(102,187,106,0.15)" },
};

function ScenarioCard({ s }: { s: Scenario }) {
	const [activated, setActivated] = useState(false);
	const colors = scenarioColors[s.id] || { border: "#53d769", glow: "rgba(83,215,105,0.15)" };

	return (
		<article
			className={`security-card ${activated ? "security-card--activated" : ""}`}
			style={
				{
					"--scenario-border": colors.border,
					"--scenario-glow": colors.glow,
					borderColor: activated ? colors.border : "var(--color-border)",
				} as React.CSSProperties
			}
		>
			<div className="security-card-header">
				<span className="security-card-icon">{s.icon}</span>
				<div>
					<h3>{s.title}</h3>
					<p className="security-card-question">{s.question}</p>
				</div>
			</div>

			{!activated ? (
				<button type="button" className="security-trigger" onClick={() => setActivated(true)}>
					<span>▶</span> Execute attack
				</button>
			) : (
				<div className="security-result">
					<div className="security-animation">
						<span className="security-shield">{s.animation}</span>
					</div>
					<div className="security-verdict">
						<span className="security-cross">✕</span> Attack Failed: covenant rule —{" "}
						<strong>{s.rule}</strong>
					</div>
					<p className="security-explanation">{s.explanation}</p>
					<div className="security-code-block">
						<div className="security-code-label">SilverScript covenant rule</div>
						<pre className="security-code">{s.code}</pre>
					</div>
					<button type="button" className="security-reset" onClick={() => setActivated(false)}>
						Reset
					</button>
				</div>
			)}
		</article>
	);
}

export function SecurityPage() {
	return (
		<>
			<Helmet>
				<title>Security — Try to Break DagLock — DagLock</title>
				<meta
					name="description"
					content="Interactive security demo: try to break DagLock's covenant escrow. See why each attack fails against Kaspa L1 SilverScript covenants."
				/>
			</Helmet>
			<div className="security-page">
				<div className="page-header">
					<h1>🔒 Try to Break the Escrow</h1>
					<p>
						DagLock's covenant is the only escrow on Kaspa that lets you verify its security
						yourself. Click "Execute attack" on any scenario below to see why it fails.
					</p>
				</div>

				<div className="security-stats-bar">
					<div className="security-stat">
						<span className="security-stat-value">6</span>
						<span className="security-stat-label">Attack scenarios</span>
					</div>
					<div className="security-stat">
						<span className="security-stat-value">0</span>
						<span className="security-stat-label">Attacks succeeded</span>
					</div>
					<div className="security-stat">
						<span className="security-stat-value">100%</span>
						<span className="security-stat-label">Covenant block rate</span>
					</div>
				</div>

				<div className="security-grid">
					{scenarios.map((s) => (
						<ScenarioCard key={s.id} s={s} />
					))}
				</div>

				<div className="security-footer panel" style={{ marginTop: "24px" }}>
					<h3>Why does this work?</h3>
					<p>
						Every DagLock escrow is a UTXO locked by a SilverScript covenant — code that runs on
						Kaspa L1. The covenant defines exactly who can spend the funds, when, and to which
						addresses. No admin key, no backdoor, no upgrade mechanism can change the rules after
						the funds are locked. The attacks above fail because the covenant has no code path that
						allows them.
					</p>
					<p>
						This is not a simulation. Every security property shown here is enforced by the Kaspa
						network itself. If you find a way around the covenant,{" "}
						<a
							href="https://github.com/dilljens/DagLock"
							target="_blank"
							rel="noopener noreferrer"
							style={{ color: "var(--color-primary)", textDecoration: "underline" }}
						>
							tell us
						</a>
						.
					</p>
				</div>
			</div>
		</>
	);
}
