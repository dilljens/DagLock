import { useEffect, useMemo, useState } from "react";
import {
	api,
	type AuthHeaders,
	type CreateEscrowRequest,
	type CreateOfferRequest,
	type DisputeEvidence,
	type EscrowMessage,
	type Escrow,
	type Health,
	type JuryCase,
	type NetworkInfo,
	type Offer,
	type Receipt,
	type Reputation,
	type Stats,
	type Vault,
	type VaultType,
	type VaultStatus,
} from "./api";

import { money, sompi, time, relativeTime, badge } from "./helpers";
import {
	SectionTitle,
	Panel,
	LookupResult,
	FormField,
	ValidatedInput,
	kvad,
	ConfirmDialog,
	StatusTimeline,
} from "./ui";
import type { LoadState } from "./helpers";
import { detectKasware, connectWallet, signMessage, type WalletState } from "./kasware";

/* ─── Wallet Button ─── */
function WalletStatus() {
	const [wallet, setWallet] = useState<WalletState>({ detected: false, connected: false, address: null, network: null, balance: null, loading: false, error: null });

	useEffect(() => {
		detectKasware().then((detected) => setWallet((s) => ({ ...s, detected })));
	}, []);

	async function handleConnect() {
		setWallet((s) => ({ ...s, loading: true, error: null }));
		try {
			const { address, network, balance } = await connectWallet();
			setWallet({ detected: true, connected: true, address, network, balance, loading: false, error: null });
		} catch (err) {
			setWallet((s) => ({ ...s, loading: false, error: (err as Error).message }));
		}
	}

	return (
		<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
			{!wallet.detected && (
				<small className="muted" style={{ fontSize: "12px" }}>No wallet</small>
			)}
			{wallet.detected && !wallet.connected && (
				<button className="button" onClick={handleConnect} disabled={wallet.loading} style={{ fontSize: "12px", padding: "4px 10px" }}>
					{wallet.loading ? "Connecting..." : "Connect Wallet"}
				</button>
			)}
			{wallet.connected && wallet.address && (
				<small className="muted" style={{ fontSize: "12px" }}>
					{wallet.address.slice(0, 10)}... | {wallet.balance} KAS
				</small>
			)}
		</div>
	);
}

/* ─── Sign With Wallet Button ─── */
function SignWithWallet({ message, onSignature, walletAddress }: { message: string; onSignature: (sig: string) => void; walletAddress: string | null }) {
	const [signing, setSigning] = useState(false);
	const [error, setError] = useState("");

	async function handleSign() {
		if (!window.kasware) {
			setError("KasWare wallet not detected");
			return;
		}
		setSigning(true);
		setError("");
		try {
			const sig = await signMessage(message, "schnorr");
			onSignature(sig);
		} catch (err) {
			setError((err as Error).message || "Signing cancelled");
		} finally {
			setSigning(false);
		}
	}

	return (
		<div>
			<button type="button" className="button" onClick={handleSign} disabled={signing} style={{ fontSize: "12px", padding: "4px 10px" }}>
				{signing ? "Signing..." : "✍️ Sign with Wallet"}
			</button>
			{error && <p className="muted" style={{ fontSize: "12px", color: "#ff7b7b", marginTop: "4px" }}>{error}</p>}
			{walletAddress && <p className="muted" style={{ fontSize: "11px", marginTop: "2px" }}>Signing as {walletAddress.slice(0, 16)}...</p>}
		</div>
	);
}

function CreateOfferForm({ onDone }: { onDone: () => void }) {
	const [side, setSide] = useState("sell");
	const [baseAsset, setBaseAsset] = useState("KAS");
	const [quoteAsset, setQuoteAsset] = useState("USDC");
	const [amount, setAmount] = useState("");
	const [address, setAddress] = useState("");
	const [counterparty, setCounterparty] = useState("");
	const [expireHours, setExpireHours] = useState("72");
	const [priceType, setPriceType] = useState("fixed");
	const [priceOffset, setPriceOffset] = useState("0");
	const [minPrice, setMinPrice] = useState("");
	const [maxPrice, setMaxPrice] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) {
			setError("Invalid amount. Please enter a positive number.");
			return;
		}
		const trimmedAddr = address.trim();
		if (!trimmedAddr.startsWith("kaspa:")) {
			setError(
				"Invalid address format. Must be a valid Kaspa address starting with 'kaspa:'. Check for leading/trailing spaces.",
			);
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const body: CreateOfferRequest = {
				creator_address: trimmedAddr,
				side,
				base_asset: baseAsset,
				quote_asset: quoteAsset,
				amount_sompi: sompi(amountNum),
				expires_at:
					Math.floor(Date.now() / 1000) + (parseInt(expireHours) || 72) * 3600,
				price_type: priceType,
			};
			if (priceType === "market") {
				body.price_offset = parseFloat(priceOffset) || 0;
				if (minPrice) body.min_price = parseFloat(minPrice);
				if (maxPrice) body.max_price = parseFloat(maxPrice);
			}
			if (counterparty.startsWith("kaspa:"))
				body.counterparty_address = counterparty;
			await api.createOffer(body);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done")
		return <p className="muted success-text">Offer created!</p>;

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Side">
				<select value={side} onChange={(e) => setSide(e.target.value)}>
					<option value="sell">Sell</option>
					<option value="buy">Buy</option>
				</select>
			</FormField>
			<FormField label="Sell asset">
				<select value={baseAsset} onChange={(e) => setBaseAsset(e.target.value)}>
					<option value="KAS">KAS</option>
					<option value="KRC20:NACHO">KRC20:NACHO</option>
					<option value="KRC20:KASPY">KRC20:KASPY</option>
					<option value="other">Other...</option>
				</select>
			</FormField>
			<FormField label="For asset">
				<select value={quoteAsset} onChange={(e) => setQuoteAsset(e.target.value)}>
					<option value="USDC">USDC</option>
					<option value="KAS">KAS</option>
					<option value="KRC20:NACHO">KRC20:NACHO</option>
					<option value="KRC20:KASPY">KRC20:KASPY</option>
					<option value="other">Other...</option>
				</select>
			</FormField>
			<FormField label={`Amount (${baseAsset})`}>
				<input
					type="number"
					step="any"
					value={amount}
					onChange={(e) => setAmount(e.target.value)}
					placeholder="100"
				/>
			</FormField>
			{(() => {
				const n = Number.parseFloat(amount) || 0;
				const fee = n * 0.005;
				if (n <= 0) return null;
				return (
					<p className="muted" style={{fontSize: "13px", marginTop: "-8px"}}>
						Fee: {fee.toFixed(4)} KAS (0.5%)
						{n < 1 && <span style={{color: "#ff9800", marginLeft: "8px"}}>⚠️ Low amount — fee may be significant</span>}
					</p>
				);
			})()}
			<ValidatedInput
				label="Your address"
				value={address}
				onChange={setAddress}
				placeholder="kaspa:..."
				validate={kvad}
			/>
			<FormField label="Price type">
				<select value={priceType} onChange={(e) => setPriceType(e.target.value)}>
					<option value="fixed">Fixed price</option>
					<option value="market">Market price (updates every 15 min)</option>
				</select>
			</FormField>
			{priceType === "market" && <>
				<FormField label="Price offset (%)">
					<input type="number" step="0.1" value={priceOffset} onChange={(e) => setPriceOffset(e.target.value)} placeholder="0" />
				</FormField>
				<FormField label="Min price (USD)">
					<input type="number" step="0.001" value={minPrice} onChange={(e) => setMinPrice(e.target.value)} placeholder="0.10" />
				</FormField>
				<FormField label="Max price (USD)">
					<input type="number" step="0.001" value={maxPrice} onChange={(e) => setMaxPrice(e.target.value)} placeholder="0.20" />
				</FormField>
			</>}
			<FormField label="Expires in">
				<select
					value={expireHours}
					onChange={(e) => setExpireHours(e.target.value)}
				>
					<option value="24">24 hours</option>
					<option value="72">3 days</option>
					<option value="168">7 days</option>
					<option value="720">30 days</option>
				</select>
			</FormField>
			<FormField label="Counterparty (optional)">
				<input
					value={counterparty}
					onChange={(e) => setCounterparty(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? "Creating…" : "Create offer"}
			</button>
		</form>
	);
}

