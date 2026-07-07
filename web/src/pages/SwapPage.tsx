import { useState, useEffect } from "react";
import { api, type Escrow } from "../api";
import { useWallet } from "../context/WalletContext";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";
import { AtomicSwapWizard } from "../components/AtomicSwapWizard";
import { Panel } from "../ui";

type Tab = "wizard" | "claim" | "how-to";

export function SwapPage() {
	const [tab, setTab] = useState<Tab>("wizard");
	const { state: wallet } = useWallet();

	// Check if we're arriving from a deep link /swap/:id
	useEffect(() => {
		const path = window.location.pathname;
		const match = path.match(/^\/swap\/(.+)/);
		if (match) {
			setTab("claim");
		}
	}, []);

	return (
		<>
			<Helmet>
				<title>Atomic Swap — DagLock</title>
				<meta
					name="description"
					content="Trustless atomic swaps between KAS and KRC-20 tokens on Kaspa L1."
				/>
				<link rel="canonical" href="https://daglock.com/swap" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Atomic Swap</h1>
					<p>Trustless asset exchange via hash-locked covenants.</p>
				</div>
				<div className="tab-bar">
					<button
						className={`tab-btn ${tab === "wizard" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("wizard")}
					>
						Create Swap
					</button>
					<button
						className={`tab-btn ${tab === "claim" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("claim")}
					>
						Claim Swap
					</button>
					<button
						className={`tab-btn ${tab === "how-to" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("how-to")}
					>
						How it Works
					</button>
				</div>
				{tab === "wizard" && <AtomicSwapWizard />}
				{tab === "claim" && <ClaimSwap />}
				{tab === "how-to" && <HowItWorks />}
			</div>
		</>
	);
}

/* ─── Claim Swap (standalone, for deep links) ─── */
function ClaimSwap() {
	const { state: wallet, connect } = useWallet();

	// Extract escrow ID from URL path
	const pathEscrowId = (() => {
		const match = window.location.pathname.match(/^\/swap\/(.+)/);
		return match ? match[1] : "";
	})();

	const [escrowId, setEscrowId] = useState(pathEscrowId);
	const [escrow, setEscrow] = useState<Escrow | null>(null);
	const [preimage, setPreimage] = useState("");
	const [loading, setLoading] = useState("");
	const [error, setError] = useState("");
	const [done, setDone] = useState(false);

	useEffect(() => {
		if (escrowId) fetchEscrow(escrowId);
	}, []); // eslint-disable-line react-hooks/exhaustive-deps

	async function fetchEscrow(id: string) {
		try {
			const data = await api.escrow(id);
			setEscrow(data);
		} catch {
			setError("Escrow not found");
		}
	}

	async function handleClaim(e: React.FormEvent) {
		e.preventDefault();
		if (!escrowId || !preimage.trim()) return;
		setLoading("claiming");
		setError("");
		try {
			await api.swapEscrow(escrowId, preimage.trim());
			setDone(true);
		} catch (err) {
			setError((err as Error).message);
		} finally {
			setLoading("");
		}
	}

	if (!wallet.connected) {
		return (
			<EmptyState
				icon="👛"
				title="Connect your wallet"
				description="Connect KasWare to claim this swap."
				action={{ label: "Connect Wallet", onClick: connect }}
			/>
		);
	}

	if (done) {
		return (
			<div style={{ textAlign: "center", padding: "24px 0" }}>
				<div style={{ fontSize: "48px", marginBottom: "16px" }}>✅</div>
				<h3>Swap Claimed!</h3>
				<p className="muted">The atomic swap has been settled successfully.</p>
			</div>
		);
	}

	return (
		<div>
			{!escrow && !error && (
				<form
					className="form"
					onSubmit={(e) => {
						e.preventDefault();
						fetchEscrow(escrowId);
					}}
				>
					<input
						value={escrowId}
						onChange={(e) => setEscrowId(e.target.value)}
						placeholder="Escrow ID (esc_...)"
						style={{ marginBottom: "8px" }}
					/>
					<button className="button primary" type="submit">
						Fetch Escrow
					</button>
				</form>
			)}

			{error && <p className="muted error-text">{error}</p>}

			{escrow && (
				<form className="form form-stacked" onSubmit={handleClaim}>
					<Panel title="Escrow Details">
						<div className="stack">
							<div className="row">
								<span>ID</span>
								<code>{escrow.id}</code>
							</div>
							<div className="row">
								<span>Amount</span>
								<strong>{(escrow.amount_sompi / 1e8).toFixed(2)} KAS</strong>
							</div>
							<div className="row">
								<span>Status</span>
								<strong>{escrow.status}</strong>
							</div>
							{escrow.trade_hash && (
								<div className="row">
									<span>Expected hash</span>
									<code style={{ fontSize: "11px", wordBreak: "break-all" }}>
										{escrow.trade_hash}
									</code>
								</div>
							)}
						</div>
					</Panel>

					<div style={{ marginTop: "16px" }}>
						<label style={{ display: "block", marginBottom: "4px", fontSize: "13px" }}>
							Preimage (secret)
						</label>
						<input
							value={preimage}
							onChange={(e) => setPreimage(e.target.value)}
							placeholder="Paste the secret here"
						/>
					</div>

					{error && <p className="muted error-text">{error}</p>}

					<button
						className="button primary"
						type="submit"
						disabled={loading === "claiming" || !preimage.trim()}
						style={{ marginTop: "12px" }}
					>
						{loading === "claiming" ? "Claiming…" : "Claim Swap"}
					</button>
				</form>
			)}
		</div>
	);
}

/* ─── How it Works ─── */
function HowItWorks() {
	return (
		<div className="stack" style={{ maxWidth: "600px" }}>
			<div className="panel">
				<h3 style={{ marginTop: 0 }}>What is an Atomic Swap?</h3>
				<p>
					An atomic swap is a trustless exchange where both parties either complete the trade or
					neither does. The covenant enforces the rules — no third party needed.
				</p>
			</div>

			<div className="panel">
				<h3 style={{ marginTop: 0 }}>How the Wizard Works</h3>
				<ol style={{ lineHeight: 1.8, paddingLeft: "20px" }}>
					<li>
						<strong>Set terms</strong> — Enter the amount, asset, and counterparty address
					</li>
					<li>
						<strong>Generate secret</strong> — Creates a random secret + SHA-256 hash. Save the
						secret! Share the hash.
					</li>
					<li>
						<strong>Create escrow</strong> — Lock funds in a covenant with the hash embedded
					</li>
					<li>
						<strong>Share link</strong> — Send the swap link to your counterparty
					</li>
					<li>
						<strong>Counterparty claims</strong> — They enter the secret preimage → covenant
						verifies → funds released
					</li>
				</ol>
			</div>

			<div className="panel">
				<h3 style={{ marginTop: 0 }}>Security Notes</h3>
				<ul style={{ lineHeight: 1.8, paddingLeft: "20px" }}>
					<li>
						<strong>Never share the secret</strong> until you're ready to claim. Once revealed,
						anyone with it can settle the escrow.
					</li>
					<li>
						<strong>Save the secret before navigating away</strong>. It's generated client-side
						and never stored on the server.
					</li>
					<li>
						<strong>If the timeout expires</strong> without the swap being claimed, the buyer
						can refund the funds.
					</li>
				</ul>
			</div>
		</div>
	);
}
