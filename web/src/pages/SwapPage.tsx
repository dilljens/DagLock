import { useState } from "react";
import { api } from "../api";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";
import { EmptyState } from "../components/empty-state";

type Tab = "generate" | "submit" | "how-to";

export function SwapPage() {
	const [tab, setTab] = useState<Tab>("generate");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<div>
			<div className="page-header">
				<h1>Atomic Swap</h1>
				<p>Generate secret/hash pairs and settle hash-locked escrows.</p>
			</div>
			<div className="tab-bar">
				<button
					className={`tab-btn ${tab === "generate" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("generate")}
				>
					Generate Swap
				</button>
				<button
					className={`tab-btn ${tab === "submit" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("submit")}
				>
					Submit Preimage
				</button>
				<button
					className={`tab-btn ${tab === "how-to" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("how-to")}
				>
					How it Works
				</button>
			</div>
			{tab === "generate" && <GenerateSwap />}
			{tab === "submit" &&
				(wallet.connected ? <SubmitPreimage address={address!} /> : <ConnectPrompt />)}
			{tab === "how-to" && <HowItWorks />}
		</div>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<EmptyState
			icon="👛"
			title="Connect your wallet"
			description="Connect KasWare to submit preimages and settle atomic swaps."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

/* ─── Generate Swap (secret + hash) ─── */
function GenerateSwap() {
	const [secret, setSecret] = useState("");
	const [hash, setHash] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [copiedIdx, setCopiedIdx] = useState<"secret" | "hash" | null>(null);

	async function handleGenerate() {
		setStatus("loading");
		setError("");
		try {
			const res = await api.generateSwap();
			setSecret(res.secret);
			setHash(res.hash);
			setStatus("done");
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	async function copyToClipboard(value: string, label: "secret" | "hash") {
		try {
			await navigator.clipboard.writeText(value);
			setCopiedIdx(label);
			setTimeout(() => setCopiedIdx(null), 2000);
		} catch {
			// Fallback: select text
		}
	}

	return (
		<div>
			<p className="muted" style={{ marginBottom: "16px" }}>
				Generate a random secret and its SHA-256 hash for use in atomic swaps. Share the hash with
				your counterparty — the secret is revealed only when the swap executes.
			</p>

			{status === "idle" && (
				<button className="button primary" onClick={handleGenerate}>
					Generate Secret & Hash
				</button>
			)}

			{status === "loading" && <p className="muted">Generating…</p>}

			{status === "error" && <p className="muted error-text">{error}</p>}

			{status === "done" && (
				<div className="stack">
					<div
						style={{
							background: "#332200",
							border: "1px solid #ff9800",
							borderRadius: "8px",
							padding: "12px",
							marginBottom: "16px",
						}}
					>
						<strong style={{ color: "#ff9800" }}>⚠ Save this secret!</strong>
						<p style={{ fontSize: "13px", margin: "8px 0 0", color: "#ccc" }}>
							The secret is needed to claim the escrow. If lost, the funds may be locked forever.
							The hash is safe to share — it's what goes into the escrow covenant.
						</p>
					</div>

					<FormField label="Secret (keep private)">
						<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
							<code
								style={{
									flex: 1,
									padding: "8px",
									background: "#1a1a1a",
									borderRadius: "4px",
									fontSize: "12px",
									wordBreak: "break-all",
								}}
							>
								{secret}
							</code>
							<button
								className="button"
								onClick={() => copyToClipboard(secret, "secret")}
								style={{ whiteSpace: "nowrap" }}
							>
								{copiedIdx === "secret" ? "Copied!" : "Copy"}
							</button>
						</div>
					</FormField>

					<FormField label="Hash (share with counterparty)">
						<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
							<code
								style={{
									flex: 1,
									padding: "8px",
									background: "#1a1a1a",
									borderRadius: "4px",
									fontSize: "12px",
									wordBreak: "break-all",
								}}
							>
								{hash}
							</code>
							<button
								className="button"
								onClick={() => copyToClipboard(hash, "hash")}
								style={{ whiteSpace: "nowrap" }}
							>
								{copiedIdx === "hash" ? "Copied!" : "Copy"}
							</button>
						</div>
					</FormField>

					<div style={{ marginTop: "16px" }}>
						<button className="button primary" onClick={handleGenerate}>
							Generate Another
						</button>
					</div>
				</div>
			)}
		</div>
	);
}

/* ─── Submit Preimage ─── */
function SubmitPreimage({ address: _address }: { address: string }) {
	const [escrowId, setEscrowId] = useState("");
	const [preimage, setPreimage] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [expectedHash, setExpectedHash] = useState<string | null>(null);
	const [result, setResult] = useState<string | null>(null);
	const { notify } = useToast();

	async function fetchEscrow(id: string) {
		if (!id.trim()) {
			setExpectedHash(null);
			return;
		}
		try {
			const data = await api.escrow(id.trim());
			if (data.trade_hash) {
				setExpectedHash(data.trade_hash);
			} else {
				setExpectedHash(null);
			}
		} catch {
			setExpectedHash(null);
		}
	}

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!escrowId.trim() || !preimage.trim()) return;
		setStatus("loading");
		setError("");
		try {
			const res = await api.swapEscrow(escrowId.trim(), preimage.trim());
			setResult(res.preimage_hash || "Settled");
			setStatus("done");
			notify("success", "Swap settled!");
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") {
		return (
			<EmptyState
				icon="✅"
				title="Preimage submitted!"
				description={`Preimage hash: ${result}`}
				action={{
					label: "Settle Another",
					onClick: () => {
						setStatus("idle");
						setEscrowId("");
						setPreimage("");
						setResult(null);
						setExpectedHash(null);
					},
				}}
			/>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">
				Submit the preimage (secret) to settle an escrow that was created with a trade hash. The
				preimage must match the hash in the escrow.
			</p>

			<FormField label="Escrow ID">
				<input
					value={escrowId}
					onChange={(e) => {
						setEscrowId(e.target.value);
						fetchEscrow(e.target.value);
					}}
					placeholder="esc_..."
				/>
			</FormField>

			{expectedHash && (
				<div style={{ fontSize: "13px", padding: "8px 0" }}>
					<span style={{ color: "#888" }}>Expected hash: </span>
					<code style={{ fontSize: "12px", display: "inline", wordBreak: "break-all" }}>
						{expectedHash}
					</code>
				</div>
			)}
			{!expectedHash && escrowId.trim() && (
				<p className="muted" style={{ fontSize: "13px" }}>
					This escrow has no trade hash. A preimage is not needed — use the Settle action instead.
				</p>
			)}

			<FormField label="Preimage (hex secret)">
				<input
					value={preimage}
					onChange={(e) => setPreimage(e.target.value)}
					placeholder="hex encoded secret from Generate tab"
				/>
			</FormField>

			{error && <p className="muted error-text">{error}</p>}

			<button
				className="button primary"
				type="submit"
				disabled={status === "loading" || !escrowId.trim() || !preimage.trim()}
			>
				{status === "loading" ? "Settling…" : "Claim with Preimage"}
			</button>
		</form>
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
					neither does. No third party or escrow agent needed — the covenant enforces the rules.
				</p>
			</div>

			<div className="panel">
				<h3 style={{ marginTop: 0 }}>Step-by-Step</h3>
				<ol style={{ lineHeight: 1.8, paddingLeft: "20px" }}>
					<li>
						<strong>Seller generates a secret</strong> — a random hex string and its SHA-256 hash.
						The secret is kept private; the hash is shared with the buyer.
					</li>
					<li>
						<strong>Buyer creates an escrow</strong> on the{" "}
						<a href="#/escrows" style={{ color: "#88b888" }}>
							Escrows page
						</a>{" "}
						using the hash from step 1. The escrow locks funds in a covenant that can only be
						released with the matching preimage (or after timeout).
					</li>
					<li>
						<strong>Seller claims the escrow</strong> by submitting the original secret (preimage)
						via the <strong>Submit Preimage</strong> tab. The covenant verifies that hash(preimage)
						matches the stored trade hash, and releases funds.
					</li>
					<li>
						<strong>Buyer receives funds</strong> on the other side of the swap (the linked
						transaction). Both sides execute atomically — if one fails, neither completes.
					</li>
				</ol>
			</div>

			<div className="panel">
				<h3 style={{ marginTop: 0 }}>Why use this?</h3>
				<ul style={{ lineHeight: 1.8, paddingLeft: "20px" }}>
					<li>
						<strong>Trustless</strong> — No counterparty risk. The covenant enforces the swap.
					</li>
					<li>
						<strong>No middleman</strong> — No escrow agent, no mediator fees for simple swaps.
					</li>
					<li>
						<strong>Time-boxed</strong> — If the swap isn't claimed within the timeout, funds are
						refunded to the buyer.
					</li>
					<li>
						<strong>Cross-chain ready</strong> — The same hash-lock pattern works across blockchains
						(BTC, LTC, etc.).
					</li>
				</ul>
			</div>

			<div className="panel">
				<h3 style={{ marginTop: 0 }}>Security Notes</h3>
				<ul style={{ lineHeight: 1.8, paddingLeft: "20px" }}>
					<li>
						<strong>Never share the secret</strong> until you're ready to claim. Once revealed,
						anyone with the secret and access to the escrow can settle it.
					</li>
					<li>
						<strong>Save the secret before navigating away</strong>. The secret is generated
						client-side and never stored on the server. If you lose it, the funds may be locked
						until the timeout expires.
					</li>
					<li>
						<strong>Verify the hash</strong> after pasting — the hash shown in the escrow must match
						the hash from your generated secret.
					</li>
				</ul>
			</div>
		</div>
	);
}