/* ─── Create Escrow ─── */
function CreateEscrowForm({ onDone }: { onDone: () => void }) {
	const [amount, setAmount] = useState("");
	const [buyerAddress, setBuyerAddress] = useState("");
	const [sellerAddress, setSellerAddress] = useState("");
	const [assetType, setAssetType] = useState("KAS");
	const [disputeMode, setDisputeMode] = useState("standard");
	const [mediatorKey, setMediatorKey] = useState("");
	const [tradeHash, setTradeHash] = useState("");
	const [tradeSecret, setTradeSecret] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");
	const [result, setResult] = useState<Escrow | null>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		const trimmedBuyer = buyerAddress.trim();
		if (!trimmedBuyer.startsWith("kaspa:")) {
			setError(
				"Invalid buyer address. Must be a valid Kaspa address starting with 'kaspa:'.",
			);
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const body: CreateEscrowRequest = {
				lock_tx_id: crypto.randomUUID(),
				lock_tx_output_index: 0,
				buyer_address: trimmedBuyer,
				amount_sompi: sompi(amountNum),
				asset_type: assetType,
			};
			if (sellerAddress.startsWith("kaspa:"))
				body.seller_address = sellerAddress;
			body.dispute_mode = disputeMode;
			if (disputeMode === "mediator" && mediatorKey.startsWith("kaspa:"))
				body.mediator_key = mediatorKey;
			if (tradeHash.trim())
				body.trade_hash = tradeHash.trim();
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
							${result.price_at_creation.toFixed(4)}{" "}
							{result.price_currency || "USD"}
						</strong>
					</div>
				)}
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Asset type">
				<select
					value={assetType}
					onChange={(e) => setAssetType(e.target.value)}
				>
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
				<select
					value={disputeMode}
					onChange={(e) => setDisputeMode(e.target.value)}
				>
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
					<p className="muted" style={{fontSize:"12px",marginTop:0}}>
						Secret: <code>{tradeSecret}</code>
					</p>
				</FormField>
			)}
			<FormField label="Trade hash (optional)">
				<div style={{display:"flex",gap:"8px",alignItems:"center"}}>
					<input
						value={tradeHash}
						onChange={(e) => setTradeHash(e.target.value)}
						placeholder="Leave empty for non-atomic escrow"
						style={{flex:1}}
					/>
					<button type="button" className="button" onClick={async () => {
						try {
							const res = await api.generateSwap();
							setTradeHash(res.hash);
							setTradeSecret(res.secret);
						} catch (err) {
							setError((err as Error).message);
						}
					}}>
						Generate
					</button>
				</div>
				<small className="muted" style={{fontSize:"11px",marginTop:"4px",display:"block"}}>Save this secret! It's needed to claim the escrow atomically.</small>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? "Creating…" : "Create escrow"}
			</button>
		</form>
	);
}

/* ─── Atomic Swap Form ─── */
function SwapForm({ onDone }: { onDone: () => void }) {
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
			<p className="muted">Submit a preimage to atomically settle an escrow. The preimage must match the trade hash stored in the covenant.</p>
			<FormField label="Escrow ID">
				<input value={escrowId} onChange={(e) => { setEscrowId(e.target.value); fetchEscrow(); }} placeholder="esc_..." />
			</FormField>
			{expectedHash && (
				<div className="row" style={{fontSize:"13px"}}>
					<span>Expected hash</span>
					<code>{expectedHash}</code>
				</div>
			)}
			<FormField label="Preimage (hex)">
				<input value={preimage} onChange={(e) => setPreimage(e.target.value)} placeholder="hex encoded secret" />
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

function DisputeWithEvidenceForm({ onDone }: { onDone: () => void }) {
	const [escrowId, setEscrowId] = useState("");
	const [reason, setReason] = useState("");
	const [evidenceContent, setEvidenceContent] = useState("");
	const [mode, setMode] = useState("standard");
	const [authAddress, setAuthAddress] = useState("");
	const [authSig, setAuthSig] = useState("");
	const [status, setStatus] = useState<
		"idle" | "loading" | "disputed" | "error"
	>("idle");
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!escrowId || !reason) return;
		const trimmedDispAddr = authAddress.trim();
		if (!trimmedDispAddr || !authSig) {
			setError(
				"Authentication required. Please provide your Kaspa address and signature.",
			);
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
			await api.disputeEscrow(
				escrowId,
				reason,
				mode === "jury" ? "jury" : undefined,
			);
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
				<div style={{display:"flex",gap:"8px",alignItems:"center"}}>
					<input value={authSig} onChange={(e) => setAuthSig(e.target.value)} placeholder="auto-filled when signing" readOnly={authSig.length > 0} style={{flex:1}} />
					<SignWithWallet message={`dispute:${escrowId}`} onSignature={(sig) => setAuthSig(sig)} walletAddress={authAddress} />
				</div>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? "Submitting…" : "Submit dispute"}
			</button>
		</form>
	);
}

