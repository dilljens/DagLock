import { useState, useEffect, useCallback, useRef } from "react";
import {
	api,
	type AuthHeaders,
	type Escrow,
	type MilestoneEscrow,
	type MultiEscrow,
	type Deposit,
} from "../api";
import { useRouter } from "../router";
import { money, badge, time, moneyCompact } from "../helpers";
import type { LoadState } from "../helpers";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonTable } from "../ui";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";
import { ReceiptLookup } from "../components/lookup";
import { CreateInvoiceForm } from "../components/invoice-form";
import { ExplorerTxLink, ExplorerAddressLink } from "../components/ExplorerLink";
import { FeeCalculator } from "../components/FeeCalculator";
import { ChatPanel } from "../components/ChatPanel";
import { MediationPanel } from "../components/MediationPanel";
import { generateChatKeypair, type ChatKeypair } from "../crypto/chat-crypto";
import { encodeBase64 } from "tweetnacl-util";
import { saveKeypair } from "../crypto/chat-store";
import { downloadRecoverySheet } from "../crypto/recovery-sheet";
import { useQuery } from "@tanstack/react-query";

type Tab =
	| "my-escrows"
	| "create"
	| "lookup"
	| "receipt"
	| "invoice";

/* Escrow sub-type for filtering/display */
type EscrowSubtype = "standard" | "swap" | "milestone" | "multi" | "all";

const DEAL_PRESETS = {
	goods: {
		label: "🛒 Goods",
		description: "Physical items — 72h dispute window",
		disputeWindow: 72,
		autoSettle: 72 * 3600,
	},
	otc: {
		label: "🤝 OTC Trade",
		description: "KAS/KRC-20 trades — 24h dispute window",
		disputeWindow: 24,
		autoSettle: 24 * 3600,
	},
	service: {
		label: "🛠️ Service",
		description: "Freelance work — 120h dispute window",
		disputeWindow: 120,
		autoSettle: 120 * 3600,
	},
	custom: {
		label: "⚙️ Custom",
		description: "Set your own terms",
		disputeWindow: null,
		autoSettle: null,
	},
};

function dealTypeFromTimeout(escrow: Escrow): { key: string; label: string } | null {
	if (!escrow.auto_settle_timeout || !escrow.created_at) return null;
	const duration = escrow.auto_settle_timeout - escrow.created_at;
	if (duration === 259200) return { key: "goods", label: "🛒 Goods (72h)" };
	if (duration === 86400) return { key: "otc", label: "🤝 OTC (24h)" };
	if (duration === 432000) return { key: "service", label: "🛠️ Service (120h)" };
	return { key: "custom", label: "⚙️ Custom" };
}

function DealTypeBadge({ escrow }: { escrow: Escrow }) {
	const info = dealTypeFromTimeout(escrow);
	if (!info) return null;
	const colors: Record<string, { bg: string; fg: string }> = {
		goods: { bg: "#4caf5022", fg: "#4caf50" },
		otc: { bg: "#2196f322", fg: "#2196f3" },
		service: { bg: "#ff980022", fg: "#ff9800" },
		custom: { bg: "#88888822", fg: "#888" },
	};
	const c = colors[info.key] || colors.custom;
	return (
		<span
			className="pill"
			style={{
				background: c.bg,
				color: c.fg,
				border: `1px solid ${c.fg}44`,
				fontSize: "11px",
			}}
		>
			{info.label}
		</span>
	);
}

