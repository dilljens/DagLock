import { useState, useEffect, useMemo } from "react";
import { api, type Health, type Stats, type Offer, type Escrow } from "../api";
import { useRouter } from "../router";
import { useWallet } from "../context/WalletContext";
import { money } from "../helpers";
import type { LoadState } from "../helpers";

interface DashboardProps {
	health: LoadState<Health>;
	stats: LoadState<Stats>;
	offers: LoadState<Offer[]>;
}

export function Dashboard({ health, stats, offers }: DashboardProps) {
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
			<div>
				<div className="dashboard-hero">
					<h2> Trustless Escrow on Kaspa</h2>
					<p>
						Create, settle, and dispute escrows without trusting anyone. Connect your KasWare wallet
						to get started.
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
				<div className="action-grid">
					<div className="action-card" onClick={() => navigate("/")}>
						<span className="action-card-icon"></span>
						<span className="action-card-label">Browse Offers</span>
						<span className="action-card-desc">View open offers from the community</span>
					</div>
					<div className="action-card" onClick={() => navigate("/reputation")}>
						<span className="action-card-icon"></span>
						<span className="action-card-label">Check Reputation</span>
						<span className="action-card-desc">Look up any address's trading history</span>
					</div>
				</div>
			</div>
		);
	}

	// Connected — show dashboard
	const myEscrows = escrows.data || [];
	const activeEscrows = myEscrows.filter((e) =>
		["pending_confirmation", "active", "disputed"].includes(e.status),
	);
	const settledCount = myEscrows.filter((e) => e.status === "settled").length;

	const s = stats.data;
	const highlights = [
		["Escrows", s?.total_escrows ?? "—"],
		["Active", s?.active_escrows ?? "—"],
		["Volume", s ? money(s.total_volume_kas) : "—"],
		["Settled", s?.settled_escrows ?? "—"],
	];

	return (
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
					<span className="action-card-icon"></span>
					<span className="action-card-label">Create Offer</span>
					<span className="action-card-desc">List a trade for others to accept</span>
				</div>
				<div className="action-card" onClick={() => navigate("/escrows")}>
					<span className="action-card-icon"></span>
					<span className="action-card-label">Create Escrow</span>
					<span className="action-card-desc">Lock funds in a trustless covenant</span>
				</div>
				<div className="action-card" onClick={() => navigate("/vaults")}>
					<span className="action-card-icon"></span>
					<span className="action-card-label">Create Vault</span>
					<span className="action-card-desc">Time-locked self-custody storage</span>
				</div>
			</div>

			{/* My active escrows */}
			<h3 style={{ margin: "24px 0 12px" }}>Active Escrows</h3>
			{escrows.loading && <p className="muted">Loading escrows…</p>}
			{activeEscrows.length === 0 && !escrows.loading && (
				<div className="empty-state">
					<div className="empty-state-icon"></div>
					<h3>No active escrows</h3>
					<p>Create your first escrow to start trading trustlessly.</p>
					<button className="button primary" onClick={() => navigate("/escrows")}>
						Create Escrow
					</button>
				</div>
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
				<div className="empty-state" style={{ marginTop: "12px" }}>
					<div className="empty-state-icon"></div>
					<h3>{settledCount} settled escrows</h3>
					<p>Tap to close the dashboard empty state.</p>
				</div>
			)}
		</div>
	);
}
