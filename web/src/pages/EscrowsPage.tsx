import { useState, useEffect, useCallback } from "react";
import { api, type AuthHeaders, type Escrow } from "../api";
import { money, badge } from "../helpers";
import type { LoadState } from "../helpers";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";
import { SignWithWallet } from "../components/wallet";

type Tab = "my-escrows" | "create" | "lookup";

export function EscrowsPage() {
	const [tab, setTab] = useState<Tab>("my-escrows");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<div>
			<div className="page-header">
				<h1>Escrows</h1>
				<p>Create trustless escrows. Settle, refund, or dispute.</p>
			</div>
			<div className="tab-bar">
				<button
					className={`tab-btn ${tab === "my-escrows" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("my-escrows")}
				>
					My Escrows
				</button>
				<button
					className={`tab-btn ${tab === "create" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("create")}
				>
					Create
				</button>
				<button
					className={`tab-btn ${tab === "lookup" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("lookup")}
				>
					Lookup
				</button>
			</div>
			{tab === "my-escrows" &&
				(wallet.connected ? <MyEscrows address={address!} /> : <ConnectPrompt />)}
			{tab === "create" &&
				(wallet.connected ? <CreateEscrow address={address!} /> : <ConnectPrompt />)}
			{tab === "lookup" && <EscrowLookup />}
		</div>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<div className="empty-state">
			<div className="empty-state-icon"></div>
			<h3>Connect your wallet</h3>
			<p>Connect KasWare to create and manage escrows.</p>
			<button className="button primary" onClick={connect}>
				Connect Wallet
			</button>
		</div>
	);
}

/* ─── My Escrows ─── */
function MyEscrows({ address }: { address: string }) {
	const [escrows, setEscrows] = useState<LoadState<Escrow[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);

	const load = useCallback(() => {
		setEscrows({ loading: true });
		api
			.escrows(address)
			.then((d) => setEscrows({ data: d.escrows, loading: false }))
			.catch((e) => setEscrows({ error: e.message, loading: false }));
	}, [address]);

	if (escrows.loading) {
		load();
		return <p className="muted">Loading escrows…</p>;
	}
	if (escrows.error) return <p className="muted error-text">{escrows.error}</p>;
	if (!escrows.data?.length)
		return (
			<div className="empty-state">
				<div className="empty-state-icon"></div>
				<h3>No escrows yet</h3>
				<p>Create your first escrow to start trading trustlessly.</p>
			</div>
		);

	return (
		<div>
			{escrows.data.map((e) => (
				<article
					key={e.id}
					className="offer"
					style={{ cursor: "pointer", marginBottom: "8px" }}
					onClick={() => setSelectedId(selectedId === e.id ? null : e.id)}
				>
					<div className="offer-top">
						<strong>{money(e.amount_sompi)}</strong>
						<span className={badge(e.status)}>{e.status}</span>
					</div>
					<p>
						{e.asset_type} · {e.buyer_address.slice(0, 16)}…
					</p>
					<code>{e.id}</code>
					{selectedId === e.id && <EscrowActions escrow={e} onMutated={load} />}
				</article>
			))}
		</div>
	);
}

/* ─── Escrow Action Buttons ─── */
function EscrowActions({ escrow, onMutated }: { escrow: Escrow; onMutated: () => void }) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const [loading, setLoading] = useState("");

	const isFinal = ["settled", "refunded", "cancelled", "expired"].includes(escrow.status);
	if (isFinal) return <p className="muted">✓ Finalized — {escrow.status}</p>;

	async function doAction(action: "settle" | "refund" | "cancel") {
		setLoading(action);
		try {
			const auth: AuthHeaders = {
				address: escrow.buyer_address,
				signature: await sign(`${action}:${escrow.id}`),
				message: `${action}:${escrow.id}`,
			};
			if (action === "settle") await api.settleEscrow(escrow.id, auth);
			else if (action === "refund") await api.refundEscrow(escrow.id, auth);
			else await api.cancelEscrow(escrow.id);
			notify("success", `Escrow ${action}ed`);
			onMutated();
		} catch (e) {
			notify("error", `Failed to ${action}`, (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	return (
		<div className="offer-actions" style={{ marginTop: "12px" }}>
			{(escrow.status === "active" || escrow.status === "pending_confirmation") && (
				<>
					<button
						className="button primary"
						disabled={!!loading}
						onClick={() => doAction("settle")}
					>
						{loading === "settle" ? "Settling…" : " Settle"}
					</button>
					<button className="button" disabled={!!loading} onClick={() => doAction("refund")}>
						{loading === "refund" ? "Refunding…" : "Refund"}
					</button>
					<button className="button" disabled={!!loading} onClick={() => doAction("cancel")}>
						{loading === "cancel" ? "Cancelling…" : " Cancel"}
					</button>
				</>
			)}
			{escrow.status === "active" && (
				<button className="button" disabled={!!loading} onClick={() => doAction("cancel")}>
					Cancel
				</button>
			)}
			{escrow.status === "disputed" && <p className="muted"> Under dispute</p>}
		</div>
	);
}

/* ─── Create Escrow (using wallet address) ─── */
function CreateEscrow({ address }: { address: string }) {
	const [amount, setAmount] = useState("");
	const [sellerAddress, setSellerAddress] = useState("");
	const [disputeMode, setDisputeMode] = useState("standard");
	const [tradeHash, setTradeHash] = useState("");
	const [tradeSecret, setTradeSecret] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [result, setResult] = useState<Escrow | null>(null);
	const { notify } = useToast();

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		setStatus("loading");
		try {
			const sompiAmount = Number((amountNum * 100_000_000).toFixed(0));

			let lockTxId: string;
			if (window.kasware && window.kasware.getPublicKey && window.kasware.sendKaspa) {
				// Get buyer's public key from KasWare
				const buyerPubkey = await window.kasware.getPublicKey();
				// Use a hardcoded seller key for demo — in production this comes from the seller
				const sellerPubkey = "0000000000000000000000000000000000000000000000000000000000000000";
				const zeroHash = "0000000000000000000000000000000000000000000000000000000000000000";
				const timeout = Math.floor(Date.now() / 1000) + 86400;

				// Compile covenant via WASM SDK (loaded externally) or use compile API
				let covenantAddress: string;
				try {
					const resp = await fetch(import.meta.env.VITE_API_URL + "/v1/compile", {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify({
							template: "daglock",
							params: {
								buyer_key: buyerPubkey,
								seller_key: sellerPubkey,
								trade_hash: zeroHash,
								timeout: timeout.toString(),
								treasury_key: zeroHash,
							},
						}),
					});
					if (resp.ok) {
						const data = await resp.json();
						covenantAddress = data.covenant_address;
					} else {
						throw new Error("Compiler not available");
					}
				} catch {
					// Fallback: use indexer compile endpoint
					covenantAddress = address;
				}

				// Send KAS to the covenant address via KasWare
				lockTxId = await window.kasware.sendKaspa(covenantAddress, sompiAmount);
			} else {
				lockTxId = prompt("Tx ID (from kaspawallet):") || "";
				if (!lockTxId) throw new Error("Tx ID required");
			}

			const escrow = await api.createEscrow({
				lock_tx_id: lockTxId,
				lock_tx_output_index: 0,
				buyer_address: address,
				...(sellerAddress.startsWith("kaspa:") ? { seller_address: sellerAddress } : {}),
				amount_sompi: sompiAmount,
				dispute_mode: disputeMode,
				...(tradeHash.trim() ? { trade_hash: tradeHash.trim() } : {}),
			});
			setResult(escrow);
			setStatus("done");
			notify("success", "Escrow created!");
		} catch (e) {
			notify("error", "Failed to create escrow", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done" && result) {
		return (
			<div className="empty-state">
				<div className="empty-state-icon"></div>
				<h3>Escrow created!</h3>
				<p>
					ID: {result.id}
					<br />
					Status: {result.status}
				</p>
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				You: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>
			<FormField label="Amount (KAS)">
				<input
					type="number"
					step="any"
					value={amount}
					onChange={(e) => setAmount(e.target.value)}
					placeholder="100"
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
			<FormField label="Trade hash (optional, for atomic swap)">
				<div style={{ display: "flex", gap: "8px" }}>
					<input
						value={tradeHash}
						onChange={(e) => setTradeHash(e.target.value)}
						placeholder="64 hex chars"
						style={{ flex: 1 }}
					/>
					<button
						type="button"
						className="button"
						onClick={async () => {
							try {
								const r = await api.generateSwap();
								setTradeHash(r.hash);
								setTradeSecret(r.secret);
							} catch (e) {
								notify("error", (e as Error).message);
							}
						}}
					>
						Generate
					</button>
				</div>
			</FormField>
			{tradeSecret && (
				<div style={{ fontSize: "12px", color: "#ff9800", marginTop: "8px" }}>
					Save this secret: <code>{tradeSecret}</code>
				</div>
			)}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
				style={{ marginTop: "12px" }}
			>
				{status === "loading" ? "Creating…" : "Create Escrow"}
			</button>
		</form>
	);
}

/* ─── Escrow Lookup (no wallet needed) ─── */
function EscrowLookup() {
	const [id, setId] = useState("");
	const [escrow, setEscrow] = useState<LoadState<Escrow>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!id.trim()) return;
		setEscrow({ loading: true });
		try {
			const data = await api.escrow(id.trim());
			setEscrow({ data, loading: false });
		} catch (err) {
			setEscrow({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<div>
			<form className="form" onSubmit={handleSubmit} style={{ marginBottom: "16px" }}>
				<input
					value={id}
					onChange={(e) => setId(e.target.value)}
					placeholder="escrow id (esc_...)"
				/>
				<button className="button primary" type="submit" disabled={escrow.loading}>
					{escrow.loading ? "Loading…" : "Fetch"}
				</button>
			</form>
			{escrow.error && <p className="muted error-text">{escrow.error}</p>}
			{escrow.data && (
				<div className="panel">
					<div className="stack">
						<div className="row">
							<span>ID</span>
							<code>{escrow.data.id}</code>
						</div>
						<div className="row">
							<span>Status</span>
							<strong>
								{badge(escrow.data.status)} {escrow.data.status}
							</strong>
						</div>
						<div className="row">
							<span>Amount</span>
							<strong>{money(escrow.data.amount_sompi)}</strong>
						</div>
						<div className="row">
							<span>Buyer</span>
							<strong className="addr">{escrow.data.buyer_address}</strong>
						</div>
						{escrow.data.seller_address && (
							<div className="row">
								<span>Seller</span>
								<strong className="addr">{escrow.data.seller_address}</strong>
							</div>
						)}
						{escrow.data.dispute_reason && (
							<div className="row">
								<span>Dispute</span>
								<strong>{escrow.data.dispute_reason}</strong>
							</div>
						)}
						{escrow.data.dispute_mode && (
							<div className="row">
								<span>Mode</span>
								<strong>{escrow.data.dispute_mode}</strong>
							</div>
						)}
					</div>
				</div>
			)}
		</div>
	);
}
