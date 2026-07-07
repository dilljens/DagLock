import { useState } from "react";
import { Helmet } from "react-helmet-async";

interface CodeSnippetParams {
	amount?: string;
	seller?: string;
	memo?: string;
	theme?: "light" | "dark";
	apiKey?: string;
}

function generateSnippet(p: CodeSnippetParams): string {
	const attrs = [
		`amount="${p.amount || "100"}"`,
		`seller="${p.seller || "kaspa:YOUR_ADDRESS"}"`,
		`label="Pay with KasWare"`,
	];
	if (p.memo) attrs.push(`memo="${p.memo}"`);
	if (p.theme) attrs.push(`theme="${p.theme}"`);
	if (p.apiKey) attrs.push(`api-key="${p.apiKey}"`);
	return `<script src="https://cdn.daglock.com/daglock-pay.js"></script>

<daglock-pay ${attrs.join("\n\t")}></daglock-pay>`;
}

const WEBHOOK_EVENTS = [
	{
		event: "escrow.created",
		payload: '{ "event": "escrow.created", "data": { "id": "esc_xxx" } }',
	},
	{
		event: "escrow.settled",
		payload: '{ "event": "escrow.settled", "data": { "id": "esc_xxx" } }',
	},
	{
		event: "escrow.disputed",
		payload: '{ "event": "escrow.disputed", "data": { "id": "esc_xxx", "reason": "..." } }',
	},
	{
		event: "escrow.refunded",
		payload: '{ "event": "escrow.refunded", "data": { "id": "esc_xxx" } }',
	},
];

const FAQ = [
	{
		q: "What if the buyer doesn't release funds?",
		a: "Escrows have a configurable timeout. If the buyer doesn't settle or dispute within the timeout, the seller can claim a refund. For arbiter escrows, a mediator or jury can resolve disputes.",
	},
	{
		q: "How are fees handled?",
		a: "DagLock charges a 0.5% protocol fee (1/200) on every escrow settlement. This is enforced by the SilverScript covenant — no admin keys can bypass it. The fee is deducted from the escrow amount at settlement.",
	},
	{
		q: "Is KasWare required?",
		a: "Yes — the <daglock-pay> component requires the KasWare browser extension. KasWare handles account management and transaction signing. Mobile users can use Kaspium wallet via the full DagLock web app.",
	},
	{
		q: "Can I customize the button appearance?",
		a: "Yes. The component exposes CSS variables for theming: --daglock-primary, --daglock-bg, --daglock-text, --daglock-border, --daglock-error. Set them via CSS on the parent element or inline styles.",
	},
	{
		q: "What happens if the buyer doesn't complete payment?",
		a: "The escrow is only created when the buyer connects KasWare and submits the transaction. If the buyer abandons the flow, no on-chain transaction occurs. Payment sessions expire after 24 hours.",
	},
	{
		q: "Is this non-custodial?",
		a: "Yes. Funds are held in a SilverScript covenant on the Kaspa blockchain. DagLock never has access to user funds. The covenant enforces all rules — settlement, refund, dispute resolution — without admin keys.",
	},
	{
		q: "Which assets are supported?",
		a: "Native KAS and all KRC-20 tokens on Kaspa. The component uses the asset attribute to pick the right covenant template.",
	},
	{
		q: "What about disputes?",
		a: "For standard escrows, if a dispute arises, it can be escalated to the DagLock jury system — a decentralized panel of KAS holders who vote on the outcome based on evidence.",
	},
];