export function EscrowsPage() {
	const [tab, setTab] = useState<Tab>("my-escrows");
	const [subtypeFilter, setSubtypeFilter] = useState<EscrowSubtype>("all");
	const address = useAddress();
	const { state: wallet } = useWallet();

	// Read ?type= and ?asset= query params
	const [presetAsset, setPresetAsset] = useState<string | null>(null);
	useEffect(() => {
		const params = new URLSearchParams(window.location.search);
		const type = params.get("type");
		const asset = params.get("asset");
		if (type === "milestone" || type === "multi" || type === "create") {
			setTab("create");
		}
		if (asset) {
			setPresetAsset(asset);
		}
	}, []);

	return (
		<>
			<Helmet>
				<title>Escrows — DagLock</title>
				<meta
					name="description"
					content="Create and manage trustless escrow contracts on Kaspa. Lock KAS or KRC-20 tokens with covenant-enforced terms."
				/>
				<link rel="canonical" href="https://daglock.com/escrows" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Escrows</h1>
					<p>Create trustless escrows. Settle, refund, or dispute.</p>
				</div>
				<div className="tab-bar">
					<button
						className={`tab-btn ${tab === "my-escrows" ? "tab-btn--active" : ""}`}
						onClick={() => { setTab("my-escrows"); setSubtypeFilter("all"); }}
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
					<button
						className={`tab-btn ${tab === "receipt" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("receipt")}
					>
						Receipt
					</button>
					{wallet.connected && (
						<button
							className={`tab-btn ${tab === "invoice" ? "tab-btn--active" : ""}`}
							onClick={() => setTab("invoice")}
						>
							Invoice
						</button>
					)}
				</div>
				{tab === "my-escrows" &&
					(wallet.connected ? (
						<AllEscrows address={address!} subtypeFilter={subtypeFilter} onSetFilter={setSubtypeFilter} />
					) : (
						<ConnectPrompt />
					))}
				{tab === "create" &&
					(wallet.connected ? <CreateFlow address={address!} presetAsset={presetAsset} /> : <ConnectPrompt />)}
				{tab === "lookup" && <EscrowLookup />}
				{tab === "receipt" && <ReceiptLookup />}
				{tab === "invoice" && <CreateInvoiceForm />}
			</div>
		</>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<EmptyState
			icon="👛"
			title="Connect your wallet"
			description="Connect KasWare to create and manage escrows."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

/* ─── Current KAS/USD price (for USD display) ─── */
function useKasUsdPrice() {
	const { data } = useQuery({
		queryKey: ["kas-usd-price"],
		queryFn: () => api.networkPrice(),
		staleTime: 60_000,
		refetchInterval: 120_000,
	});
	return data?.kas_usd ?? null;
}

function formatUsd(sompi: number, priceUsd: number | null): string {
	if (!priceUsd || !sompi) return "";
	const kas = sompi / 100_000_000;
	return `~$${(kas * priceUsd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

/* ─── My Escrows ─── */
/* ─── Unified All Escrows — standard, swap, milestone, multi-party ─── */
function AllEscrows({
	address,
	subtypeFilter,
	onSetFilter,
}: {
	address: string;
	subtypeFilter: EscrowSubtype;
	onSetFilter: (f: EscrowSubtype) => void;
}) {
	const { navigate } = useRouter();
	const [escrows, setEscrows] = useState<LoadState<Escrow[]>>({ loading: true });
	const [milestones, setMilestones] = useState<LoadState<MilestoneEscrow[]>>({ loading: true });
	const [multiEscrows, setMultiEscrows] = useState<LoadState<MultiEscrow[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const [dealTypeFilter, setDealTypeFilter] = useState("all");
	const loaded = useRef(false);
	const usdPrice = useKasUsdPrice();

	/* ── determine sub-type of a regular escrow ── */
	function escrowSubtype(e: Escrow): "standard" | "swap" {
		return e.trade_hash && e.trade_hash.length > 0 ? "swap" : "standard";
	}

	/* ── fetch everything ── */
	const fetchAll = useCallback(() => {
		setEscrows({ loading: true });
		setMilestones({ loading: true });
		setMultiEscrows({ loading: true });
		Promise.all([
			api.escrows(address).then((d) => setEscrows({ data: d.escrows, loading: false })),
			api
				.milestones(address)
				.then((d) => setMilestones({ data: d.milestones, loading: false }))
				.catch(() => setMilestones({ data: [], loading: false })),
			api
				.multiEscrows(address)
				.then((d) => setMultiEscrows({ data: d.multi_escrows, loading: false }))
				.catch(() => setMultiEscrows({ data: [], loading: false })),
		]).catch((e) => {
			setEscrows((s) => (s.loading ? { error: e.message, loading: false } : s));
		});
	}, [address]);

	useEffect(() => {
		if (loaded.current) return;
		loaded.current = true;
		fetchAll();
	}, [fetchAll]);

	/* ── subtype filter chips ── */
	const subtypeChips: { key: EscrowSubtype; label: string }[] = [
		{ key: "all", label: "All" },
		{ key: "standard", label: "Standard" },
		{ key: "swap", label: "Atomic Swaps" },
		{ key: "milestone", label: "Milestones" },
		{ key: "multi", label: "Multi-Party" },
	];

	const escrowCount = (escrows.data || []).length;
	const milestoneCount = (milestones.data || []).length;
	const multiCount = (multiEscrows.data || []).length;
	const totalCount = escrowCount + milestoneCount + multiCount;
	const isLoading = escrows.loading || milestones.loading || multiEscrows.loading;

	if (isLoading) {
		return <SkeletonTable rows={5} />;
	}

	if (totalCount === 0) {
		return (
			<div>
				<EmptyState
					icon="🤝"
					title="Nothing here yet"
					description="Create your first escrow, milestone, or multi-party agreement."
					action={{ label: "Create Escrow", onClick: () => navigate("/escrows") }}
				/>
				<div
					style={{
						display: "flex",
						justifyContent: "center",
						gap: "12px",
						marginTop: "12px",
					}}
				>
					<button className="button" onClick={() => navigate("/swap")}>
						🔄 Atomic Swap
					</button>
					<button className="button" onClick={() => navigate("/vaults")}>
						🏗️ Milestone
					</button>
				</div>
			</div>
		);
	}

	/* ── render a type badge for the subtype ── */
	function SubtypeBadge({ type }: { type: EscrowSubtype }) {
		if (type === "all" || type === "standard") return null;
		const chip = subtypeChips.find((c) => c.key === type);
		if (!chip) return null;
		const colors: Record<string, { bg: string; fg: string }> = {
			swap: { bg: "#2196f322", fg: "#2196f3" },
			milestone: { bg: "#ff980022", fg: "#ff9800" },
			multi: { bg: "#9c27b022", fg: "#9c27b0" },
		};
		const c = colors[type] || { bg: "#88888822", fg: "#888" };
		return (
			<span
				className="pill"
				style={{
					background: c.bg,
					color: c.fg,
					border: `1px solid ${c.fg}44`,
					fontSize: "11px",
				}}
			>
				{chip.label}
			</span>
		);
	}

	return (
		<div>
			{/* Filter row */}
			<div
				style={{
					display: "flex",
					justifyContent: "space-between",
					alignItems: "center",
					marginBottom: "8px",
					flexWrap: "wrap",
					gap: "8px",
				}}
			>
				<div style={{ display: "flex", gap: "6px", alignItems: "center", flexWrap: "wrap" }}>
					<span style={{ fontSize: "12px", color: "#888" }}>Type:</span>
					{subtypeChips.map((chip) => {
						let count = 0;
						if (chip.key === "all") count = totalCount;
						else if (chip.key === "standard")
							count = escrows.data?.filter((e) => escrowSubtype(e) === "standard").length || 0;
						else if (chip.key === "swap")
							count = escrows.data?.filter((e) => escrowSubtype(e) === "swap").length || 0;
						else if (chip.key === "milestone") count = milestoneCount;
						else if (chip.key === "multi") count = multiCount;

						return (
							<button
								key={chip.key}
								className={`button ${subtypeFilter === chip.key ? "primary" : ""}`}
								onClick={() => onSetFilter(chip.key)}
								style={{ fontSize: "11px", padding: "2px 8px" }}
							>
								{chip.label} ({count})
							</button>
						);
					})}
				</div>
				<a
					href={`${import.meta.env.VITE_API_URL || ""}/v1/escrows/export?address=${encodeURIComponent(address)}`}
					download="daglock-escrows.csv"
					className="button"
					style={{ fontSize: "12px", padding: "4px 12px", textDecoration: "none" }}
				>
					⬇ Export
				</a>
			</div>

			{/* Escrow cards */}
			{(subtypeFilter === "all" || subtypeFilter === "standard" || subtypeFilter === "swap") &&
				(escrows.data || [])
					.filter((e) => {
						if (subtypeFilter === "all") return true;
						return escrowSubtype(e) === subtypeFilter;
					})
					.filter((e) => {
						if (dealTypeFilter === "all") return true;
						const dt = dealTypeFromTimeout(e);
						if (dealTypeFilter === "custom") return dt?.key === "custom" || dt === null;
						return dt?.key === dealTypeFilter;
					})
					.map((e) => {
						const subType = escrowSubtype(e);
						return (
							<article
								key={e.id}
								className="offer"
								style={{ cursor: "pointer", marginBottom: "8px" }}
								onClick={() => setSelectedId(selectedId === e.id ? null : e.id)}
							>
								<div className="offer-top">
									<strong>{money(e.amount_sompi)}</strong>
									{usdPrice && (
										<span style={{ fontSize: "12px", color: "#888" }}>
											({formatUsd(e.amount_sompi, usdPrice)})
										</span>
									)}
									<SubtypeBadge type={subType} />
									<DealTypeBadge escrow={e} />
									<span className={badge(e.status)}>{e.status}</span>
								</div>
								<p>
									{e.asset_type} · {e.buyer_address.slice(0, 16)}…
									{e.seller_address ? ` → ${e.seller_address.slice(0, 16)}…` : ""}
								</p>
								<code>{e.id}</code>
								{e.trade_hash && (
									<code style={{ fontSize: "10px", wordBreak: "break-all", display: "block" }}>
										Hash: {e.trade_hash.slice(0, 32)}…
									</code>
								)}
								{e.price_at_creation && (
									<div style={{ fontSize: "11px", color: "#888", marginTop: "2px" }}>
										~${e.price_at_creation.toFixed(2)} USD at creation
									</div>
								)}
								<div style={{ display: "flex", gap: "12px", marginTop: "4px" }}>
									<ExplorerTxLink txid={e.lock_tx_id} label="View TX" />
									<ExplorerAddressLink address={e.buyer_address} label="Buyer" />
								</div>
								{selectedId === e.id && (
									<>
										<EscrowActions escrow={e} onMutated={fetchAll} />
										{address && <ChatPanel escrow={e} onMutated={fetchAll} />}
									</>
								)}
							</article>
						);
					})}

			{/* Milestone cards */}
			{(subtypeFilter === "all" || subtypeFilter === "milestone") &&
				(milestones.data || []).map((ms) => (
					<article
						key={ms.id}
						className="offer"
						style={{ cursor: "pointer", marginBottom: "8px" }}
						onClick={() => setSelectedId(selectedId === ms.id ? null : ms.id)}
					>
						<div className="offer-top">
							<strong>{money(ms.total_amount)}</strong>
							<SubtypeBadge type="milestone" />
							<span className={badge(ms.status)}>{ms.status}</span>
						</div>
						<p>
							{ms.buyer_address.slice(0, 16)}… → {ms.seller_address.slice(0, 16)}…
						</p>
						<MilestoneProgressBar
							milestoneStatuses={ms.milestone_statuses}
							currentMilestone={ms.current_milestone}
						/>
						<code>{ms.id}</code>
						{selectedId === ms.id && (
							<MilestoneActions escrow={ms} onMutated={fetchAll} />
						)}
					</article>
				))}

			{/* Multi-party cards */}
			{(subtypeFilter === "all" || subtypeFilter === "multi") &&
				(multiEscrows.data || []).map((m) => (
					<article
						key={m.id}
						className="offer"
						style={{ cursor: "pointer", marginBottom: "8px" }}
						onClick={() => setSelectedId(selectedId === m.id ? null : m.id)}
					>
						<div className="offer-top">
							<strong>{money(m.total_amount)}</strong>
							<SubtypeBadge type="multi" />
							<span className={badge(m.status)}>{m.status}</span>
						</div>
						<p>
							{m.parties.length} parties · {m.signatures.length}/{m.parties.length} signed
						</p>
						<code>{m.id}</code>
						{selectedId === m.id && (
							<MultiEscrowActions escrow={m} onMutated={fetchAll} />
						)}
					</article>
				))}
		</div>
	);
}

/* ─── My Swaps (atomic swap filter) ─── */
function MySwaps({ address }: { address: string }) {
	const { navigate } = useRouter();
	const [escrows, setEscrows] = useState<LoadState<Escrow[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const loaded = useRef(false);

	const fetchEscrows = useCallback(() => {
		setEscrows({ loading: true });
		api
			.escrows(address)
			.then((d) => {
				const swaps = d.escrows.filter((e) => e.trade_hash && e.trade_hash.length > 0);
				setEscrows({ data: swaps, loading: false });
			})
			.catch((e) => setEscrows({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => {
		if (loaded.current) return;
		loaded.current = true;
		fetchEscrows();
	}, [fetchEscrows]);

	function swapStatus(escrow: Escrow): string {
		if (escrow.status === "settled") return "Complete (settled)";
		if (
			escrow.status === "refunded" ||
			escrow.status === "expired" ||
			escrow.status === "cancelled"
		)
			return "Refunded / Expired";
		if (escrow.status === "active" || escrow.status === "pending_confirmation")
			return "Waiting for counterparty";
		return escrow.status;
	}

	function swapStatusEmoji(status: string): string {
		if (status.startsWith("Complete")) return "🎉";
		if (status.startsWith("Refunded")) return "↩️";
		if (status.startsWith("Waiting")) return "⏳";
		return "❓";
	}

	if (escrows.loading) {
		return <SkeletonTable rows={3} />;
	}
	if (escrows.error) return <p className="muted error-text">{escrows.error}</p>;
	if (!escrows.data?.length)
		return (
			<EmptyState
				icon="🔄"
				title="No atomic swaps"
				description="Swaps are escrows with a trade hash for preimage-based settlement."
				action={{ label: "Create Swap", onClick: () => navigate("/swap") }}
			/>
		);

	return (
		<div>
			{escrows.data.map((e) => {
				const s = swapStatus(e);
				return (
					<article
						key={e.id}
						className="offer"
						style={{ cursor: "pointer", marginBottom: "8px" }}
						onClick={() => setSelectedId(selectedId === e.id ? null : e.id)}
					>
						<div className="offer-top">
							<strong>{money(e.amount_sompi)}</strong>
							<span style={{ fontSize: "12px", color: "#888" }}>
								{swapStatusEmoji(s)} {s}
							</span>
						</div>
						<p>
							{e.asset_type || "KAS"} · {e.buyer_address.slice(0, 16)}…
							{e.seller_address ? ` → ${e.seller_address.slice(0, 16)}…` : ""}
						</p>
						{e.trade_hash && (
							<code style={{ fontSize: "10px", wordBreak: "break-all" }}>
								Hash: {e.trade_hash.slice(0, 32)}…
							</code>
						)}
						<div style={{ display: "flex", gap: "12px", marginTop: "4px" }}>
							<ExplorerTxLink txid={e.lock_tx_id} label="View TX" />
							{e.status === "settled" && (
								<button
									className="button"
									style={{ fontSize: "11px", padding: "2px 8px" }}
									onClick={async (ev) => {
										ev.stopPropagation();
										try {
											await api.receipt(e.id);
										} catch {
											// fallback
										}
									}}
								>
									🧾 Receipt
								</button>
							)}
						</div>
					</article>
				);
			})}
		</div>
	);
}

/* ─── Escrow Action Buttons ─── */
function EscrowActions({ escrow, onMutated }: { escrow: Escrow; onMutated: () => void }) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const [loading, setLoading] = useState("");
	const [countdown, setCountdown] = useState("");
	const [deposit, setDeposit] = useState<Deposit | null | undefined>(undefined);

	// Fetch deposit for this escrow
	useEffect(() => {
		api
			.getDeposit(escrow.id)
			.then((d) => setDeposit(d))
			.catch(() => setDeposit(null));
	}, [escrow.id]);

	const isFinal = ["settled", "refunded", "cancelled", "expired"].includes(escrow.status);
	if (isFinal) return <p className="muted">✓ Finalized — {escrow.status}</p>;

	// Auto-settle countdown timer
	useEffect(() => {
		if (!escrow.auto_settle_timeout || escrow.status !== "active") return;
		function tick() {
			const diff = escrow.auto_settle_timeout! - Math.floor(Date.now() / 1000);
			if (diff <= 0) {
				setCountdown("ready");
				return;
			}
			const d = Math.floor(diff / 86400);
			const h = Math.floor((diff % 86400) / 3600);
			const m = Math.floor((diff % 3600) / 60);
			setCountdown(d > 0 ? `${d}d ${h}h ${m}m` : `${h}h ${m}m`);
		}
		tick();
		const id = setInterval(tick, 30_000);
		return () => clearInterval(id);
	}, [escrow.auto_settle_timeout, escrow.status]);

	async function doAction(action: "settle" | "refund" | "cancel" | "auto-settle") {
		setLoading(action);
		try {
			if (action === "auto-settle") {
				await api.autoSettleEscrow(escrow.id);
				notify("success", "Escrow auto-settled");
			} else {
				const auth: AuthHeaders = {
					address: escrow.buyer_address,
					signature: await sign(`${action}:${escrow.id}`),
					message: `${action}:${escrow.id}`,
				};
				if (action === "settle") await api.settleEscrow(escrow.id, auth);
				else if (action === "refund") await api.refundEscrow(escrow.id, auth);
				else await api.cancelEscrow(escrow.id);
				notify("success", `Escrow ${action}ed`);
			}
			onMutated();
		} catch (e) {
			notify("error", `Failed to ${action}`, (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	return (
		<div className="offer-actions" style={{ marginTop: "12px" }}>
			{/* Deposit status badge */}
			{deposit && (
				<div
					style={{
						marginBottom: "8px",
						padding: "4px 8px",
						borderRadius: "4px",
						fontSize: "12px",
						background:
							deposit.status === "locked"
								? "#fff3cd"
								: deposit.status === "released"
									? "#d4edda"
									: deposit.status === "forfeited"
										? "#f8d7da"
										: "#e2e3e5",
						color:
							deposit.status === "locked"
								? "#856404"
								: deposit.status === "released"
									? "#155724"
									: deposit.status === "forfeited"
										? "#721c24"
										: "#383d41",
					}}
				>
					🔒 {money(deposit.deposit_amount)} deposit ({deposit.status})
					{deposit.forfeited_to && ` → ${deposit.forfeited_to.slice(0, 16)}…`}
				</div>
			)}

			{/* Auto-settle countdown + button */}
			{escrow.auto_settle_timeout &&
				escrow.status === "active" &&
				countdown &&
				countdown !== "ready" && (
					<p className="muted" style={{ fontSize: "13px", marginBottom: "8px" }}>
						⏳ Auto-settles in {countdown}
					</p>
				)}
			{escrow.auto_settle_timeout && escrow.status === "active" && countdown === "ready" && (
				<button
					className="button primary"
					disabled={!!loading}
					onClick={() => doAction("auto-settle")}
					style={{ marginRight: "8px" }}
				>
					{loading === "auto-settle" ? "Settling…" : "Auto-settle now"}
				</button>
			)}
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
			{escrow.status === "disputed" && (
				<>
					<p className="muted"> Under dispute</p>
					<MediationPanel escrowId={escrow.id} disputeMode={escrow.dispute_mode} />
				</>
			)}

			{/* Forfeit deposit button (jury members only, when deposit is locked) */}
			{deposit && deposit.status === "locked" && escrow.status === "disputed" && (
				<button
					className="button"
					disabled={!!loading}
					onClick={async () => {
						const forfeitTo = prompt("Forfeit deposit to address (buyer or seller):");
						if (!forfeitTo || !forfeitTo.startsWith("kaspa:")) {
							notify("error", "Valid kaspa: address required");
							return;
						}
						setLoading("forfeit");
						try {
							const jurySig = await sign(`forfeit:${escrow.id}:${forfeitTo}`);
							await api.forfeitDeposit(escrow.id, {
								forfeited_to: forfeitTo,
								jury_signature: jurySig,
							});
							notify("success", "Deposit forfeited");
							onMutated();
						} catch (e) {
							notify("error", "Failed to forfeit deposit", (e as Error).message);
						} finally {
							setLoading("");
						}
					}}
					style={{ marginRight: "8px" }}
				>
					{loading === "forfeit" ? "Forfeiting…" : "Forfeit Deposit"}
				</button>
			)}
		</div>
	);
}

/* ─── Unified Create Flow — Standard, Milestone, Multi-party ─── */
type CreateMode = "standard" | "milestone" | "multi";

function CreateFlow({ address, presetAsset }: { address: string; presetAsset?: string | null }) {
	const [mode, setMode] = useState<CreateMode>("standard");

	return (
		<div>
			{presetAsset && presetAsset.startsWith("KRC20:") && (
				<div
					style={{
						background: "var(--accent-dim)",
						borderRadius: "8px",
						padding: "8px 12px",
						marginBottom: "12px",
						fontSize: "13px",
						color: "var(--accent)",
					}}
				>
					Trading for <strong>{presetAsset}</strong> — you'll lock KAS and receive
					the tokens when the escrow settles.
				</div>
			)}
			<div style={{ display: "flex", gap: "8px", marginBottom: "16px" }}>
				{[
					{ key: "standard" as const, label: "Standard Escrow", desc: "Simple release/refund" },
					{ key: "milestone" as const, label: "Milestone", desc: "Phased payments (up to 5)" },
					{ key: "multi" as const, label: "Multi-Party", desc: "Split among up to 4 parties" },
				].map((m) => (
					<button
						key={m.key}
						type="button"
						className={`button ${mode === m.key ? "primary" : ""}`}
						onClick={() => setMode(m.key)}
						style={{ flex: 1, textAlign: "center", fontSize: "12px", padding: "10px 8px" }}
					>
						<div>{m.label}</div>
						<div style={{ fontSize: "10px", fontWeight: 400, opacity: 0.7 }}>{m.desc}</div>
					</button>
				))}
			</div>
			{mode === "standard" && <CreateEscrow address={address} presetAsset={presetAsset} />}
			{mode === "milestone" && <CreateMilestoneForm address={address} />}
			{mode === "multi" && <CreateMultiForm address={address} />}
		</div>
	);
}

/* ─── Create Escrow (using wallet address) ─── */
function CreateEscrow({ address, presetAsset }: { address: string; presetAsset?: string | null }) {
	const [amount, setAmount] = useState("");
	const [sellerAddress, setSellerAddress] = useState("");
	const [disputeMode, setDisputeMode] = useState("standard");
	const [tradeHash, setTradeHash] = useState("");
	const [tradeSecret, setTradeSecret] = useState("");
	const [memo, setMemo] = useState("");
	const [autoSettle, setAutoSettle] = useState(false);
	const [autoSettleDuration, setAutoSettleDuration] = useState(86400);
	const [selectedPreset, setSelectedPreset] = useState("custom");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [result, setResult] = useState<Escrow | null>(null);
	const [chatKeypair, setChatKeypair] = useState<ChatKeypair | null>(null);
	const { notify } = useToast();

	useEffect(() => {
		setChatKeypair(generateChatKeypair());
	}, []);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		setStatus("loading");
		try {
			const sompiAmount = Number((amountNum * 100_000_000).toFixed(0));

			let lockTxId: string;
			if (window.kasware?.getPublicKey && window.kasware.sendKaspa) {
				// KasWare is detected — get buyer's public key
				const buyerPubkey = await window.kasware.getPublicKey();
				// Require the seller to provide their public key (prevents broken covenants)
				const sellerPubkey = prompt("Enter seller's public key (64 hex chars):");
				if (
					!sellerPubkey ||
					sellerPubkey.length !== 64 ||
					!/^[0-9a-fA-F]{64}$/.test(sellerPubkey)
				) {
					setStatus("idle");
					notify(
						"error",
						"Valid seller public key (64 hex) is required to create an escrow covenant.",
					);
					return;
				}
				const treasuryKey = prompt(
					"Enter treasury public key (64 hex chars, or leave blank for default):",
				)?.trim();
				const resolvedTreasuryKey =
					treasuryKey || "0000000000000000000000000000000000000000000000000000000000000000";
				const timeout = Math.floor(Date.now() / 1000) + 86400;

				// Compile covenant via API
				let covenantAddress: string;
				try {
					const resp = await fetch(`${import.meta.env.VITE_API_URL}/v1/compile`, {
						method: "POST",
						headers: { "Content-Type": "application/json" },
						body: JSON.stringify({
							template: "daglock",
							params: {
								buyer_key: buyerPubkey,
								seller_key: sellerPubkey,
								trade_hash: "0000000000000000000000000000000000000000000000000000000000000000",
								timeout: timeout.toString(),
								treasury_key: resolvedTreasuryKey,
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
					notify("error", "Covenant compilation failed. Use manual mode instead.");
					setStatus("idle");
					return;
				}

				// Send KAS to the covenant address via KasWare
				lockTxId = await window.kasware.sendKaspa(covenantAddress, sompiAmount);
			} else {
				// Manual mode or no KasWare: prompt for tx ID
				lockTxId = prompt("Enter tx_id from wallet (or any hex for testnet dev mode):") || "";
				if (!lockTxId) throw new Error("Tx ID required");
			}

			const escrow = await api.createEscrow({
				lock_tx_id: lockTxId,
				lock_tx_output_index: 0,
				buyer_address: address,
				...(sellerAddress.startsWith("kaspa:") ? { seller_address: sellerAddress } : {}),
				amount_sompi: sompiAmount,
				asset_type: presetAsset || "KAS",
				dispute_mode: disputeMode,
				...(tradeHash.trim() ? { trade_hash: tradeHash.trim() } : {}),
				...(memo.trim() ? { memo: memo.trim() } : {}),
				...(autoSettle
					? { auto_settle_timeout: Math.floor(Date.now() / 1000) + autoSettleDuration }
					: {}),
				...(chatKeypair ? { chat_pubkey: encodeBase64(chatKeypair.pubkey) } : {}),
			});
			if (chatKeypair) saveKeypair(escrow.id, chatKeypair);
			setResult(escrow);
			setStatus("done");
		} catch (e) {
			notify("error", "Failed to create escrow", (e as Error).message);
			setStatus("idle");
		}
	}

	function handlePresetSelect(key: string) {
		setSelectedPreset(key);
		if (key === "custom") {
			setAutoSettle(false);
			return;
		}
		const preset = DEAL_PRESETS[key as keyof typeof DEAL_PRESETS];
		if (preset.autoSettle != null) {
			setAutoSettle(true);
			setAutoSettleDuration(preset.autoSettle);
		}
	}

	if (status === "done" && result) {
		return (
			<div>
				<EmptyState
					icon="✅"
					title="Escrow created!"
					description={`ID: ${result.id} | Status: ${result.status}`}
				/>
				{chatKeypair && (
					<div
						style={{
							marginTop: "16px",
							padding: "16px",
							background: "#1a2a1a",
							borderRadius: "8px",
							textAlign: "center",
						}}
					>
						<p style={{ marginBottom: "8px", fontSize: "14px", color: "#4caf50" }}>
							📥 Download your chat recovery sheet
						</p>
						<p style={{ fontSize: "12px", color: "#aaa", marginBottom: "12px" }}>
							Keep this file safe. You'll need it to restore encrypted chat on another device.
						</p>
						<button
							className="button primary"
							onClick={() =>
								downloadRecoverySheet({
									escrowId: result.id,
									chatPubkey: encodeBase64(chatKeypair.pubkey),
									chatSecret: encodeBase64(chatKeypair.secret),
									createdAt: new Date().toISOString(),
								})
							}
						>
							Download Recovery Sheet
						</button>
					</div>
				)}
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				You: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>
			<FormField label="Deal type">
				<div style={{ display: "flex", gap: "6px", flexWrap: "wrap" }}>
					{Object.entries(DEAL_PRESETS).map(([key, preset]) => (
						<button
							key={key}
							type="button"
							className={`button ${selectedPreset === key ? "primary" : ""}`}
							onClick={() => handlePresetSelect(key)}
							style={{
								fontSize: "12px",
								padding: "6px 10px",
								textAlign: "center",
								lineHeight: 1.3,
							}}
						>
							<div>{preset.label}</div>
							<div style={{ fontSize: "10px", fontWeight: 400, opacity: 0.7 }}>
								{preset.description}
							</div>
						</button>
					))}
				</div>
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
			{!(isNaN(Number(amount)) || Number(amount) <= 0) && (
				<div style={{ margin: "-8px 0 12px" }}>
					<FeeCalculator />
				</div>
			)}
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
			<FormField label="Memo (optional)">
				<input
					value={memo}
					onChange={(e) => setMemo(e.target.value)}
					placeholder="e.g. Invoice #42 — Website redesign"
					maxLength={200}
				/>
			</FormField>
			<FormField label="Auto-settle">
				<label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
					<input
						type="checkbox"
						checked={autoSettle}
						onChange={(e) => setAutoSettle(e.target.checked)}
					/>
					Auto-settle after timeout (no signature needed)
				</label>
			</FormField>
			{autoSettle && (
				<FormField label="Timeout duration">
					<select
						value={autoSettleDuration}
						onChange={(e) => setAutoSettleDuration(Number(e.target.value))}
					>
						<option value={3600}>1 hour</option>
						<option value={7200}>2 hours</option>
						<option value={14400}>4 hours</option>
						<option value={43200}>12 hours</option>
						<option value={86400}>24 hours</option>
						<option value={259200}>3 days</option>
						<option value={604800}>7 days</option>
					</select>
				</FormField>
			)}
			{/* Security deposit — backend support not yet implemented */}
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

/* ─── Milestone Progress Bar ─── */
function MilestoneProgressBar({
	milestoneStatuses,
	currentMilestone,
}: { milestoneStatuses: string[]; currentMilestone: number }) {
	const labels = ["M1", "M2", "M3", "M4", "M5"];
	return (
		<div style={{ display: "flex", gap: "4px", alignItems: "center", margin: "8px 0" }}>
			{milestoneStatuses.map((s, i) => {
				const done = s === "released" || s === "approved";
				const active = i === currentMilestone && s === "pending";
				return (
					<div
						key={i}
						style={{
							flex: 1,
							height: "8px",
							borderRadius: "4px",
							background: done ? "#4caf50" : active ? "#ff9800" : "#333",
							transition: "background 0.3s",
							position: "relative",
						}}
						title={`${labels[i]}: ${s}`}
					>
						<span
							style={{
								position: "absolute",
								top: "-18px",
								left: "50%",
								transform: "translateX(-50%)",
								fontSize: "10px",
								color: done ? "#4caf50" : active ? "#ff9800" : "#666",
							}}
						>
							{labels[i]}
						</span>
					</div>
				);
			})}
		</div>
	);
}

/* ─── My Milestones ─── */
function MyMilestones({ address }: { address: string }) {
	const [milestones, setMilestones] = useState<LoadState<MilestoneEscrow[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const loaded = useRef(false);

	const fetch = useCallback(() => {
		setMilestones({ loading: true });
		api
			.milestones(address)
			.then((d) => setMilestones({ data: d.milestones, loading: false }))
			.catch((e) => setMilestones({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => {
		if (loaded.current) return;
		loaded.current = true;
		fetch();
	}, [fetch]);

	if (milestones.loading) return <SkeletonTable rows={3} />;
	if (milestones.error) return <p className="muted error-text">{milestones.error}</p>;
	if (!milestones.data?.length)
		return (
			<EmptyState
				icon="🏗️"
				title="No milestone escrows"
				description="Create a milestone-based escrow for phased payments."
			/>
		);

	return (
		<div>
			{milestones.data.map((m) => (
				<article
					key={m.id}
					className="offer"
					style={{ cursor: "pointer", marginBottom: "8px" }}
					onClick={() => setSelectedId(selectedId === m.id ? null : m.id)}
				>
					<div className="offer-top">
						<strong>{money(m.total_amount)}</strong>
						<span className={badge(m.status)}>{m.status}</span>
					</div>
					<p>
						{m.buyer_address.slice(0, 16)}… → {m.seller_address.slice(0, 16)}…
					</p>
					<MilestoneProgressBar
						milestoneStatuses={m.milestone_statuses}
						currentMilestone={m.current_milestone}
					/>
					<code>{m.id}</code>
					{selectedId === m.id && <MilestoneActions escrow={m} onMutated={fetch} />}
				</article>
			))}
		</div>
	);
}

/* ─── Milestone Action Buttons ─── */
function MilestoneActions({
	escrow,
	onMutated,
}: { escrow: MilestoneEscrow; onMutated: () => void }) {
	const { notify } = useToast();
	const [loading, setLoading] = useState("");

	const isFinal = ["completed", "refunded"].includes(escrow.status);
	if (isFinal) return <p className="muted"> Finalized — {escrow.status}</p>;

	async function doAction(action: "release" | "approve" | "dispute" | "refund" | "complete") {
		setLoading(action);
		try {
			if (action === "release") await api.releaseMilestone(escrow.id);
			else if (action === "approve") await api.approveMilestone(escrow.id);
			else if (action === "dispute") await api.disputeMilestone(escrow.id);
			else if (action === "refund") await api.refundMilestone(escrow.id);
			else await api.completeMilestone(escrow.id);
			notify("success", `Milestone ${action} successful`);
			onMutated();
		} catch (e) {
			notify("error", `Failed to ${action}`, (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	const idx = escrow.current_milestone;
	const currentPending =
		idx < escrow.milestone_statuses.length && escrow.milestone_statuses[idx] === "pending";

	return (
		<div className="offer-actions" style={{ marginTop: "12px" }}>
			{escrow.status === "active" && currentPending && (
				<>
					<button
						className="button primary"
						disabled={!!loading}
						onClick={() => doAction("release")}
						style={{ marginRight: "8px" }}
					>
						{loading === "release" ? "Releasing…" : "Release M" + (idx + 1)}
					</button>
					<button
						className="button"
						disabled={!!loading}
						onClick={() => doAction("approve")}
						style={{ marginRight: "8px" }}
					>
						{loading === "approve" ? "Approving…" : "Approve M" + (idx + 1)}
					</button>
				</>
			)}
			{escrow.status === "active" && (
				<button
					className="button"
					disabled={!!loading}
					onClick={() => doAction("complete")}
					style={{ marginRight: "8px" }}
				>
					{loading === "complete" ? "Completing…" : "Complete All"}
				</button>
			)}
			{(escrow.status === "active" || escrow.status === "disputed") && (
				<button
					className="button"
					disabled={!!loading}
					onClick={() => doAction("refund")}
					style={{ marginRight: "8px" }}
				>
					{loading === "refund" ? "Refunding…" : "Refund"}
				</button>
			)}
			{escrow.status === "active" && (
				<button className="button" disabled={!!loading} onClick={() => doAction("dispute")}>
					{loading === "dispute" ? "Disputing…" : "Dispute"}
				</button>
			)}
			{escrow.status === "disputed" && <p className="muted"> Under dispute</p>}
		</div>
	);
}

/* ─── Create Milestone Form ─── */
function CreateMilestoneForm({ address }: { address: string }) {
	const [sellerAddress, setSellerAddress] = useState("");
	const [totalAmount, setTotalAmount] = useState("");
	const [milestoneCount, setMilestoneCount] = useState(3);
	const [amounts, setAmounts] = useState<string[]>(["", "", ""]);
	const [timeouts, setTimeouts] = useState<string[]>(["", "", ""]);
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [result, setResult] = useState<MilestoneEscrow | null>(null);
	const { notify } = useToast();

	function handleCountChange(count: number) {
		const clamped = Math.max(1, Math.min(5, count));
		setMilestoneCount(clamped);
		setAmounts((prev) => {
			const next = [...prev];
			while (next.length < clamped) next.push("");
			return next.slice(0, clamped);
		});
		setTimeouts((prev) => {
			const next = [...prev];
			while (next.length < clamped) next.push("");
			return next.slice(0, clamped);
		});
	}

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const totalNum = Number.parseFloat(totalAmount);
		if (!totalNum || totalNum <= 0) return;

		const amountNums = amounts.map((a) => Number.parseFloat(a));
		if (amountNums.some((a) => isNaN(a) || a <= 0)) return;

		const timeoutNums = timeouts.map((t) => {
			const days = Number.parseFloat(t);
			return isNaN(days) || days <= 0 ? 0 : Math.floor(Date.now() / 1000) + days * 86400;
		});

		const sompiAmounts = amountNums.map((a) => Math.round(a * 100_000_000));
		const sompiTotal = Math.round(totalNum * 100_000_000);

		setStatus("loading");
		try {
			const lockTxId = prompt("Enter lock transaction ID:") || "";
			if (!lockTxId) throw new Error("Transaction ID required");

			const escrow = await api.createMilestone({
				lock_tx_id: lockTxId,
				buyer_address: address,
				seller_address: sellerAddress,
				total_amount: sompiTotal,
				milestone_amounts: sompiAmounts,
				milestone_timeouts: timeoutNums,
			});
			setResult(escrow);
			setStatus("done");
			notify("success", "Milestone escrow created!");
		} catch (e) {
			notify("error", "Failed to create milestone", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done" && result) {
		return (
			<EmptyState
				icon="✅"
				title="Milestone escrow created!"
				description={`ID: ${result.id} | Status: ${result.status} | ${result.milestone_amounts.length} milestones`}
			/>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				You: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>
			<FormField label="Seller address">
				<input
					value={sellerAddress}
					onChange={(e) => setSellerAddress(e.target.value)}
					placeholder="kaspa:..."
					required
				/>
			</FormField>
			<FormField label="Total amount (KAS)">
				<input
					type="number"
					step="any"
					value={totalAmount}
					onChange={(e) => setTotalAmount(e.target.value)}
					placeholder="1000"
					required
				/>
			</FormField>
			<FormField label="Number of milestones">
				<input
					type="number"
					min={1}
					max={5}
					value={milestoneCount}
					onChange={(e) => handleCountChange(Number.parseInt(e.target.value) || 1)}
				/>
			</FormField>
			{amounts.map((amt, i) => (
				<div key={i} style={{ display: "flex", gap: "8px", marginBottom: "8px" }}>
					<FormField label={`M${i + 1} amount (KAS)`}>
						<input
							type="number"
							step="any"
							value={amt}
							onChange={(e) => {
								const next = [...amounts];
								next[i] = e.target.value;
								setAmounts(next);
							}}
							placeholder="333"
							required
						/>
					</FormField>
					<FormField label={`M${i + 1} timeout (days)`}>
						<input
							type="number"
							min={1}
							value={timeouts[i]}
							onChange={(e) => {
								const next = [...timeouts];
								next[i] = e.target.value;
								setTimeouts(next);
							}}
							placeholder="7"
							required
						/>
					</FormField>
				</div>
			))}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
				style={{ marginTop: "12px" }}
			>
				{status === "loading" ? "Creating…" : "Create Milestone Escrow"}
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
							<strong>
								{money(escrow.data.amount_sompi)}
								{escrow.data.price_at_creation && (
									<span style={{ fontSize: "11px", color: "#888", marginLeft: "8px" }}>
										(~${escrow.data.price_at_creation.toFixed(2)} at creation)
									</span>
								)}
							</strong>
						</div>
						<div className="row">
							<span>Fee (0.5%)</span>
							<strong>{money(Math.round(escrow.data.amount_sompi / 200))}</strong>
						</div>
						<div className="row">
							<span>Lock TX</span>
							<ExplorerTxLink txid={escrow.data.lock_tx_id} />
						</div>
						<div className="row">
							<span>Buyer</span>
							<strong className="addr">
								{escrow.data.buyer_address.slice(0, 20)}…
								<ExplorerAddressLink address={escrow.data.buyer_address} />
							</strong>
						</div>
						{escrow.data.seller_address && (
							<div className="row">
								<span>Seller</span>
								<strong className="addr">
									{escrow.data.seller_address.slice(0, 20)}…
									<ExplorerAddressLink address={escrow.data.seller_address} />
								</strong>
							</div>
						)}
						{escrow.data.dispute_reason && (
							<div className="row">
								<span>Dispute</span>
								<strong>{escrow.data.dispute_reason}</strong>
							</div>
						)}
						{escrow.data.auto_settle_timeout && (
							<div className="row">
								<span>Deal Type</span>
								<DealTypeBadge escrow={escrow.data} />
							</div>
						)}
						{escrow.data.dispute_mode && (
							<div className="row">
								<span>Mode</span>
								<strong>{escrow.data.dispute_mode}</strong>
							</div>
						)}
						{escrow.data.memo && (
							<div className="row">
								<span>Memo</span>
								<strong>{escrow.data.memo}</strong>
							</div>
						)}
					</div>
				</div>
			)}
		</div>
	);
}

/* ─── Multi-Party Escrows ─── */
function MyMultiEscrows({ address }: { address: string }) {
	const [escrows, setEscrows] = useState<LoadState<MultiEscrow[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const loaded = useRef(false);

	const fetch = useCallback(() => {
		setEscrows({ loading: true });
		api
			.multiEscrows(address)
			.then((d) => setEscrows({ data: d.multi_escrows, loading: false }))
			.catch((e) => setEscrows({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => {
		if (loaded.current) return;
		loaded.current = true;
		fetch();
	}, [fetch]);

	if (escrows.loading) return <SkeletonTable rows={3} />;
	if (escrows.error) return <p className="muted error-text">{escrows.error}</p>;
	if (!escrows.data?.length)
		return (
			<EmptyState
				icon="👥"
				title="No multi-party escrows"
				description="Create a multi-party escrow to distribute KAS among up to 4 parties."
			/>
		);

	return (
		<div>
			{escrows.data.map((m) => (
				<article
					key={m.id}
					className="offer"
					style={{ cursor: "pointer", marginBottom: "8px" }}
					onClick={() => setSelectedId(selectedId === m.id ? null : m.id)}
				>
					<div className="offer-top">
						<strong>{money(m.total_amount)}</strong>
						<span className={badge(m.status)}>{m.status}</span>
					</div>
					<p>
						{m.parties.length} parties · {m.signatures.length}/{m.parties.length} signed
					</p>
					<code>{m.id}</code>
					{selectedId === m.id && <MultiEscrowActions escrow={m} onMutated={fetch} />}
				</article>
			))}
		</div>
	);
}

function MultiEscrowActions({ escrow, onMutated }: { escrow: MultiEscrow; onMutated: () => void }) {
	const { notify } = useToast();
	const { sign, state } = useWallet();
	const walletAddress = state.address;
	const [loading, setLoading] = useState("");

	const isFinal = ["settled", "refunded"].includes(escrow.status);
	if (isFinal) return <p className="muted"> Finalized — {escrow.status}</p>;

	const myIndex = escrow.parties.findIndex((p) => p === walletAddress);
	const hasSigned = escrow.signatures.includes(walletAddress || "");
	const allSigned = escrow.signatures.length === escrow.parties.length;

	async function doSign() {
		if (!walletAddress) return;
		setLoading("sign");
		try {
			const result = await api.signMultiEscrow(escrow.id, walletAddress);
			notify("success", `Signed! (${result.signature_count}/${result.parties_count})`);
			onMutated();
		} catch (e) {
			notify("error", "Failed to sign", (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	async function doSwap() {
		setLoading("swap");
		try {
			await api.swapMultiEscrow(escrow.id);
			notify("success", "Multi-party escrow settled");
			onMutated();
		} catch (e) {
			notify("error", "Failed to settle", (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	async function doRefund() {
		setLoading("refund");
		try {
			await api.refundMultiEscrow(escrow.id);
			notify("success", "Multi-party escrow refunded");
			onMutated();
		} catch (e) {
			notify("error", "Failed to refund", (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	return (
		<div className="offer-actions" style={{ marginTop: "12px" }}>
			<div style={{ marginBottom: "8px", fontSize: "13px" }}>
				{escrow.parties.map((p, i) => {
					const signed = escrow.signatures.includes(p);
					const share = ((escrow.shares[i] || 0) / 100).toFixed(1);
					return (
						<div key={i} style={{ color: signed ? "#4caf50" : "#888", marginBottom: "2px" }}>
							{signed ? "✓" : "○"} {p.slice(0, 16)}… — {share}%
						</div>
					);
				})}
			</div>
			{!hasSigned && myIndex !== -1 && (
				<button
					className="button primary"
					disabled={!!loading}
					onClick={doSign}
					style={{ marginRight: "8px" }}
				>
					{loading === "sign" ? "Signing…" : "Sign Release"}
				</button>
			)}
			{hasSigned && !allSigned && (
				<p className="muted" style={{ fontSize: "12px" }}>
					Waiting for other parties…
				</p>
			)}
			{allSigned && (
				<button
					className="button primary"
					disabled={!!loading}
					onClick={doSwap}
					style={{ marginRight: "8px" }}
				>
					{loading === "swap" ? "Settling…" : "Execute Swap"}
				</button>
			)}
			<button className="button" disabled={!!loading} onClick={doRefund}>
				{loading === "refund" ? "Refunding…" : "Refund"}
			</button>
		</div>
	);
}

function CreateMultiForm({ address }: { address: string }) {
	const [parties, setParties] = useState<string[]>([address, ""]);
	const [shares, setShares] = useState<string[]>(["", ""]);
	const [totalAmount, setTotalAmount] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [result, setResult] = useState<MultiEscrow | null>(null);
	const { notify } = useToast();

	function addParty() {
		if (parties.length >= 4) return;
		setParties([...parties, ""]);
		setShares([...shares, ""]);
	}

	function removeParty(i: number) {
		if (parties.length <= 2) return;
		setParties(parties.filter((_, idx) => idx !== i));
		setShares(shares.filter((_, idx) => idx !== i));
	}

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const totalNum = Number.parseFloat(totalAmount);
		if (!totalNum || totalNum <= 0) return;

		const shareNums = shares.map((s) => Math.round(Number.parseFloat(s) * 100));
		if (shareNums.some((s) => isNaN(s) || s <= 0)) return;
		const totalShares = shareNums.reduce((a, b) => a + b, 0);
		if (totalShares !== 10000) {
			notify("error", `Shares must sum to 100.00%, got ${(totalShares / 100).toFixed(2)}%`);
			return;
		}
		if (parties.some((p) => !p.startsWith("kaspa:"))) {
			notify("error", "All parties must have valid kaspa: addresses");
			return;
		}
		const unique = new Set(parties);
		if (unique.size !== parties.length) {
			notify("error", "Duplicate party addresses not allowed");
			return;
		}

		const sompiTotal = Math.round(totalNum * 100_000_000);

		setStatus("loading");
		try {
			const lockTxId = prompt("Enter lock transaction ID:") || "";
			if (!lockTxId) throw new Error("Transaction ID required");

			const escrow = await api.createMultiEscrow({
				lock_tx_id: lockTxId,
				parties,
				shares: shareNums,
				total_amount: sompiTotal,
			});
			setResult(escrow);
			setStatus("done");
			notify("success", "Multi-party escrow created!");
		} catch (e) {
			notify("error", "Failed to create multi-party escrow", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done" && result) {
		return (
			<EmptyState
				icon="✅"
				title="Multi-party escrow created!"
				description={`ID: ${result.id} | ${result.parties.length} parties`}
			/>
		);
	}

	const totalPct = shares.reduce((a, s) => a + (Number.parseFloat(s) || 0), 0);

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				You: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>
			<FormField label="Total amount (KAS)">
				<input
					type="number"
					step="any"
					value={totalAmount}
					onChange={(e) => setTotalAmount(e.target.value)}
					placeholder="10000"
					required
				/>
			</FormField>
			{parties.map((party, i) => (
				<div
					key={i}
					style={{ display: "flex", gap: "8px", marginBottom: "8px", alignItems: "flex-end" }}
				>
					<div style={{ flex: 1 }}>
						<FormField label={`Party ${i + 1} address`}>
							<input
								value={party}
								onChange={(e) => {
									const next = [...parties];
									next[i] = e.target.value;
									setParties(next);
								}}
								placeholder="kaspa:..."
								required
							/>
						</FormField>
					</div>
					<div style={{ width: "120px" }}>
						<FormField label="Share %">
							<input
								type="number"
								step="0.01"
								value={shares[i]}
								onChange={(e) => {
									const next = [...shares];
									next[i] = e.target.value;
									setShares(next);
								}}
								placeholder="25"
								required
							/>
						</FormField>
					</div>
					{parties.length > 2 && (
						<button
							type="button"
							className="button"
							onClick={() => removeParty(i)}
							style={{ padding: "4px 8px", fontSize: "12px", marginBottom: "8px" }}
						>
							✕
						</button>
					)}
				</div>
			))}
			<div
				style={{
					fontSize: "12px",
					color: totalPct > 100 ? "#f44336" : totalPct === 100 ? "#4caf50" : "#888",
					marginBottom: "8px",
				}}
			>
				Total: {totalPct.toFixed(2)}% {totalPct === 100 ? "(✓)" : totalPct > 100 ? "(over)" : ""}
			</div>
			{parties.length < 4 && (
				<button
					type="button"
					className="button"
					onClick={addParty}
					style={{ marginBottom: "12px" }}
				>
					+ Add Party
				</button>
			)}
			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
				style={{ marginTop: "12px" }}
			>
				{status === "loading" ? "Creating…" : "Create Multi-Party Escrow"}
			</button>
		</form>
	);
}
