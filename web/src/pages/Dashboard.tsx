import { useState, useEffect } from "react";
import { api, type Stats, type Escrow } from "../api";
import { useRouter } from "../router";
import { useWallet } from "../context/WalletContext";
import { money } from "../helpers";
import type { LoadState } from "../helpers";
import { SkeletonTable, SkeletonStats } from "../ui";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";

interface DashboardProps {
	stats?: Stats;
}

export function Dashboard({ stats }: DashboardProps) {
	const { navigate } = useRouter();
	const { state: wallet, connect } = useWallet();
	const [escrows, setEscrows] = useState<LoadState<Escrow[]>>({ loading: false });

	// Auto-fetch escrows when wallet connects
	useEffect(() => {
		if (!wallet.connected || !wallet.address) return;
		setEscrows({ loading: true });
		api
			.escrows(wallet.address)
			.then((d) => setEscrows({ data: d.escrows, loading: false }))
			.catch((e) => setEscrows({ error: e.message, loading: false }));
	}, [wallet.connected, wallet.address]);

	// Not connected — show hero
	if (!wallet.connected) {
		return (
			<>
				<Helmet>
					<title>DagLock — Trustless Escrow & Atomic Swaps on Kaspa</title>
					<meta
						name="description"
						content="Browse escrow offers, manage vaults, and trade KAS and KRC-20 tokens on Kaspa L1."
					/>
					<link rel="canonical" href="https://daglock.com/" />
				</Helmet>
				<div>
					<div className="dashboard-hero">
						<h2> Covenant Escrow on Kaspa</h2>
						<p>
							Create, settle, and dispute escrows secured by SilverScript covenants — no admin keys,
							no backdoors. Only the <strong>buyer</strong> or <strong>seller</strong> can settle. A{" "}
							<strong>0.5% escrow fee</strong> (and <strong>0.1% vault fee</strong>) is enforced by
							the covenant and paid to the DagLock treasury.
						</p>
						{wallet.detected ? (
							<button
								className="button primary"
								onClick={connect}
								disabled={wallet.loading}
								style={{ fontSize: "16px", padding: "14px 28px" }}
							>
								{wallet.loading ? "Connecting…" : "Connect Wallet"}
							</button>
						) : (
							<a
								href="https://kasware.xyz"
								target="_blank"
								rel="noopener noreferrer"
								className="button primary"
								style={{ fontSize: "16px", padding: "14px 28px" }}
							>
								Install KasWare
							</a>
						)}
					</div>

					{/* What is DagLock — feature cards */}
					<h2 style={{ marginTop: "36px", marginBottom: "4px" }}>What is DagLock?</h2>
					<p className="muted" style={{ marginTop: 0, marginBottom: "20px" }}>
						Three ways to use Kaspa's covenant system without writing SilverScript yourself.
					</p>
					<div className="feature-cards">
						<div className="feature-card">
							<div className="feature-card-icon">🤝</div>
							<h3>Trustless Escrow</h3>
							<p>
								Lock KAS or KRC-20 tokens in a SilverScript covenant. Only the buyer or seller can
								settle — neither can steal.
							</p>
						</div>
						<div className="feature-card">
							<div className="feature-card-icon">🏦</div>
							<h3>Time-Locked Vaults</h3>
							<p>
								Self-custody storage with configurable timeouts. Standard, password-recoverable, or
								multi-sig vaults. 0.1% fee on withdrawal.
							</p>
						</div>
						<div className="feature-card">
							<div className="feature-card-icon">🔄</div>
							<h3>Atomic Swaps</h3>
							<p>
								Cross-asset trades via hash preimage. Both parties commit funds, then reveal the
								secret to settle.
							</p>
						</div>
					</div>

					{/* How it works */}
					<h2 style={{ marginTop: "36px", marginBottom: "4px" }}>How It Works</h2>
					<p className="muted" style={{ marginTop: 0, marginBottom: "20px" }}>
						From offer to settlement in 3 steps.
					</p>
					<div className="feature-cards">
						<div className="feature-card">
							<div className="feature-card-icon">1️⃣</div>
							<h3>Create or Accept</h3>
							<p>
								Create an offer or accept one from the board. An escrow is created — funds aren't
								locked yet.
							</p>
						</div>
						<div className="feature-card">
							<div className="feature-card-icon">2️⃣</div>
							<h3>Lock Funds</h3>
							<p>
								The buyer sends KAS to a covenant address. The covenant enforces the rules — no one
								can steal.
							</p>
						</div>
						<div className="feature-card">
							<div className="feature-card-icon">3️⃣</div>
							<h3>Settle or Refund</h3>
							<p>
								Both parties agree → funds released to seller. Or buyer refunds after timeout. 0.5%
								escrow fee / 0.1% vault fee goes to the treasury.
							</p>
						</div>
					</div>

					<p
						className="muted"
						style={{ fontSize: "13px", marginBottom: "24px", lineHeight: 1.6, maxWidth: "600px" }}
					>
						<strong>Note:</strong> DagLock has no custody of your funds. The covenant is compiled
						into a Kaspa script address — your wallet sends KAS there directly. DagLock never
						touches your keys, and the covenant enforces the rules without us. Read the source on{" "}
						<a
							href="https://github.com/dilljens/DagLock"
							target="_blank"
							rel="noopener noreferrer"
							style={{ color: "var(--color-primary)", textDecoration: "underline" }}
						>
							GitHub
						</a>
						.
					</p>

					<h2 style={{ margin: "36px 0 12px" }}>Get Started</h2>
					<div className="action-grid">
						<div className="action-card" onClick={() => navigate("/offers")}>
							<span className="action-card-icon">📋</span>
							<span className="action-card-label">Browse Offers</span>
							<span className="action-card-desc">View open offers from the community</span>
						</div>
						<div className="action-card" onClick={() => navigate("/escrows")}>
							<span className="action-card-icon">🔒</span>
							<span className="action-card-label">Create Escrow</span>
							<span className="action-card-desc">Lock funds in a trustless covenant</span>
						</div>
						<div className="action-card" onClick={() => navigate("/docs")}>
							<span className="action-card-icon">📖</span>
							<span className="action-card-label">Developer Docs</span>
							<span className="action-card-desc">API reference, CLI, bot, integrations</span>
						</div>
					</div>
				</div>
			</>
		);
	}

	// Connected — show dashboard
	const myEscrows = escrows.data || [];
	const activeEscrows = myEscrows.filter((e) =>
		["pending_confirmation", "active", "disputed"].includes(e.status),
	);
	const settledCount = myEscrows.filter((e) => e.status === "settled").length;

	const s = stats;
	const highlights = [
		["Escrows", s?.total_escrows ?? "—"],
		["Active", s?.active_escrows ?? "—"],
		["Volume", s ? money(s.total_volume_kas) : "—"],
		["Settled", s?.settled_escrows ?? "—"],
	];

	return (
		<>
			<Helmet>
				<title>DagLock — Trustless Escrow & Atomic Swaps on Kaspa</title>
				<meta
					name="description"
					content="Browse escrow offers, manage vaults, and trade KAS and KRC-20 tokens on Kaspa L1."
				/>
				<link rel="canonical" href="https://daglock.com/" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Dashboard</h1>
					<p>
						{wallet.address?.slice(0, 24)}… · Balance: {wallet.balance} KAS
					</p>
				</div>

				{/* Stats */}
				<div className="stats-grid">
					{highlights.map(([label, value]) => (
						<div key={label} className="stat-card">
							<div className="stat-card-label">{label}</div>
							<div className="stat-card-value">{value}</div>
						</div>
					))}
				</div>

				{/* Quick actions */}
				<h3 style={{ margin: "0 0 12px" }}>Quick Actions</h3>
				<div className="action-grid">
					<div className="action-card" onClick={() => navigate("/offers")}>
						<span className="action-card-icon" />
						<span className="action-card-label">Create Offer</span>
						<span className="action-card-desc">List a trade for others to accept</span>
					</div>
					<div className="action-card" onClick={() => navigate("/escrows")}>
						<span className="action-card-icon" />
						<span className="action-card-label">Create Escrow</span>
						<span className="action-card-desc">Lock funds in a trustless covenant</span>
					</div>
					<div className="action-card" onClick={() => navigate("/vaults")}>
						<span className="action-card-icon" />
						<span className="action-card-label">Create Vault</span>
						<span className="action-card-desc">Time-locked self-custody storage</span>
					</div>
				</div>

				{/* My active escrows */}
				<h3 style={{ margin: "24px 0 12px" }}>Active Escrows</h3>
				{escrows.loading && <SkeletonTable rows={4} />}
				{activeEscrows.length === 0 && !escrows.loading && (
					<EmptyState
						icon="🤝"
						title="No active escrows"
						description="Create your first escrow to start trading trustlessly."
						action={{ label: "Create Escrow", onClick: () => navigate("/escrows") }}
					/>
				)}
				{activeEscrows.slice(0, 5).map((e) => (
					<article
						key={e.id}
						className="offer"
						style={{ cursor: "pointer", marginBottom: "8px" }}
						onClick={() => navigate("/escrows")}
					>
						<div className="offer-top">
							<strong>{money(e.amount_sompi)}</strong>
							<span className="pill">{e.status}</span>
						</div>
						<p>
							{e.asset_type} · {e.buyer_address.slice(0, 16)}…
						</p>
						<code>{e.id}</code>
					</article>
				))}
				{settledCount > 0 && !escrows.loading && (
					<EmptyState
						icon="📊"
						title={`${settledCount} settled escrows`}
						description="Tap to close the dashboard empty state."
					/>
				)}
			</div>
		</>
	);
}