function EscrowActionForm({ action }: { action: EscrowAction }) {
	const [escrowId, setEscrowId] = useState("");
	const [disputeReason, setDisputeReason] = useState("");
	const [authAddress, setAuthAddress] = useState("");
	const [authSignature, setAuthSignature] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
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

			let res: { status: string; escrow_id: string };
			// Only settle reaches here (cancel/refund/dispute show confirm dialog)
			res = await api.settleEscrow(escrowId, auth!);
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
					setError(
						"Authentication required. Please provide your Kaspa address and signature.",
					);
					return;
				}
				auth = {
					address: authAddress,
					signature: authSignature,
					message: action + ":" + escrowId,
				};
			}
			let res: { status: string; escrow_id: string };
			const act: string = action;
			if (act === "settle") res = await api.settleEscrow(escrowId, auth!);
			else if (act === "refund") res = await api.refundEscrow(escrowId, auth!);
			else if (act === "dispute")
				res = await api.disputeEscrow(escrowId, disputeReason);
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
							<div style={{display:"flex",gap:"8px",alignItems:"center"}}>
								<input
									value={authSignature}
									onChange={(e) => setAuthSignature(e.target.value)}
									placeholder="auto-filled when signing"
									readOnly={authSignature.length > 0}
									style={{flex:1}}
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
			<button
				className={`button ${action === "cancel" ? "" : "primary"}`}
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? `${verb}ing…` : verb}
			</button>
		</form>
	);
}

/* ─── Verify Telegram identity ─── */
function LinkTelegramForm({ onDone }: { onDone: () => void }) {
	const [address, setAddress] = useState("");
	const [telegramHandle, setTelegramHandle] = useState("");
	const [signature, setSignature] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const trimmedTgAddr = address.trim();
		const trimmedTgHandle = telegramHandle.trim();
		if (!trimmedTgAddr.startsWith("kaspa:")) {
			setError(
				"Invalid address format. Must be a valid Kaspa address starting with 'kaspa:'.",
			);
			return;
		}
		if (!trimmedTgHandle.startsWith("@")) {
			setError(
				"Invalid Telegram handle. Must start with '@' (e.g., @username).",
			);
			return;
		}
		if (!signature.trim()) {
			setError(
				"Signature is required for verification. Please sign a message with your Kaspa wallet.",
			);
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const message = `daglock.io:verify:telegram:${trimmedTgHandle}`;
			const auth: AuthHeaders = {
				address: trimmedTgAddr,
				signature: signature.trim(),
				message,
			};
			await api.createIdentity(
				"telegram",
				trimmedTgHandle,
				message,
				signature.trim(),
				auth,
			);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") {
		return <p className="muted success-text">Telegram linked!</p>;
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">
				Sign a message with your Kaspa wallet. The format is:
			</p>
			<code>daglock.io:verify:telegram:YOUR_HANDLE</code>
			<FormField label="Your address">
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Telegram handle">
				<input
					value={telegramHandle}
					onChange={(e) => setTelegramHandle(e.target.value)}
					placeholder="@yourhandle"
				/>
			</FormField>
			<FormField label="Signature">
				<div style={{display:"flex",gap:"8px",alignItems:"center"}}>
					<input value={signature} onChange={(e) => setSignature(e.target.value)} placeholder="auto-filled when signing" readOnly={signature.length > 0} style={{flex:1}} />
					<SignWithWallet message={`daglock.io:verify:telegram:${telegramHandle}`} onSignature={(sig) => setSignature(sig)} walletAddress={address} />
				</div>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? "Verifying…" : "Link Telegram"}
			</button>
		</form>
	);
}

/* ─── Offer item with accept/cancel inline ─── */
function OfferCard({
	offer,
	onMutated,
}: {
	offer: Offer;
	onMutated: () => void;
}) {
	const [status, setStatus] = useState<"idle" | "loading">("idle");
	const [error, setError] = useState("");
	const [counterparty, setCounterparty] = useState("");

	async function handleAccept() {
		if (!counterparty.startsWith("kaspa:")) return;
		setStatus("loading");
		setError("");
		try {
			await api.acceptOffer(offer.id, counterparty);
			onMutated();
		} catch (err) {
			setError((err as Error).message);
			setStatus("idle");
		}
	}

	async function handleCancel() {
		setStatus("loading");
		setError("");
		try {
			await api.cancelOffer(offer.id);
			onMutated();
		} catch (err) {
			setError((err as Error).message);
			setStatus("idle");
		}
	}

	const canAct = offer.status === "proposed";

	return (
		<article className="offer">
			<div className="offer-top">
				<strong>
					{offer.side.toUpperCase()} {money(offer.amount_sompi)}
				</strong>
				<span className={badge(offer.status)}>{offer.status}</span>
			</div>
			<p>
				{offer.base_asset} for {offer.quote_asset}
			</p>
			{offer.price_type === "market" && offer.current_price && (
				<small className="muted">Market price: ${offer.current_price.toFixed(4)} USD</small>
			)}
			<small className="muted addr">
				by {offer.creator_address.slice(0, 24)}…
			</small>
			<code>{offer.id}</code>
			<small className="muted">{relativeTime(offer.created_at)}</small>
			{canAct && (
				<div className="offer-actions">
					<input
						value={counterparty}
						onChange={(e) => setCounterparty(e.target.value)}
						placeholder="your kaspa address"
						className="offer-input"
					/>
					<button
						className="button primary"
						disabled={status === "loading"}
						onClick={handleAccept}
					>
						Accept
					</button>
					<button
						className="button"
						disabled={status === "loading"}
						onClick={handleCancel}
					>
						Cancel
					</button>
				</div>
			)}
			{error && <p className="muted error-text">{error}</p>}
		</article>
	);
}