export function MerchantPage() {
	const [preview, setPreview] = useState<CodeSnippetParams>({
		amount: "100",
		seller: "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q",
		memo: "Order #1234",
		theme: "dark",
		apiKey: "",
	});
	const [copied, setCopied] = useState(false);

	const snippet = generateSnippet(preview);

	const handleCopy = async () => {
		await navigator.clipboard.writeText(snippet);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	return (
		<>
			<Helmet>
				<title>Merchant — DagLock Escrow-as-a-Service</title>
				<meta
					name="description"
					content="Accept escrow payments on your website with DagLock. Embed a web component, no backend required."
				/>
			</Helmet>
			<div style={{ maxWidth: "800px", margin: "0 auto", padding: "2rem 1rem" }}>
				<h1>Accept Escrow Payments on Your Site</h1>
				<p className="muted" style={{ fontSize: "1.1rem", lineHeight: 1.6 }}>
					DagLock Escrow-as-a-Service lets you add trustless escrow payments to any website. Embed a
					single <code>&lt;daglock-pay&gt;</code> element — no backend integration needed. Funds are
					held in a SilverScript covenant on the Kaspa blockchain.
				</p>

				<hr
					style={{ border: "none", borderTop: "1px solid var(--color-border)", margin: "2rem 0" }}
				/>

				{/* Three-step setup */}
				<h2>Setup Guide</h2>
				<div
					style={{
						display: "grid",
						gap: "1.5rem",
						marginTop: "1.5rem",
					}}
				>
					{[
						{
							step: 1,
							title: "Register Your App",
							desc: "Go to the Apps page and register your site. You'll receive an API key — save it securely. This key authenticates your merchant requests.",
							link: "/apps",
							linkText: "Register →",
						},
						{
							step: 2,
							title: "Add the Script",
							desc: "Add the daglock-pay script to your HTML. You can host it yourself or use the DagLock CDN.",
						},
						{
							step: 3,
							title: "Embed the Element",
							desc: "Drop the <daglock-pay> element wherever you want the payment button to appear. Configure it with your seller address, amount, and optional memo.",
						},
					].map(({ step, title, desc, link, linkText }) => (
						<div
							key={step}
							className="panel"
							style={{
								border: "1px solid var(--color-border)",
								borderRadius: "12px",
								padding: "1.5rem",
							}}
						>
							<div style={{ display: "flex", alignItems: "flex-start", gap: "1rem" }}>
								<span
									style={{
										background: "var(--color-primary)",
										color: "#000",
										width: "32px",
										height: "32px",
										borderRadius: "50%",
										display: "flex",
										alignItems: "center",
										justifyContent: "center",
										fontWeight: 700,
										flexShrink: 0,
									}}
								>
									{step}
								</span>
								<div>
									<h3 style={{ margin: 0, fontWeight: 600 }}>{title}</h3>
									<p className="muted" style={{ margin: "0.5rem 0 0", lineHeight: 1.5 }}>
										{desc}
									</p>
									{link && (
										<a
											href={link}
											style={{
												display: "inline-block",
												marginTop: "0.5rem",
												color: "var(--color-primary)",
												fontWeight: 600,
											}}
										>
											{linkText}
										</a>
									)}
								</div>
							</div>
						</div>
					))}
				</div>

				<hr
					style={{ border: "none", borderTop: "1px solid var(--color-border)", margin: "2rem 0" }}
				/>

				{/* Code snippet generator */}
				<h2>Code Snippet</h2>
				<p className="muted">Customize the snippet below and paste it into your website.</p>

				<div
					style={{
						display: "flex",
						flexWrap: "wrap",
						gap: "1rem",
						margin: "1rem 0",
						padding: "1rem",
						border: "1px solid var(--color-border)",
						borderRadius: "12px",
					}}
				>
					<label
						style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "0.85rem" }}
					>
						Amount (KAS)
						<input
							type="number"
							value={preview.amount}
							onChange={(e) => setPreview({ ...preview, amount: e.target.value })}
							style={{
								background: "var(--color-bg)",
								border: "1px solid var(--color-border)",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "var(--color-text)",
								width: "100px",
							}}
						/>
					</label>
					<label
						style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "0.85rem" }}
					>
						Seller Address
						<input
							type="text"
							value={preview.seller}
							onChange={(e) => setPreview({ ...preview, seller: e.target.value })}
							style={{
								background: "var(--color-bg)",
								border: "1px solid var(--color-border)",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "var(--color-text)",
								width: "300px",
								fontFamily: "monospace",
								fontSize: "0.8rem",
							}}
						/>
					</label>
					<label
						style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "0.85rem" }}
					>
						Memo
						<input
							type="text"
							value={preview.memo || ""}
							onChange={(e) => setPreview({ ...preview, memo: e.target.value })}
							style={{
								background: "var(--color-bg)",
								border: "1px solid var(--color-border)",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "var(--color-text)",
								width: "200px",
							}}
						/>
					</label>
					<label
						style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "0.85rem" }}
					>
						Theme
						<select
							value={preview.theme}
							onChange={(e) =>
								setPreview({ ...preview, theme: e.target.value as "light" | "dark" })
							}
							style={{
								background: "var(--color-bg)",
								border: "1px solid var(--color-border)",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "var(--color-text)",
							}}
						>
							<option value="dark">Dark</option>
							<option value="light">Light</option>
						</select>
					</label>
				</div>

				<div
					style={{
						position: "relative",
						background: "#111",
						borderRadius: "12px",
						padding: "1.5rem",
						margin: "1rem 0",
						overflowX: "auto",
					}}
				>
					<pre
						style={{
							margin: 0,
							fontSize: "0.85rem",
							lineHeight: 1.6,
							color: "#e0e0e0",
							whiteSpace: "pre-wrap",
							wordBreak: "break-all",
						}}
					>
						{snippet}
					</pre>
					<button
						type="button"
						onClick={handleCopy}
						style={{
							position: "absolute",
							top: "0.75rem",
							right: "0.75rem",
							background: "rgba(255,255,255,0.1)",
							border: "none",
							borderRadius: "6px",
							padding: "6px 12px",
							color: "#fff",
							cursor: "pointer",
							fontSize: "0.8rem",
						}}
					>
						{copied ? "Copied!" : "Copy"}
					</button>
				</div>

				<hr
					style={{ border: "none", borderTop: "1px solid var(--color-border)", margin: "2rem 0" }}
				/>

				{/* Attributes reference */}
				<h2>Attributes Reference</h2>
				<div style={{ overflowX: "auto", marginTop: "1rem" }}>
					<table
						style={{
							width: "100%",
							borderCollapse: "collapse",
							fontSize: "0.9rem",
						}}
					>
						<thead>
							<tr style={{ borderBottom: "1px solid var(--color-border)" }}>
								<th style={{ textAlign: "left", padding: "8px 12px" }}>Attribute</th>
								<th style={{ textAlign: "left", padding: "8px 12px" }}>Type</th>
								<th style={{ textAlign: "left", padding: "8px 12px" }}>Required</th>
								<th style={{ textAlign: "left", padding: "8px 12px" }}>Description</th>
							</tr>
						</thead>
						<tbody>
							{[
								["amount", "string", "Yes", "Payment amount in KAS"],
								["seller", "string", "Yes", "Recipient Kaspa address (kaspa:...)"],
								["asset", "string", "No", 'Asset type (default: "KAS")'],
								["memo", "string", "No", "Optional memo/order reference"],
								["api-key", "string", "No", "Your DagLock API key for session creation"],
								["theme", '"dark" | "light"', "No", 'Color theme (default: "dark")'],
								["label", "string", "No", 'Button label (default: "Pay with KasWare")'],
							].map(([attr, type, req, desc]) => (
								<tr key={attr} style={{ borderBottom: "1px solid var(--color-border)" }}>
									<td
										style={{ padding: "10px 12px", fontFamily: "monospace", fontSize: "0.85rem" }}
									>
										{attr}
									</td>
									<td style={{ padding: "10px 12px", fontSize: "0.85rem", color: "#888" }}>
										{type}
									</td>
									<td style={{ padding: "10px 12px", fontSize: "0.85rem" }}>{req}</td>
									<td style={{ padding: "10px 12px", fontSize: "0.85rem" }}>{desc}</td>
								</tr>
							))}
						</tbody>
					</table>
				</div>

				<hr
					style={{ border: "none", borderTop: "1px solid var(--color-border)", margin: "2rem 0" }}
				/>

				{/* Webhook reference */}
				<h2>Webhooks</h2>
				<p className="muted">
					When you create a payment session with a <code>webhook_url</code>, DagLock sends HTTP POST
					notifications for lifecycle events. Webhooks include a <code>X-Daglock-Webhook-Id</code>{" "}
					header for idempotency and are retried up to 3 times with exponential backoff (1s, 4s,
					10s).
				</p>

				<h3>Events</h3>
				<div style={{ overflowX: "auto", marginTop: "0.5rem" }}>
					<table
						style={{
							width: "100%",
							borderCollapse: "collapse",
							fontSize: "0.9rem",
						}}
					>
						<thead>
							<tr style={{ borderBottom: "1px solid var(--color-border)" }}>
								<th style={{ textAlign: "left", padding: "8px 12px" }}>Event</th>
								<th style={{ textAlign: "left", padding: "8px 12px" }}>Payload</th>
							</tr>
						</thead>
						<tbody>
							{WEBHOOK_EVENTS.map(({ event, payload }) => (
								<tr key={event} style={{ borderBottom: "1px solid var(--color-border)" }}>
									<td
										style={{
											padding: "10px 12px",
											fontFamily: "monospace",
											fontSize: "0.85rem",
											whiteSpace: "nowrap",
										}}
									>
										{event}
									</td>
									<td
										style={{
											padding: "10px 12px",
											fontFamily: "monospace",
											fontSize: "0.8rem",
											color: "#888",
											wordBreak: "break-all",
										}}
									>
										{payload}
									</td>
								</tr>
							))}
						</tbody>
					</table>
				</div>

				<hr
					style={{ border: "none", borderTop: "1px solid var(--color-border)", margin: "2rem 0" }}
				/>

				{/* FAQ */}
				<h2>FAQ</h2>
				<div style={{ marginTop: "1rem" }}>
					{FAQ.map(({ q, a }) => (
						<details
							key={q}
							style={{
								borderBottom: "1px solid var(--color-border)",
								padding: "1rem 0",
							}}
						>
							<summary style={{ fontWeight: 600, cursor: "pointer", fontSize: "0.95rem" }}>
								{q}
							</summary>
							<p
								className="muted"
								style={{ marginTop: "0.75rem", lineHeight: 1.6, fontSize: "0.9rem" }}
							>
								{a}
							</p>
						</details>
					))}
				</div>
			</div>
		</>
	);
}
