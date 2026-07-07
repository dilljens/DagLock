import { Helmet } from "react-helmet-async";
import { useRouter } from "../router";

const wallets = [
	{
		role: "Buyer",
		address: "kaspa:qtqwyqtmgczzjmj44vjzy",
		pubkey: "a66a35ed08534f4fe540eb1553383b65f321e40330d2b2483d03887d18c18882",
		privkey: "2d93f2f2a4181731a682284db003140dd5ef18c868a77a685b89424788d21e73",
		color: "#4fc3f7",
	},
	{
		role: "Seller",
		address: "kaspa:qjdpca9zm8aafdue2q0zn",
		pubkey: "8673a04072e11d8c275a3aaaa4a5eeb26f074f8c0178707a839776ca28f248b1",
		privkey: "9e4fc9be71da065e090bb077da09b60b260898c06145b8b656cc4b873a0eaaeb",
		color: "#53d769",
	},
	{
		role: "Mediator / Jury",
		address: "kaspa:qyp29592perates764gj8",
		pubkey: "2693198653e6caad0306c3b5477310aa7df040904a20494d5bbbdef3042d5a36",
		privkey: "1ad52ce15703e9664c5c690640b5dc13546c5cc9199b39f93276453afa1093eb",
		color: "#ff9800",
	},
];

const steps = [
	{
		step: "1",
		title: "Visit DagLock",
		desc: 'Open daglock.com — you\'ll see a testnet banner at the top. Click "Use manual mode" in the sidebar.',
	},
	{
		step: "2",
		title: "Connect a test wallet",
		desc: "Paste one of the test addresses below into the manual address input. No real keys needed — the testnet accepts any address.",
	},
	{
		step: "3",
		title: "Create an escrow",
		desc: 'Go to Escrows → Create tab. Enter an amount (e.g. 100), a counterparty address, then click Create. For TX ID, paste any 64-character hex string.',
	},
	{
		step: "4",
		title: "Settle or refund",
		desc: "Click the escrow in your list, then use Settle or Refund. The testnet accepts any signature — no real wallet required.",
	},
	{
		step: "5",
		title: "Try the Telegram bot",
		desc: "Open @DagLock_bot on Telegram. Run /setaddress with a test address, then /create to follow the native wizard.",
	},
];

