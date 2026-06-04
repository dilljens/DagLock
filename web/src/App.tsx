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
} from "./api";

import { money, sompi, time, relativeTime, badge } from "./helpers";
import { SectionTitle, Panel, LookupResult, FormField, ValidatedInput, kvad, ConfirmDialog, StatusTimeline } from "./ui";
import type { LoadState } from "./helpers";

function CreateOfferForm({ onDone }: { onDone: () => void }) {
	const [side, setSide] = useState("sell");
	const [baseAsset, setBaseAsset] = useState("KAS");
	const [quoteAsset, setQuoteAsset] = useState("USDC");
	const [amount, setAmount] = useState("");
	const [address, setAddress] = useState("");
	const [counterparty, setCounterparty] = useState("");
	const [expireHours, setExpireHours] = useState("72");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">(
		"idle",
	);
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) {
			setError("Amount must be positive");
			return;
		}
		const trimmedAddr = address.trim();
		if (!trimmedAddr.startsWith("kaspa:")) {
			setError("Address must start with kaspa: (check for leading/trailing spaces)");
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
				expires_at: Math.floor(Date.now() / 1000) + (parseInt(expireHours) || 72) * 3600,
			};
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
				<input
					value={baseAsset}
					onChange={(e) => setBaseAsset(e.target.value)}
					placeholder="KAS"
				/>
			</FormField>
			<FormField label="For asset">
				<input
					value={quoteAsset}
					onChange={(e) => setQuoteAsset(e.target.value)}
					placeholder="USDC"
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
			<ValidatedInput
				label="Your address"
				value={address}
				onChange={setAddress}
				placeholder="kaspa:..."
				validate={kvad}
			/>
			<FormField label="Expires in">
				<select value={expireHours} onChange={e => setExpireHours(e.target.value)}>
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
	const [disputeMode, setDisputeMode] = useState('standard');
	const [mediatorKey, setMediatorKey] = useState("");
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
			setError("Buyer address must start with kaspa: (check for spaces)");
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
						<strong>${result.price_at_creation.toFixed(4)} {result.price_currency || "USD"}</strong>
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
				<select value={disputeMode} onChange={e => setDisputeMode(e.target.value)}>
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