/* ─── Lookup panels ─── */
function EscrowLookup() {
	const [id, setId] = useState("");
	const [state, setState] = useState<LoadState<Escrow>>({ loading: false });
	const [evidence, setEvidence] = useState<LoadState<DisputeEvidence[]>>({
		loading: false,
	});
	const [messages, setMessages] = useState<LoadState<EscrowMessage[]>>({
		loading: false,
	});
	const [msgText, setMsgText] = useState("");
	const [chatAddr, setChatAddr] = useState("");
	const [chatSig, setChatSig] = useState("");
	const [msgStatus, setMsgStatus] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!id) return;
		setState({ loading: true });
		try {
			const data = await api.escrow(id.trim());
			setState({ data, loading: false });
			// Also fetch messages (requires auth)
			if (chatAddr && chatSig) {
				const cauth: AuthHeaders = {
					address: chatAddr,
					signature: chatSig,
					message: "messages",
				};
				api
					.listMessages(id.trim(), cauth)
					.then((r) => setMessages({ data: r.messages, loading: false }))
					.catch(() => setMessages({ loading: false }));
			}
			// Also fetch evidence if disputed
			if (data.status === "disputed") {
				setEvidence({ loading: true });
				api
					.listEvidence(id.trim())
					.then((r) => setEvidence({ data: r.evidence, loading: false }))
					.catch(() => setEvidence({ loading: false }));
			} else {
				setEvidence({ loading: false });
			}
		} catch (err) {
			setState({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<Panel title="Escrow lookup">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={id}
					onChange={(e) => setId(e.target.value)}
					placeholder="escrow id"
				/>
				<button className="button primary" type="submit">
					Fetch
				</button>
			</form>
			<div className="form form-stacked" style={{ marginTop: 8 }}>
				<FormField label="Address (for chat)">
					<input
						value={chatAddr}
						onChange={(e) => setChatAddr(e.target.value)}
						placeholder="kaspa:..."
					/>
				</FormField>
				<FormField label="Signature">
					<input
						value={chatSig}
						onChange={(e) => setChatSig(e.target.value)}
						placeholder="hex"
					/>
				</FormField>
			</div>
			<LookupResult
				loading={state.loading}
				error={state.error}
				data={state.data}
				render={(data) => (
					<div className="stack">
						<div className="row">
							<span>Status</span>
							<strong>
								<span className={badge(data.status)}>{data.status}</span>
							</strong>
						</div>
						<div className="row">
							<span>Amount</span>
							<strong>{money(data.amount_sompi)}</strong>
						</div>
						<div className="row">
							<span>Fee (0.5%)</span>
							<strong>{money(data.fee_sompi)}</strong>
						</div>
						{data.price_at_creation != null && (
							<div className="row">
								<span>Price</span>
								<strong>
									${data.price_at_creation.toFixed(4)}{" "}
									{data.price_currency || "USD"}
								</strong>
							</div>
						)}
						{data.dispute_mode && (
							<div className="row">
								<span>Dispute mode</span>
								<strong>
									<span className={badge(data.dispute_mode)}>
										{data.dispute_mode}
									</span>
								</strong>
							</div>
						)}
						<div className="row">
							<span>Buyer</span>
							<strong className="addr">{data.buyer_address}</strong>
						</div>
						{data.mediator_key && (
							<div className="row">
								<span>Mediator</span>
								<strong className="addr">{data.mediator_key}</strong>
							</div>
						)}
						<div className="row">
							<span>Created</span>
							<strong>{time(data.created_at)}</strong>
						</div>
						{data.dispute_reason && (
							<div className="row">
								<span>Dispute</span>
								<strong>{data.dispute_reason}</strong>
							</div>
						)}
						{data.dispute_outcome && (
							<div className="row">
								<span>Outcome</span>
								<strong>{data.dispute_outcome}</strong>
							</div>
						)}
						{evidence.data && evidence.data.length > 0 && (
							<div className="evidence-log">
								<h4>Evidence ({evidence.data.length})</h4>
								{evidence.data.map((ev) => (
									<div key={ev.id} className="evidence-item">
										<div className="row">
											<span className="addr">{ev.submitted_by}</span>
											<small>{time(ev.created_at)}</small>
										</div>
										<p className="evidence-content">{ev.content}</p>
									</div>
								))}
							</div>
						)}
						{messages.data && (
							<div className="evidence-log">
								<h4>Messages ({messages.data.length})</h4>
								{messages.data.map((m, i) => (
									<div key={i} className="evidence-item">
										<div className="row">
											<span className="addr">
												{m.sender_address.slice(0, 20)}…
											</span>
											<small>{time(m.created_at)}</small>
										</div>
										<p className="evidence-content">{m.content}</p>
									</div>
								))}
							</div>
						)}
					</div>
				)}
			/>
			{messages.data != null && (
				<div className="form form-stacked" style={{ marginTop: 12 }}>
					<div className="form">
						<input
							value={msgText}
							onChange={(e) => setMsgText(e.target.value)}
							placeholder="Type a message..."
						/>
						<button
							className="button primary"
							onClick={async () => {
								if (!msgText || !id) return;
								if (!chatAddr || !chatSig) {
									setMsgStatus(
										"Authentication required. Please provide your address and signature.",
									);
									return;
								}
								try {
									const cauth: AuthHeaders = {
										address: chatAddr,
										signature: chatSig,
										message: "messages",
									};
									await api.sendMessage(id, msgText, cauth);
									setMsgText("");
									setMsgStatus("Sent");
									const r = await api.listMessages(id, cauth);
									setMessages({ data: r.messages, loading: false });
								} catch (err) {
									setMsgStatus((err as Error).message);
								}
							}}
						>
							Send
						</button>
					</div>
					{msgStatus && <p className="muted">{msgStatus}</p>}
				</div>
			)}
		</Panel>
	);

	/* ─── Auth for chat ─── */
	const chatAuthSection = (
		<div className="form form-stacked" style={{ marginTop: 8 }}>
			<FormField label="Address (for chat)">
				<input
					value={chatAddr}
					onChange={(e) => setChatAddr(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Signature">
				<input
					value={chatSig}
					onChange={(e) => setChatSig(e.target.value)}
					placeholder="hex"
				/>
			</FormField>
		</div>
	);
}

/* ─── Receipt lookup ─── */

function ReputationLookup() {
	const [address, setAddress] = useState("");
	const [state, setState] = useState<LoadState<Reputation>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address) return;
		setState({ loading: true });
		try {
			setState({ data: await api.reputation(address.trim()), loading: false });
		} catch (err) {
			setState({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<Panel title="Reputation">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="kaspa address"
				/>
				<button className="button" type="submit">
					Check
				</button>
			</form>
			<LookupResult
				loading={state.loading}
				error={state.error}
				data={state.data}
				render={(data) => (
					<div className="stack">
						<div className="row">
							<span>Score</span>
							<strong>{data.score.toFixed(2)}/5</strong>
						</div>
						<div className="row">
							<span>Trades</span>
							<strong>
								{data.trade_count} ({data.recent_trade_count} in last 90d)
							</strong>
						</div>
						<div className="row">
							<span>Volume</span>
							<strong>{money(data.total_volume_sompi)}</strong>
						</div>
						<div className="row">
							<span>Refund rate</span>
							<strong>{(data.refund_rate * 100).toFixed(1)}%</strong>
						</div>
						<div className="row">
							<span>Dispute rate</span>
							<strong>{(data.dispute_rate * 100).toFixed(1)}%</strong>
						</div>
						<div className="row">
							<span>Age</span>
							<strong>{data.age_days} days</strong>
						</div>
						{data.telegram_handle && (
							<div className="row">
								<span>Telegram</span>
								<strong>{data.telegram_handle}</strong>
							</div>
						)}
						<div className="row">
							<span>Vouches</span>
							<strong>
								{data.vouches_received} received / {data.vouches_given} given
							</strong>
						</div>
						{data.vouch_score != null && (
							<div className="row">
								<span>Vouch score</span>
								<strong>{data.vouch_score.toFixed(2)}/5</strong>
							</div>
						)}
						{data.trading_concentration > 0.9 && (
							<div className="row">
								<span>Wash trading</span>
								<strong className="error-text">
									Warning: {(data.trading_concentration * 100).toFixed(0)}%
									volume with one counterparty
								</strong>
							</div>
						)}
						{data.mediator_stats && (
							<div className="row">
								<span>Mediator</span>
								<strong>
									{data.mediator_stats.score.toFixed(2)}/5 (
									{data.mediator_stats.disputes_mediated} cases)
								</strong>
							</div>
						)}
					</div>
				)}
			/>
		</Panel>
	);
}

function MyEscrows() {
	const [address, setAddress] = useState("");
	const [list, setList] = useState<LoadState<Escrow[]>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address.trim()) return;
		setList({ loading: true });
		try {
			const data = await api.escrows(address.trim());
			setList({ data: data.escrows, loading: false });
		} catch (err) {
			setList({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<Panel title="My escrows">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="your kaspa address"
				/>
				<button className="button" type="submit">
					List
				</button>
			</form>
			<LookupResult
				loading={list.loading}
				error={list.error}
				data={list.data}
				render={(data) => (
					<div className="stack">
						{data.length === 0 && (
							<p className="muted">No escrows found for this address.</p>
						)}
						{data.map((e) => (
							<article
								key={e.id}
								className="offer"
								style={{ cursor: "default" }}
							>
								<div className="offer-top">
									<strong>{money(e.amount_sompi)}</strong>
									<span className={badge(e.status)}>{e.status}</span>
								</div>
								<p>{e.asset_type} escrow</p>
								<code>{e.id}</code>
								<small className="muted">{relativeTime(e.created_at)}</small>
							</article>
						))}
					</div>
				)}
			/>
		</Panel>
	);
}

function ReceiptLookup() {
	const [id, setId] = useState("");
	const [state, setState] = useState<LoadState<Receipt>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!id) return;
		setState({ loading: true });
		try {
			setState({ data: await api.receipt(id.trim()), loading: false });
		} catch (err) {
			setState({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<Panel title="Receipt lookup">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={id}
					onChange={(e) => setId(e.target.value)}
					placeholder="escrow id"
				/>
				<button className="button" type="submit">
					Fetch
				</button>
			</form>
			<LookupResult
				loading={state.loading}
				error={state.error}
				data={state.data}
				render={(data) => (
					<div className="stack">
						<div className="row">
							<span>ID</span>
							<strong>{data.receipt_id}</strong>
						</div>
						<div className="row">
							<span>Status</span>
							<strong>{data.status}</strong>
						</div>
						<div className="row">
							<span>Amount</span>
							<strong>{money(data.amount_sompi)}</strong>
						</div>
					</div>
				)}
			/>
		</Panel>
	);
}

/* ─── Jury Panel ─── */
function JuryPanel() {
	const [regStatus, setRegStatus] = useState<
		"idle" | "loading" | "registered" | "error"
	>("idle");
	const [regError, setRegError] = useState("");
	const [authAddr, setAuthAddr] = useState("");
	const [authSig, setAuthSig] = useState("");
	const [cases, setCases] = useState<LoadState<JuryCase[]>>({ loading: false });
	const [selectedCase, setSelectedCase] = useState<JuryCase | null>(null);
	const [vote, setVote] = useState("");
	const [reasoning, setReasoning] = useState("");
	const [voteResult, setVoteResult] = useState("");

	function makeAuth() {
		if (!authAddr || !authSig) return undefined;
		return {
			address: authAddr,
			signature: authSig,
			message: "jury:auth",
		} as AuthHeaders;
	}

	async function handleRegister() {
		const a = makeAuth();
		if (!a) return;
		setRegStatus("loading");
		try {
			await api.juryRegister(a);
			setRegStatus("registered");
		} catch (err) {
			setRegStatus("error");
			setRegError((err as Error).message);
		}
	}

	async function handleUnregister() {
		const a = makeAuth();
		if (!a) return;
		setRegStatus("loading");
		try {
			await api.juryUnregister(a);
			setRegStatus("idle");
		} catch (err) {
			setRegStatus("error");
			setRegError((err as Error).message);
		}
	}

	async function loadCases() {
		const a = makeAuth();
		if (!a) return;
		setCases({ loading: true });
		try {
			const r = await api.juryCases(a);
			setCases({ data: r.cases, loading: false });
		} catch (err) {
			setCases({ error: (err as Error).message, loading: false });
		}
	}

	async function handleVote() {
		if (!selectedCase || !vote) return;
		const a = makeAuth();
		if (!a) return;
		try {
			const r = await api.juryVote(
				selectedCase.id,
				vote,
				reasoning || undefined,
				a,
			);
			setVoteResult(
				r.verdict ? `Verdict: ${r.vote} (case decided)` : `Voted: ${r.vote}`,
			);
			loadCases();
		} catch (err) {
			setVoteResult(`Error: ${(err as Error).message}`);
		}
	}

	return (
		<div className="stack">
			<FormField label="Your address">
				<input
					value={authAddr}
					onChange={(e) => setAuthAddr(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Signature (hex)">
				<input
					value={authSig}
					onChange={(e) => setAuthSig(e.target.value)}
					placeholder="hex signature"
				/>
			</FormField>
			<div className="action-tabs">
				<button
					className="button primary"
					onClick={handleRegister}
					disabled={regStatus === "loading"}
				>
					{regStatus === "loading" ? "Registering…" : "Register as juror"}
				</button>
				<button className="button" onClick={handleUnregister}>
					Unregister
				</button>
				<button className="button" onClick={loadCases}>
					Load my cases
				</button>
			</div>
			{regStatus === "registered" && (
				<p className="muted success-text">Registered as juror!</p>
			)}
			{regError && <p className="muted error-text">{regError}</p>}
			{voteResult && <p className="muted">{voteResult}</p>}

			{cases.loading && <p className="muted">Loading cases…</p>}
			{cases.data && cases.data.length === 0 && (
				<p className="muted">No active cases assigned to you.</p>
			)}
			{cases.data?.map((c) => (
				<article
					key={c.id}
					className="offer"
					onClick={() => setSelectedCase(c)}
				>
					<div className="offer-top">
						<strong>Case: {c.id.slice(0, 16)}…</strong>
						<span className={badge(c.status)}>{c.status}</span>
					</div>
					<p>
						Escrow: {c.escrow_id} | Votes:{" "}
						{c.votes_for_seller + c.votes_for_buyer}/{c.juror_count} |
						Threshold: {c.threshold}
					</p>
				</article>
			))}

			{selectedCase && selectedCase.status === "voting" && (
				<div className="panel">
					<h4>Cast vote for {selectedCase.id.slice(0, 16)}…</h4>
					<FormField label="Vote">
						<select value={vote} onChange={(e) => setVote(e.target.value)}>
							<option value="">— select —</option>
							<option value="seller_wins">Seller wins</option>
							<option value="buyer_wins">Buyer wins</option>
						</select>
					</FormField>
					<FormField label="Reasoning (optional)">
						<input
							value={reasoning}
							onChange={(e) => setReasoning(e.target.value)}
							placeholder="Why?"
						/>
					</FormField>
					<button className="button primary" onClick={handleVote}>
						Submit vote
					</button>
				</div>
			)}
		</div>
	);
}

/* ─── Vault Lookup Panel ─── */
function VaultLookup() {
	const [vaultId, setVaultId] = useState("");
	const [vault, setVault] = useState<Vault | null>(null);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!vaultId.trim()) return;
		setLoading(true);
		setError("");
		try {
			const data = await api.vault(vaultId.trim());
			setVault(data);
		} catch (err) {
			setError((err as Error).message);
			setVault(null);
		} finally {
			setLoading(false);
		}
	}

	return (
		<div className="panel">
			<div className="panel-head">
				<h3>Vault lookup</h3>
			</div>
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={vaultId}
					onChange={(e) => setVaultId(e.target.value)}
					placeholder="vault id (vault_...)"
				/>
				<button className="button" type="submit" disabled={loading}>
					{loading ? "Loading…" : "Fetch"}
				</button>
			</form>
			{error && <p className="muted error-text">{error}</p>}
			{vault && (
				<VaultStatusPanel
					vault={vault}
					onWithdraw={() => {
						setVault(null);
						setVaultId("");
					}}
				/>
			)}
		</div>
	);
}

/* ─── Vault List Panel ─── */
function VaultListPanel() {
	const [address, setAddress] = useState("");
	const [list, setList] = useState<LoadState<Vault[]>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address.trim()) return;
		setList({ loading: true });
		try {
			const data = await api.vaults(address.trim());
			setList({ data: data.vaults, loading: false });
		} catch (err) {
			setList({ error: (err as Error).message, loading: false });
		}
	}

	function formatVaultType(type: VaultType): string {
		switch (type) {
			case "time":
				return "Time-locked";
			case "beneficiary":
				return "Beneficiary";
			case "deadman":
				return "Deadman switch";
			case "inheritance":
				return "Inheritance";
			case "multisig":
				return "Multi-sig";
			default:
				return type;
		}
	}

	function formatVaultStatus(status: VaultStatus): string {
		switch (status) {
			case "locked":
				return "🔒 Locked";
			case "unlocked":
				return "🔓 Unlocked";
			case "expired":
				return "⏰ Expired";
			case "transferred":
				return "↗️ Transferred";
			default:
				return status;
		}
	}

	return (
		<div className="stack">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="your kaspa address"
				/>
				<button className="button" type="submit">
					List my vaults
				</button>
			</form>
			{list.loading && <p className="muted">Loading vaults…</p>}
			{list.error && <p className="muted error-text">{list.error}</p>}
			{list.data?.length === 0 && (
				<p className="muted">No vaults found for this address.</p>
			)}
			{list.data?.map((v) => (
				<article key={v.id} className="offer" style={{ cursor: "default" }}>
					<div className="offer-top">
						<strong>{formatVaultType(v.vault_type)}</strong>
						<span className={`pill pill-${v.status}`}>
							{formatVaultStatus(v.status)}
						</span>
					</div>
					<p>{money(v.amount_sompi)} KAS</p>
					<small className="muted">Expires: {time(v.timeout)}</small>
					<code>{v.id}</code>
				</article>
			))}
		</div>
	);
}

/* ─── Vault Status Panel ─── */
function VaultStatusPanel({
	vault,
	onWithdraw,
}: {
	vault: Vault;
	onWithdraw: () => void;
}) {
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");

	const now = Math.floor(Date.now() / 1000);
	const isLocked = vault.status === "locked";
	const canWithdraw = isLocked && now >= vault.timeout;
	const timeRemaining = vault.timeout - now;

	function formatTimeRemaining(seconds: number): string {
		if (seconds <= 0) return "Ready to withdraw";
		const days = Math.floor(seconds / 86400);
		const hours = Math.floor((seconds % 86400) / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (days > 0) return `${days}d ${hours}h remaining`;
		if (hours > 0) return `${hours}h ${minutes}m remaining`;
		return `${minutes}m remaining`;
	}

	async function handleWithdraw() {
		const address = prompt("Enter your Kaspa address:");
		if (!address) return;
		const signature = prompt("Enter your signature (hex):");
		if (!signature) return;

		setStatus("loading");
		try {
			await api.withdrawVault(vault.id, address, signature);
			setStatus("done");
			onWithdraw();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	return (
		<div className="panel">
			<div className="panel-head">
				<h3>Vault Status</h3>
			</div>
			<div className="stack">
				<div className="row">
					<span>Type</span>
					<strong>{vault.vault_type}</strong>
				</div>
				<div className="row">
					<span>Amount</span>
					<strong>{money(vault.amount_sompi)} KAS</strong>
				</div>
				<div className="row">
					<span>Status</span>
					<strong className={isLocked ? "error-text" : "success-text"}>
						{isLocked ? "🔒 Locked" : "🔓 Unlocked"}
					</strong>
				</div>
				<div className="row">
					<span>Timeout</span>
					<strong>{time(vault.timeout)}</strong>
				</div>
				<div className="row">
					<span>Time</span>
					<strong className={canWithdraw ? "success-text" : ""}>
						{formatTimeRemaining(timeRemaining)}
					</strong>
				</div>
				{vault.beneficiary_address && (
					<div className="row">
						<span>Beneficiary</span>
						<strong className="addr">{vault.beneficiary_address}</strong>
					</div>
				)}
				<div className="row">
					<span>Created</span>
					<strong>{time(vault.created_at)}</strong>
				</div>
				{status === "done" && (
					<p className="muted success-text">Vault unlocked successfully!</p>
				)}
				{error && <p className="muted error-text">{error}</p>}
				{canWithdraw && (
					<button
						className="button primary"
						onClick={handleWithdraw}
						disabled={status === "loading"}
					>
						{status === "loading" ? "Withdrawing…" : "Withdraw"}
					</button>
				)}
			</div>
		</div>
	);
}

function MyOffersPanel() {
	const [address, setAddress] = useState("");
	const [filter, setFilter] = useState("all");
	const [list, setList] = useState<LoadState<Offer[]>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address.trim()) return;
		setList({ loading: true });
		try {
			const data = await api.offers(address.trim());
			setList({ data: data.offers, loading: false });
		} catch (err) {
			setList({ error: (err as Error).message, loading: false });
		}
	}

	const filtered = list.data?.filter((o) => {
		if (filter === "all") return true;
		return o.status === filter;
	});

	return (
		<div className="stack">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="your kaspa address"
				/>
				<button className="button" type="submit">
					List my offers
				</button>
			</form>
			{list.data && list.data.length > 0 && (
				<div className="action-tabs" style={{ marginTop: "8px" }}>
					{["all", "proposed", "accepted", "cancelled"].map((f) => (
						<button key={f} className={`button ${filter === f ? "primary" : ""}`} onClick={() => setFilter(f)} style={{ fontSize: "11px", padding: "2px 8px" }}>
							{f === "all" ? "All" : f.charAt(0).toUpperCase() + f.slice(1)}
						</button>
					))}
				</div>
			)}
			<LookupResult
				loading={list.loading}
				error={list.error}
				data={filtered}
				render={(data) => (
					<div>
						{data.length === 0 && (
							<p className="muted">No {filter === "all" ? "" : filter} offers found for this address.</p>
						)}
						{data.map((o) => (
							<article
								key={o.id}
								className="offer"
								style={{ cursor: "default" }}
							>
								<div className="offer-top">
									<strong>
										{o.side.toUpperCase()} {money(o.amount_sompi)}
									</strong>
									<span className={badge(o.status)}>{o.status}</span>
								</div>
								<p>
									{o.base_asset} for {o.quote_asset}
								</p>
								<code>{o.id}</code>
								<small className="muted">{relativeTime(o.created_at)}</small>
								{o.expires_at && (
									<small className="muted">
										Expires: {relativeTime(o.expires_at)}
									</small>
								)}
							</article>
						))}
					</div>
				)}
			/>
		</div>
	);
}

/* ─── Confirmation Dialog ─── */

/* ─── Status Timeline ─── */

/* ─── Create Vault Form ─── */
function CreateVaultForm({ onDone }: { onDone: () => void }) {
	const [ownerAddress, setOwnerAddress] = useState("");
	const [amount, setAmount] = useState("");
	const [timeoutDays, setTimeoutDays] = useState("30");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");
	const [result, setResult] = useState<{
		script: string;
		template_hash: string;
	} | null>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const trimmedAddress = ownerAddress.trim();
		if (!trimmedAddress || !trimmedAddress.startsWith("kaspa:")) {
			setError("Enter a valid Kaspa address starting with 'kaspa:'");
			return;
		}
		const amountNum = parseFloat(amount);
		if (!amountNum || amountNum <= 0) {
			setError("Amount must be a positive number");
			return;
		}

		const timeoutSec =
			Math.floor(Date.now() / 1000) + (parseInt(timeoutDays) || 30) * 86400;
		setStatus("loading");
		setError("");

		try {
			// Create vault entry in database
			const vault = await api.createVault({
				owner_address: trimmedAddress,
				vault_type: "time",
				amount_sompi: Math.round(amountNum * 100_000_000),
				timeout: timeoutSec,
			});

			setResult({
				script: "Vault created",
				template_hash: vault.id,
			});
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
				<p className="muted success-text">Vault created!</p>
				<div className="row">
					<span>Vault ID</span>
					<code>{result.template_hash}</code>
				</div>
				<p className="muted">
					Your vault is now locked. You can withdraw after the timeout expires.
				</p>
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">
				Create a time-locked KAS vault. Only the owner can withdraw after the
				timeout.
			</p>
			<FormField label="Owner address">
				<input
					value={ownerAddress}
					onChange={(e) => setOwnerAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
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
			<FormField label="Lock duration">
				<select
					value={timeoutDays}
					onChange={(e) => setTimeoutDays(e.target.value)}
				>
					<option value="1">1 day</option>
					<option value="7">7 days</option>
					<option value="30">30 days</option>
					<option value="90">90 days</option>
					<option value="365">1 year</option>
				</select>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? "Creating…" : "Create vault"}
			</button>
		</form>
	);
}

/* ─── Compile Covenant Form ─── */
function CompileCovenantForm({ onDone }: { onDone: () => void }) {
	const [template, setTemplate] = useState("daglock");
	const [paramsStr, setParamsStr] = useState("{}");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");
	const [result, setResult] = useState<any>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		let params: Record<string, string>;
		try {
			params = JSON.parse(paramsStr);
		} catch {
			setError("Params must be valid JSON");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const r = await api.compile(template, params);
			setResult(r);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Template">
				<select value={template} onChange={(e) => setTemplate(e.target.value)}>
					<option value="daglock">DagLock (KAS escrow)</option>
					<option value="daglock_arbiter">
						DagLock Arbiter (with mediator)
					</option>
					<option value="daglock_vault">DagLock Vault (time-locked)</option>
				</select>
			</FormField>
			<FormField label="Params (JSON)">
				<textarea
					value={paramsStr}
					onChange={(e) => setParamsStr(e.target.value)}
					className="evidence-input"
					placeholder='{"buyer_key":"...","seller_key":"...","timeout":"1700000000","treasury_key":"..."}'
				/>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
			>
				{status === "loading" ? "Compiling…" : "Compile"}
			</button>
			{result && (
				<pre className="muted" style={{ fontSize: "0.7rem", marginTop: 8 }}>
					{JSON.stringify(result, null, 2)}
				</pre>
			)}
		</form>
	);
}

/* ─── Main App ─── */
export default function App() {
	const [health, setHealth] = useState<LoadState<Health>>({ loading: true });
	const [network, setNetwork] = useState<LoadState<NetworkInfo>>({
		loading: true,
	});
	const [stats, setStats] = useState<LoadState<Stats>>({ loading: true });
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
	const [activeTab, setActiveTab] = useState<
		| "create-vault"
		| "compile"
		| "create-offer"
		| "create-escrow"
		| "settle"
		| "refund"
		| "swap"
		| "dispute"
		| "cancel"
		| "my-offers"
		| "link-telegram"
		| "jury"
		| null
	>(null);

	function loadAll() {
		setHealth({ loading: true });
		setNetwork({ loading: true });
		setStats({ loading: true });
		setOffers({ loading: true });
		void Promise.all([
			api
				.health()
				.then((data) => setHealth({ data, loading: false }))
				.catch((err) => setHealth({ error: err.message, loading: false })),
			api
				.network()
				.then((data) => setNetwork({ data, loading: false }))
				.catch((err) => setNetwork({ error: err.message, loading: false })),
			api
				.stats()
				.then((data) => setStats({ data, loading: false }))
				.catch((err) => setStats({ error: err.message, loading: false })),
			api
				.offers()
				.then((data) => setOffers({ data: data.offers, loading: false }))
				.catch((err) => setOffers({ error: err.message, loading: false })),
		]);
	}

	useEffect(loadAll, []);

	const highlights = useMemo(() => {
		const s = stats.data;
		return [
			["Escrows", s?.total_escrows ?? "—"],
			["Active", s?.active_escrows ?? "—"],
			["Volume", s ? money(s.total_volume_kas) : "—"],
			["Settled", s?.settled_escrows ?? "—"],
		];
	}, [stats.data]);

	function closeTab() {
		setActiveTab(null);
		loadAll();
	}

	const tabPanels: Record<string, { title: string; content: React.ReactNode }> =
		{
			"create-vault": {
				title: "Create vault",
				content: <CreateVaultForm onDone={closeTab} />,
			},
			compile: {
				title: "Compile covenant",
				content: <CompileCovenantForm onDone={closeTab} />,
			},
			"create-offer": {
				title: "Create offer",
				content: <CreateOfferForm onDone={closeTab} />,
			},
			"create-escrow": {
				title: "Create escrow",
				content: <CreateEscrowForm onDone={closeTab} />,
			},
			settle: {
				title: "Settle escrow",
				content: <EscrowActionForm action="settle" />,
			},
			refund: {
				title: "Refund escrow",
				content: <EscrowActionForm action="refund" />,
			},
			swap: {
				title: "Atomic Swap",
				content: <SwapForm onDone={closeTab} />,
			},
			dispute: {
				title: "Dispute escrow",
				content: <DisputeWithEvidenceForm onDone={closeTab} />,
			},
			cancel: {
				title: "Cancel escrow",
				content: <EscrowActionForm action="cancel" />,
			},
			"link-telegram": {
				title: "Link Telegram",
				content: <LinkTelegramForm onDone={closeTab} />,
			},
			jury: {
				title: "Jury panel",
				content: <JuryPanel />,
			},
		};

	return (
		<main className="app">
			<div
				style={{
					background: "#ff9800",
					color: "#000",
					textAlign: "center",
					padding: "8px",
					fontWeight: "bold",
					fontSize: "14px",
				}}
			>
				⚠️ TESTNET — This is a testnet deployment. Do not use real funds. Get
				testnet KAS from the{" "}
				<a
					href="https://faucet-tn10.kaspanet.io/"
					target="_blank"
					rel="noopener noreferrer"
					style={{ color: "#000", textDecoration: "underline" }}
				>
					Kaspa Testnet Faucet
				</a>
				.
			</div>
			<div style={{background:'#1a3a1a',border:'1px solid rgba(83,215,105,0.3)',borderRadius:'8px',padding:'16px',marginTop:'8px',marginBottom:'8px'}}>
				<strong>🚀 Getting Started</strong>
				<ol style={{margin:'8px 0 0 0',paddingLeft:'20px',fontSize:'13px',lineHeight:1.8}}>
					<li>Install <a href="https://kasware.xyz" target="_blank" rel="noopener noreferrer" style={{color:'#53d769'}}>KasWare</a> browser extension</li>
					<li>Get testnet KAS from <a href="https://faucet-tn10.kaspanet.io/" target="_blank" rel="noopener noreferrer" style={{color:'#53d769'}}>Testnet Faucet</a></li>
					<li>Connect your wallet using the button in the header</li>
					<li>Create an offer or escrow below</li>
				</ol>
			</div>
			<header className="hero">
				<div>
					<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", width: "100%", marginBottom: "8px" }}>
						<div className="brand">Kaspa Escrow</div>
						<WalletStatus />
					</div>
					<h1>Trustless escrow and atomic swaps on Kaspa.</h1>
					<p>
						The public front door for offers, escrows, reputation, and receipts.
					</p>
				</div>
				<div className="hero-actions">
					<a href="#offers" className="button primary">
						Browse offers
					</a>
					<a href="#actions" className="button">
						Take action
					</a>
				</div>
			</header>

			<section className="grid cards">
				{highlights.map(([label, value]) => (
					<article key={label} className="card">
						<span>{label}</span>
						<strong>{value}</strong>
					</article>
				))}
			</section>

			<section className="grid two-up">
				<Panel title="Network">
					{health.error || network.error ? (
						<p className="muted">{health.error || network.error}</p>
					) : (
						<div className="stack">
							<div className="row">
								<span>API</span>
								<strong>{health.data?.status ?? "—"}</strong>
							</div>
							<div className="row">
								<span>Network</span>
								<strong>{network.data?.network ?? "—"}</strong>
							</div>
							<div className="row">
								<span>Version</span>
								<strong>{health.data?.version ?? "—"}</strong>
							</div>
							<div className="row">
								<span>Fee tier</span>
								<strong>0.5%</strong>
							</div>
						</div>
					)}
				</Panel>

				<Panel title="Public stats">
					{stats.data ? (
						<div className="stack">
							<div className="row">
								<span>Total escrows</span>
								<strong>{stats.data.total_escrows}</strong>
							</div>
							<div className="row">
								<span>Settled</span>
								<strong>{stats.data.settled_escrows}</strong>
							</div>
							<div className="row">
								<span>Disputed</span>
								<strong>{stats.data.disputed_escrows}</strong>
							</div>
							<div className="row">
								<span>Fees</span>
								<strong>{money(stats.data.total_fees_collected_kas)}</strong>
							</div>
						</div>
					) : (
						<p className="muted">Loading stats…</p>
					)}
				</Panel>
			</section>

			<section id="offers">
				<SectionTitle
					title="Open offers"
					subtitle="Public listings available to counterparties."
				/>
				<div className="offers">
					{offers.loading && <p className="muted">Loading offers…</p>}
					{offers.error && <p className="muted error-text">{offers.error}</p>}
					{offers.data?.length === 0 && (
						<p className="muted">No open offers right now. Create one below!</p>
					)}
					{offers.data?.map((offer) => (
						<OfferCard key={offer.id} offer={offer} onMutated={loadAll} />
					))}
				</div>
			</section>

			<section id="actions" className="actions-section">
				<SectionTitle
					title="Actions"
					subtitle="Create offers & escrows, settle, refund, dispute, or cancel."
				/>

				<div className="action-tabs">
					<div className="action-group">
						<span className="action-group-label">Create</span>
						{(
							[
								["Offer", "create-offer"],
								["Escrow", "create-escrow"],
								["Vault", "create-vault"],
							] as const
						).map(([label, key]) => (
							<button
								key={key}
								className={`button ${activeTab === key ? "primary" : ""}`}
								onClick={() =>
									setActiveTab(
										activeTab === key ? null : (key as typeof activeTab),
									)
								}
							>
								{label}
							</button>
						))}
					</div>
					<div className="action-group">
						<span className="action-group-label">Manage</span>
						{(
							[
								["Settle", "settle"],
								["Refund", "refund"],
								["Dispute", "dispute"],
								["Cancel", "cancel"],
							] as const
						).map(([label, key]) => (
							<button
								key={key}
								className={`button ${activeTab === key ? "primary" : ""}`}
								onClick={() =>
									setActiveTab(
										activeTab === key ? null : (key as typeof activeTab),
									)
								}
							>
								{label}
							</button>
						))}
					</div>
					<div className="action-group">
						<span className="action-group-label">Account</span>
						{(
							[
								["My offers", "my-offers"],
								["Telegram", "link-telegram"],
								["Jury", "jury"],
							] as const
						).map(([label, key]) => (
							<button
								key={key}
								className={`button ${activeTab === key ? "primary" : ""}`}
								onClick={() =>
									setActiveTab(
										activeTab === key ? null : (key as typeof activeTab),
									)
								}
							>
								{label}
							</button>
						))}
					</div>
				</div>

				{activeTab && (
					<div className="panel action-panel">
						<div className="panel-head">
							<h3>{tabPanels[activeTab].title}</h3>
							<button className="button" onClick={closeTab}>
								✕
							</button>
						</div>
						{tabPanels[activeTab].content}
					</div>
				)}
			</section>

			<section className="grid lookup-grid lookup-section">
				<EscrowLookup />
				<MyEscrows />
				<VaultLookup />
				<VaultListPanel />
				<ReputationLookup />
				<ReceiptLookup />
			</section>
		</main>
	);
}