export function TestnetPage() {
	const { navigate } = useRouter();

	return (
		<>
			<Helmet>
				<title>Testnet — DagLock</title>
				<meta
					name="description"
					content="Try DagLock on Kaspa Testnet-10. Test wallet addresses, quick start guide, and feature tour — no real funds needed."
				/>
			</Helmet>
			<div className="testnet-page">
				{/* Warning banner */}
				<div
					style={{
						background: "linear-gradient(135deg, #3a1a1a, #2a0a0a)",
						border: "1px solid #ff4444",
						borderRadius: "12px",
						padding: "16px 20px",
						marginBottom: "24px",
					}}
				>
					<strong style={{ color: "#ff4444", fontSize: "16px" }}>
						⚠️ Testnet Only — No Real Funds
					</strong>
					<p style={{ margin: "8px 0 0", fontSize: "14px", color: "#ccc" }}>
						DagLock is running on Kaspa Testnet-10. All addresses, transactions, and balances here
						are for testing. <strong>Never send real KAS (mainnet funds) to any address on this page.</strong>
					</p>
				</div>

				<div className="page-header">
					<h1>Testnet Quick Start</h1>
					<p>Try DagLock without connecting a real wallet or spending real KAS.</p>
				</div>

				{/* Quick start steps */}
				<section className="panel" style={{ marginBottom: "24px" }}>
					<h3 style={{ margin: "0 0 16px" }}>Get Started in 5 Minutes</h3>
					<div className="quick-start-steps">
						{steps.map((s) => (
							<div key={s.step} className="quick-start-step">
								<div className="quick-start-number">{s.step}</div>
								<div>
									<strong>{s.title}</strong>
									<p className="muted" style={{ margin: "4px 0 0", fontSize: "13px" }}>
										{s.desc}
									</p>
								</div>
							</div>
						))}
					</div>
				</section>

				{/* No KAS needed callout */}
				<div
					style={{
						background: "linear-gradient(135deg, #0a2a1a, #0a1a0a)",
						border: "1px solid var(--color-primary)",
						borderRadius: "12px",
						padding: "16px 20px",
						marginBottom: "24px",
					}}
				>
					<strong style={{ color: "var(--color-primary)", fontSize: "16px" }}>
						💡 No Real KAS Required
					</strong>
					<p style={{ margin: "8px 0 0", fontSize: "14px", color: "#ccc" }}>
						The testnet runs in offline mode. Use any 64-character hex string as a TX ID:
						<code
							style={{
								display: "block",
								marginTop: "6px",
								padding: "6px 10px",
								background: "#0a1a0a",
								borderRadius: "6px",
								fontSize: "12px",
								wordBreak: "break-all",
							}}
						>
							deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef
						</code>
					</p>
				</div>

				{/* Test wallets */}
				<section className="panel" style={{ marginBottom: "24px" }}>
					<h3 style={{ margin: "0 0 4px" }}>Test Wallet Addresses</h3>
					<p className="muted" style={{ margin: "0 0 16px", fontSize: "13px" }}>
						Pre-generated test wallets. Use manual mode to connect. Do not send real KAS to these.
					</p>

					{wallets.map((w) => (
						<div
							key={w.role}
							className="testnet-wallet-card"
							style={{ borderLeft: `3px solid ${w.color}` }}
						>
							<div className="testnet-wallet-header">
								<strong style={{ color: w.color }}>{w.role}</strong>
								<button
									className="button"
									style={{ fontSize: "11px", padding: "2px 10px" }}
									onClick={() => {
										navigator.clipboard.writeText(w.address);
									}}
								>
									Copy Address
								</button>
							</div>
							<code className="testnet-wallet-address">{w.address}</code>
							<details style={{ marginTop: "8px", fontSize: "13px" }}>
								<summary style={{ cursor: "pointer", color: "var(--color-text-muted)" }}>
									Show private key
								</summary>
								<code style={{ display: "block", marginTop: "6px", fontSize: "11px", wordBreak: "break-all" }}>
									{w.privkey}
								</code>
							</details>
						</div>
					))}
				</section>

				{/* Feature tour */}
				<section className="panel" style={{ marginBottom: "24px" }}>
					<h3 style={{ margin: "0 0 16px" }}>What to Try</h3>
					<div className="testnet-features">
						<div className="testnet-feature" onClick={() => navigate("/escrows" as any)}>
							<span className="testnet-feature-icon">🔒</span>
							<span className="testnet-feature-title">Create Escrow</span>
							<span className="testnet-feature-desc">Lock funds in a covenant</span>
						</div>
						<div className="testnet-feature" onClick={() => navigate("/offers" as any)}>
							<span className="testnet-feature-icon">📋</span>
							<span className="testnet-feature-title">Offer Board</span>
							<span className="testnet-feature-desc">Browse open trades</span>
						</div>
						<div className="testnet-feature" onClick={() => navigate("/swap" as any)}>
							<span className="testnet-feature-icon">🔄</span>
							<span className="testnet-feature-title">Atomic Swap</span>
							<span className="testnet-feature-desc">Step-by-step wizard</span>
						</div>
						<div className="testnet-feature" onClick={() => navigate("/tokens" as any)}>
							<span className="testnet-feature-icon">🏷️</span>
							<span className="testnet-feature-title">KRC-20 Tokens</span>
							<span className="testnet-feature-desc">Chart & trade tokens</span>
						</div>
						<div className="testnet-feature" onClick={() => navigate("/reputation" as any)}>
							<span className="testnet-feature-icon">📊</span>
							<span className="testnet-feature-title">Reputation</span>
							<span className="testnet-feature-desc">Check scores & vouch</span>
						</div>
						<div className="testnet-feature" onClick={() => navigate("/help" as any)}>
							<span className="testnet-feature-icon">❓</span>
							<span className="testnet-feature-title">Help & FAQ</span>
							<span className="testnet-feature-desc">Full documentation</span>
						</div>
					</div>
				</section>

				{/* Bot instructions */}
				<section className="panel" style={{ marginBottom: "24px" }}>
					<h3 style={{ margin: "0 0 4px" }}>Try the Telegram Bot</h3>
					<p className="muted" style={{ margin: "0 0 12px", fontSize: "13px" }}>
						Open{" "}
						<a href="https://t.me/DagLock_bot" target="_blank" rel="noopener noreferrer" style={{ color: "var(--color-primary)" }}>
							@DagLock_bot
						</a>{" "}
						on Telegram and run these commands:
					</p>
					<div className="testnet-bot-commands">
						<div className="testnet-bot-cmd">
							<code>/setaddress kaspa:qtqwyqtmgczzjmj44vjzy</code>
							<span className="muted">Set a test wallet</span>
						</div>
						<div className="testnet-bot-cmd">
							<code>/create</code>
							<span className="muted">Start the native wizard</span>
						</div>
						<div className="testnet-bot-cmd">
							<code>/fee 1000</code>
							<span className="muted">Calculate escrow fee</span>
						</div>
						<div className="testnet-bot-cmd">
							<code>/offers</code>
							<span className="muted">Browse live offers</span>
						</div>
					</div>
				</section>

				{/* Disclaimer */}
				<div
					style={{
						background: "#1a1a2e",
						border: "1px solid #333",
						borderRadius: "8px",
						padding: "12px 16px",
						fontSize: "12px",
						color: "#888",
						lineHeight: 1.6,
					}}
				>
					<strong>🔒 Security Note:</strong> DagLock is post-audit but pre-mainnet. The covenants have
					been reviewed internally. Key properties: no admin keys, covenant-enforced rules, open
					source at{" "}
					<a href="https://github.com/dilljens/DagLock" target="_blank" rel="noopener noreferrer" style={{ color: "var(--color-primary)" }}>
						github.com/dilljens/DagLock
					</a>. Testnet funds have no real value. Report issues on GitHub.
				</div>
			</div>
		</>
	);
}
