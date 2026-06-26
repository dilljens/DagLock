import { useState } from "react";
import { api, type AuthHeaders, type CreateEscrowRequest, type Escrow } from "../api";
import { money, sompi, time, relativeTime, badge } from "../helpers";
import type { LoadState } from "../helpers";
import { Panel, LookupResult, FormField, ConfirmDialog } from "../ui";
import { SignWithWallet } from "./wallet";

/* ─── Create Escrow ─── */
export function CreateEscrowForm({ onDone }: { onDone: () => void }) {
	const [amount, setAmount] = useState("");
	const [buyerAddress, setBuyerAddress] = useState("");
	const [sellerAddress, setSellerAddress] = useState("");
	const [assetType, setAssetType] = useState("KAS");
	const [disputeMode, setDisputeMode] = useState("standard");
	const [mediatorKey, setMediatorKey] = useState("");
	const [tradeHash, setTradeHash] = useState("");
	const [tradeSecret, setTradeSecret] = useState("");
	const [priceType, setPriceType] = useState("market");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<Escrow | null>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		const trimmedBuyer = buyerAddress.trim();
		if (!trimmedBuyer.startsWith("kaspa:")) {
			setError("Invalid buyer address. Must be a valid Kaspa address starting with 'kaspa:'.");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			// Use KasWare broadcast tx_id if available, fallback to manual
			let lockTxId = "";
			const outputIndex = 0;

			if (window.kasware?.sendKaspa) {
				// Broadcast the covenant funding tx via KasWare
				lockTxId = await window.kasware.sendKaspa(trimmedBuyer, sompi(amountNum));
			} else {
				// Fallback: manually input tx_id (kaspawallet CLI flow)
				lockTxId = prompt("Tx broadcast tx_id (from kaspawallet):") || "";
				if (!lockTxId)
					throw new Error("Tx ID required. Use: kaspawallet send --to <covenant_address>");
			}

			const body: CreateEscrowRequest = {
				lock_tx_id: lockTxId,
				lock_tx_output_index: outputIndex,
				buyer_address: trimmedBuyer,
				amount_sompi: sompi(amountNum),
				asset_type: assetType,
			};
			if (sellerAddress.startsWith("kaspa:")) body.seller_address = sellerAddress;
			body.dispute_mode = disputeMode;
			if (disputeMode === "mediator" && mediatorKey.startsWith("kaspa:"))
				body.mediator_key = mediatorKey;
			if (tradeHash.trim()) body.trade_hash = tradeHash.trim();
			body.price_type = priceType;
			if (priceType === "market") {
				// Price will be fetched and locked by the backend
			}
			const escrow = await api.createEscrow(body);
			setResult(escrow);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done" && result) {
		return (
			<div className="result stack">
				<p className="muted success-text">Escrow created!</p>
				<div className="row">
					<span>ID</span>
					<code>{result.id}</code>
				</div>
				<div className="row">
					<span>Amount</span>
					<strong>{money(result.amount_sompi)}</strong>
				</div>
				<div className="row">
					<span>Lock TX</span>
					<code>{result.lock_tx_id.slice(0, 32)}…</code>
				</div>
				{result.mediator_key && (
					<div className="row">
						<span>Mediator</span>
						<strong className="addr">{result.mediator_key}</strong>
					</div>
				)}
				{result.price_at_creation != null && (
					<div className="row">
						<span>Price at creation</span>
						<strong>
							${result.price_at_creation.toFixed(4)} {result.price_currency || "USD"}
						</strong>
					</div>
				)}
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Asset type">
				<select value={assetType} onChange={(e) => setAssetType(e.target.value)}>
					<option value="KAS">KAS</option>
					<option value="KRC20">KRC-20</option>
				</select>
			</FormField>
			<FormField label="Amount (KAS)">
				<input
					type="number"
					step="any"
					value={amount}
					onChange={(e) => setAmount(e.target.value)}
					placeholder="100"
				/>
			</FormField>
			<FormField label="Buyer address">
				<input
					value={buyerAddress}
					onChange={(e) => setBuyerAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Seller address (optional)">
				<input
					value={sellerAddress}
					onChange={(e) => setSellerAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Dispute resolution">
				<select value={disputeMode} onChange={(e) => setDisputeMode(e.target.value)}>
					<option value="standard">Standard (timeout refund)</option>
					<option value="mediator">Specific mediator</option>
					<option value="jury">Jury (community vote)</option>
				</select>
			</FormField>
			{disputeMode === "mediator" && (
				<FormField label="Mediator address">
					<input
						value={mediatorKey}
						onChange={(e) => setMediatorKey(e.target.value)}
						placeholder="kaspa:..."
					/>
				</FormField>
			)}
			{tradeHash && (
				<FormField label="Trade secret (for atomic swap)">
					<p className="muted" style={{ fontSize: "12px", marginTop: 0 }}>
						Secret: <code>{tradeSecret}</code>
					</p>
				</FormField>
			)}
			<FormField label="Price type">
				<select value={priceType} onChange={(e) => setPriceType(e.target.value)}>
					<option value="market">Market price (locked at creation)</option>
					<option value="fixed">Fixed price</option>
				</select>
			</FormField>
			{priceType === "market" && (
				<small className="muted" style={{ fontSize: "12px", marginTop: "-8px" }}>
					Price will be fetched from CoinGecko and locked at creation time.
				</small>
			)}
			<FormField label="Trade hash (optional)">
				<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
					<input
						value={tradeHash}
						onChange={(e) => setTradeHash(e.target.value)}
						placeholder="Leave empty for non-atomic escrow"
						style={{ flex: 1 }}
					/>
					<button
						type="button"
						className="button"
						onClick={async () => {
							try {
								const res = await api.generateSwap();
								setTradeHash(res.hash);
								setTradeSecret(res.secret);
							} catch (err) {
								setError((err as Error).message);
							}
						}}
					>
						Generate
					</button>
				</div>
				<small className="muted" style={{ fontSize: "11px", marginTop: "4px", display: "block" }}>
					Save this secret! It's needed to claim the escrow atomically.
				</small>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<p className="muted" style={{ fontSize: "12px", margin: "4px 0" }}>
				A <strong>0.5% protocol fee</strong> will be charged on settlement (enforced by the
				covenant).
			</p>
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create escrow"}
			</button>
		</form>
	);
}

/* ─── Atomic Swap Form ─── */
export function SwapForm({ onDone }: { onDone: () => void }) {
	const [escrowId, setEscrowId] = useState("");
	const [preimage, setPreimage] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<string | null>(null);
	const [expectedHash, setExpectedHash] = useState<string | null>(null);

	async function fetchEscrow() {
		if (!escrowId.trim()) return;
		try {
			const data = await api.escrow(escrowId.trim());
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
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") {
		return <p className="muted success-text">Swap settled! Preimage hash: {result}</p>;
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">
				Settle an escrow. For market orders, no preimage is needed. For atomic swaps, submit the
				preimage.
			</p>
			<FormField label="Escrow ID">
				<input
					value={escrowId}
					onChange={(e) => {
						setEscrowId(e.target.value);
						fetchEscrow();
					}}
					placeholder="esc_..."
				/>
			</FormField>
			{expectedHash && (
				<div className="row" style={{ fontSize: "13px" }}>
					<span>Expected hash</span>
					<code>{expectedHash}</code>
				</div>
			)}
			<FormField label="Preimage (hex, optional for market orders)">
				<input
					value={preimage}
					onChange={(e) => setPreimage(e.target.value)}
					placeholder="hex encoded secret (leave empty for market orders)"
				/>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Settling..." : "Submit Preimage"}
			</button>
		</form>
	);
}

/* ─── Escrow Action (settle / refund / dispute / cancel) ─── */
type EscrowAction = "settle" | "refund" | "dispute" | "cancel" | "swap";

function EscrowActionForm({ action }: { action: EscrowAction }) {
	const [escrowId, setEscrowId] = useState("");
	const [disputeReason, setDisputeReason] = useState("");
	const [authAddress, setAuthAddress] = useState("");
	const [authSignature, setAuthSignature] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<{
		status: string;
		escrow_id: string;
	} | null>(null);
	const [showConfirm, setShowConfirm] = useState(false);

	const needsAuth = action === "settle" || action === "refund";
	const verb =
		action === "settle"
			? "Settle"
			: action === "refund"
				? "Refund"
				: action === "dispute"
					? "Dispute"
					: "Cancel";

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!escrowId) return;
		if (action === "dispute" && !disputeReason) return;

		if (action === "cancel" || action === "refund" || action === "dispute") {
			setShowConfirm(true);
			return;
		}

		setStatus("loading");
		setError("");

		try {
			let auth: AuthHeaders | undefined;
			if (needsAuth) {
				if (!authAddress || !authSignature) {
					setStatus("error");
					setError(
						"Authentication required. Please provide your Kaspa address and signature to proceed.",
					);
					return;
				}
				auth = {
					address: authAddress,
					signature: authSignature,
					message: `${action}:${escrowId}`,
				};
			}

			const res = await api.settleEscrow(escrowId, auth as AuthHeaders);
			setResult(res);
			setStatus("done");
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	async function confirmAndSubmit() {
		setShowConfirm(false);
		setStatus("loading");
		setError("");
		try {
			let auth: AuthHeaders | undefined;
			if (needsAuth) {
				if (!authAddress || !authSignature) {
					setStatus("error");
					setError("Authentication required. Please provide your Kaspa address and signature.");
					return;
				}
				auth = {
					address: authAddress,
					signature: authSignature,
					message: `${action}:${escrowId}`,
				};
			}
			let res: { status: string; escrow_id: string };
			if (action === "settle") res = await api.settleEscrow(escrowId, auth as AuthHeaders);
			else if (action === "refund") res = await api.refundEscrow(escrowId, auth as AuthHeaders);
			else if (action === "dispute") res = await api.disputeEscrow(escrowId, disputeReason);
			else res = await api.cancelEscrow(escrowId);
			setResult(res);
			setStatus("done");
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done" && result) {
		return (
			<p className="muted success-text">
				{result.status} — {result.escrow_id}
			</p>
		);
	}

	return (
		<>
			<form className="form form-stacked" onSubmit={handleSubmit}>
				<FormField label="Escrow ID">
					<input
						value={escrowId}
						onChange={(e) => setEscrowId(e.target.value)}
						placeholder="esc_..."
					/>
				</FormField>
				{action === "dispute" && (
					<FormField label="Reason">
						<input
							value={disputeReason}
							onChange={(e) => setDisputeReason(e.target.value)}
							placeholder="Why are you disputing?"
						/>
					</FormField>
				)}
				{needsAuth && (
					<>
						<FormField label="Your address">
							<input
								value={authAddress}
								onChange={(e) => setAuthAddress(e.target.value)}
								placeholder="kaspa:..."
							/>
						</FormField>
						{authAddress.startsWith("kaspa:") && (
							<FormField label="Signature">
								<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
									<input
										value={authSignature}
										onChange={(e) => setAuthSignature(e.target.value)}
										placeholder="auto-filled when signing"
										readOnly={authSignature.length > 0}
										style={{ flex: 1 }}
									/>
									<SignWithWallet
										message={`${action}:${escrowId}`}
										onSignature={(sig) => setAuthSignature(sig)}
										walletAddress={authAddress}
									/>
								</div>
							</FormField>
						)}
					</>
				)}
				{error && <p className="muted error-text">{error}</p>}
				{action === "settle" && (
					<p className="muted" style={{ fontSize: "12px", margin: "4px 0" }}>
						A <strong>0.5% protocol fee</strong> is charged on settlement (enforced by the
						covenant).
					</p>
				)}
				<button
					className={`button ${action === "cancel" ? "" : "primary"}`}
					type="submit"
					disabled={status === "loading"}
				>
					{status === "loading" ? `${verb}ing…` : verb}
				</button>
			</form>
			{showConfirm && (
				<ConfirmDialog
					title={`${verb} escrow`}
					message={`Are you sure you want to ${action} escrow ${escrowId}? This action cannot be undone.`}
					confirmLabel={verb}
					onConfirm={confirmAndSubmit}
					onCancel={() => setShowConfirm(false)}
				/>
			)}
		</>
	);
}

export { EscrowActionForm };

/* ─── Dispute with Evidence Form ─── */
export function DisputeWithEvidenceForm({ onDone }: { onDone: () => void }) {
	const [escrowId, setEscrowId] = useState("");
	const [reason, setReason] = useState("");
	const [evidenceContent, setEvidenceContent] = useState("");
	const [authAddress, setAuthAddress] = useState("");
	const [authSig, setAuthSig] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "disputed" | "error">("idle");
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!escrowId || !reason) return;
		const trimmedDispAddr = authAddress.trim();
		if (!trimmedDispAddr || !authSig) {
			setError("Authentication required. Please provide your Kaspa address and signature.");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const auth: AuthHeaders = {
				address: trimmedDispAddr,
				signature: authSig,
				message: `dispute:${escrowId}`,
			};
			// Submit evidence first, then dispute
			if (evidenceContent) {
				await api.submitEvidence(escrowId, evidenceContent, undefined, auth);
			}
			await api.disputeEscrow(escrowId, reason, undefined);
			setStatus("disputed");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "disputed") {
		return <p className="muted success-text">Disputed — {escrowId}</p>;
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Escrow ID">
				<input
					value={escrowId}
					onChange={(e) => setEscrowId(e.target.value)}
					placeholder="esc_..."
				/>
			</FormField>
			<FormField label="Reason">
				<input
					value={reason}
					onChange={(e) => setReason(e.target.value)}
					placeholder="Why are you disputing?"
				/>
			</FormField>
			<FormField label="Evidence (optional)">
				<textarea
					value={evidenceContent}
					onChange={(e) => setEvidenceContent(e.target.value)}
					placeholder="Describe what happened, attach links, etc."
					className="evidence-input"
				/>
			</FormField>
			<FormField label="Your address">
				<input
					value={authAddress}
					onChange={(e) => setAuthAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Signature">
				<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
					<input
						value={authSig}
						onChange={(e) => setAuthSig(e.target.value)}
						placeholder="auto-filled when signing"
						readOnly={authSig.length > 0}
						style={{ flex: 1 }}
					/>
					<SignWithWallet
						message={`dispute:${escrowId}`}
						onSignature={(sig) => setAuthSig(sig)}
						walletAddress={authAddress}
					/>
				</div>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Submitting…" : "Submit dispute"}
			</button>
		</form>
	);
}