/* ─── Escrow Action (settle / refund / dispute / cancel) ─── */
type EscrowAction = "settle" | "refund" | "dispute" | "cancel";

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
			setError("Address and signature are required.");
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
			await api.disputeEscrow(escrowId, reason, mode === "jury" ? "jury" : undefined);
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
			<FormField label="Signature (hex)">
				<input
					value={authSig}
					onChange={(e) => setAuthSig(e.target.value)}
					placeholder="hex signature from wallet"
				/>
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
					setError("Address and signature are required for this action.");
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
					setError("Address and signature are required.");
					return;
				}
				auth = { address: authAddress, signature: authSignature, message: action + ":" + escrowId };
			}
			let res: { status: string; escrow_id: string };
			const act: string = action;
			if (act === "settle") res = await api.settleEscrow(escrowId, auth!);
			else if (act === "refund") res = await api.refundEscrow(escrowId, auth!);
			else if (act === "dispute") res = await api.disputeEscrow(escrowId, disputeReason);
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
					<FormField label="Signature (hex)">
						<input
							value={authSignature}
							onChange={(e) => setAuthSignature(e.target.value)}
							placeholder="hex signature from wallet"
						/>
					</FormField>
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
			setError("Address must start with kaspa:");
			return;
		}
		if (!trimmedTgHandle.startsWith("@")) {
			setError("Telegram handle must start with @");
			return;
		}
		if (!signature.trim()) {
			setError("Signature is required");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const message = `daglock.io:verify:telegram:${trimmedTgHandle}`;
			const auth: AuthHeaders = { address: trimmedTgAddr, signature: signature.trim(), message };
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
			<FormField label="Signature (hex)">
				<input
					value={signature}
					onChange={(e) => setSignature(e.target.value)}
					placeholder="hex signature from wallet"
				/>
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
			<small className="muted addr">by {offer.creator_address.slice(0, 24)}…</small>
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
	const [messages, setMessages] = useState<LoadState<EscrowMessage[]>>({ loading: false });
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
				const cauth: AuthHeaders = { address: chatAddr, signature: chatSig, message: "messages" };
				api.listMessages(id.trim(), cauth)
					.then(r => setMessages({ data: r.messages, loading: false }))
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
					<input value={chatAddr} onChange={e => setChatAddr(e.target.value)} placeholder="kaspa:..." />
				</FormField>
				<FormField label="Signature">
					<input value={chatSig} onChange={e => setChatSig(e.target.value)} placeholder="hex" />
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
								<strong>${data.price_at_creation.toFixed(4)} {data.price_currency || "USD"}</strong>
							</div>
						)}
						{data.dispute_mode && (
							<div className="row">
								<span>Dispute mode</span>
								<strong><span className={badge(data.dispute_mode)}>{data.dispute_mode}</span></strong>
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
											<span className="addr">{m.sender_address.slice(0, 20)}…</span>
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
						<input value={msgText} onChange={e => setMsgText(e.target.value)} placeholder="Type a message..." />
						<button className="button primary" onClick={async () => {
							if (!msgText || !id) return;
							if (!chatAddr || !chatSig) { setMsgStatus("Auth required"); return; }
							try {
								const cauth: AuthHeaders = { address: chatAddr, signature: chatSig, message: "messages" };
								await api.sendMessage(id, msgText, cauth);
								setMsgText("");
								setMsgStatus("Sent");
								const r = await api.listMessages(id, cauth);
								setMessages({ data: r.messages, loading: false });
							} catch (err) { setMsgStatus((err as Error).message); }
						}}>Send</button>
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
				<input value={chatAddr} onChange={e => setChatAddr(e.target.value)} placeholder="kaspa:..." />
			</FormField>
			<FormField label="Signature">
				<input value={chatSig} onChange={e => setChatSig(e.target.value)} placeholder="hex" />
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
								<strong className="error-text">Warning: {(data.trading_concentration * 100).toFixed(0)}% volume with one counterparty</strong>
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
				<input value={address} onChange={e => setAddress(e.target.value)} placeholder="your kaspa address" />
				<button className="button" type="submit">List</button>
			</form>
			<LookupResult loading={list.loading} error={list.error} data={list.data} render={(data) => (
				<div className="stack">
					{data.length === 0 && <p className="muted">No escrows found for this address.</p>}
					{data.map(e => (
						<article key={e.id} className="offer" style={{ cursor: "default" }}>
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
			)} />
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

/* ─── My Offers Panel ─── */
function MyOffersPanel() {
	const [address, setAddress] = useState("");
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

	return (
		<div className="stack">
			<form className="form" onSubmit={handleSubmit}>
				<input value={address} onChange={e => setAddress(e.target.value)} placeholder="your kaspa address" />
				<button className="button" type="submit">List my offers</button>
			</form>
			<LookupResult loading={list.loading} error={list.error} data={list.data} render={(data) => (
				<div>
					{data.length === 0 && <p className="muted">No offers found for this address.</p>}
					{data.map(o => (
						<article key={o.id} className="offer" style={{ cursor: "default" }}>
							<div className="offer-top">
								<strong>{o.side.toUpperCase()} {money(o.amount_sompi)}</strong>
								<span className={badge(o.status)}>{o.status}</span>
							</div>
							<p>{o.base_asset} for {o.quote_asset}</p>
							<code>{o.id}</code>
							<small className="muted">{relativeTime(o.created_at)}</small>
						</article>
					))}
				</div>
			)} />
		</div>
	);
}

/* ─── Confirmation Dialog ─── */

/* ─── Status Timeline ─── */

/* ─── Create Vault Form ─── */
function CreateVaultForm({ onDone }: { onDone: () => void }) {
	const [ownerKey, setOwnerKey] = useState("");
	const [timeoutDays, setTimeoutDays] = useState("30");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<{ script: string; template_hash: string } | null>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const trimmedOwner = ownerKey.trim();
		if (!trimmedOwner || trimmedOwner.length < 64) {
			setError("Enter a valid 64-char hex public key");
			return;
		}
		const timeoutSec = Math.floor(Date.now() / 1000) + (parseInt(timeoutDays) || 30) * 86400;
		setStatus("loading");
		setError("");
		try {
			const r = await api.compile("daglock_vault", {
				owner_key: trimmedOwner,
				timeout: String(timeoutSec),
			});
			setResult(r);
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
				<p className="muted success-text">Vault covenant compiled!</p>
				<div className="row"><span>Template hash</span><code>{result.template_hash}</code></div>
				<div className="row"><span>Script</span><code style={{ fontSize: "0.7rem" }}>{result.script.slice(0, 80)}…</code></div>
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">Create a time-locked KAS vault. Only the owner can withdraw after the timeout.</p>
			<FormField label="Owner public key (hex)">
				<input value={ownerKey} onChange={e => setOwnerKey(e.target.value)} placeholder="64 hex chars" />
			</FormField>
			<FormField label="Lock duration">
				<select value={timeoutDays} onChange={e => setTimeoutDays(e.target.value)}>
					<option value="1">1 day</option>
					<option value="7">7 days</option>
					<option value="30">30 days</option>
					<option value="90">90 days</option>
					<option value="365">1 year</option>
				</select>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Compiling…" : "Compile vault"}
			</button>
		</form>
	);
}

/* ─── Compile Covenant Form ─── */
function CompileCovenantForm({ onDone }: { onDone: () => void }) {
	const [template, setTemplate] = useState("daglock");
	const [paramsStr, setParamsStr] = useState("{}");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<any>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		let params: Record<string, string>;
		try { params = JSON.parse(paramsStr); }
		catch { setError("Params must be valid JSON"); return; }
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
				<select value={template} onChange={e => setTemplate(e.target.value)}>
					<option value="daglock">DagLock (KAS escrow)</option>
					<option value="daglock_arbiter">DagLock Arbiter (with mediator)</option>
					<option value="daglock_vault">DagLock Vault (time-locked)</option>
				</select>
			</FormField>
			<FormField label="Params (JSON)">
				<textarea value={paramsStr} onChange={e => setParamsStr(e.target.value)} className="evidence-input" placeholder='{"buyer_key":"...","seller_key":"...","timeout":"1700000000","treasury_key":"..."}' />
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>{status === "loading" ? "Compiling…" : "Compile"}</button>
			{result && <pre className="muted" style={{ fontSize: "0.7rem", marginTop: 8 }}>{JSON.stringify(result, null, 2)}</pre>}
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
		"compile": {
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
			<div style={{background:'#ff9800',color:'#000',textAlign:'center',padding:'8px',fontWeight:'bold',fontSize:'14px'}}>
				⚠️ TESTNET — This is a testnet deployment. Do not use real funds.
			</div>
			<header className="hero">
				<div>
					<div className="brand">Kaspa Escrow</div>
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
					{(
						[
							["Create vault", "create-vault"],
							["Compile", "compile"],
							["Create offer", "create-offer"],
							["Create escrow", "create-escrow"],
							["Settle", "settle"],
							["Refund", "refund"],
							["Dispute", "dispute"],
							["Cancel", "cancel"],
							["My offers", "my-offers"],
							["Link Telegram", "link-telegram"],
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
				<ReputationLookup />
				<ReceiptLookup />
			</section>
		</main>
	);
}
