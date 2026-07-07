import { useState, useEffect } from "react";
import { api } from "../api";
import { useRouter } from "../router";
import { money, relativeTime } from "../helpers";
import type { LoadState } from "../helpers";
import { Helmet } from "react-helmet-async";
import { SkeletonTable } from "../ui";
import { ExplorerEscrowLink } from "../components/ExplorerLink";

type TokenDetail = {
	ticker: string;
	price_kas: number | null;
	volume_24h_sompi: number;
	trades_24h: number;
	total_trades: number;
	active_offers: number;
	last_trade_at: number | null;
	trades: {
		escrow_id: string;
		amount_sompi: number;
		status: string;
		created_at: number;
		buyer_address: string;
	}[];
};

type ChartPoint = {
	timestamp: number;
	volume_kas: number;
};

function formatPrice(price: number | null): string {
	if (price === null || price === 0) return "—";
	if (price < 0.0001) return price.toExponential(2);
	if (price < 1) return price.toFixed(6);
	return price.toFixed(2);
}

function SimpleVolumeChart({ points, height = 120 }: { points: ChartPoint[]; height?: number }) {
	if (points.length < 2) return null;

	const volumes = points.map((p) => p.volume_kas);
	const min = Math.min(...volumes);
	const max = Math.max(...volumes);
	const range = max - min || 1;
	const width = 400;

	const pathD = points
		.map((p, i) => {
			const x = (i / (points.length - 1)) * width;
			const y = height - ((p.volume_kas - min) / range) * (height - 20) - 10;
			return `${i === 0 ? "M" : "L"}${x.toFixed(0)},${y.toFixed(0)}`;
		})
		.join(" ");

	return (
		<svg
			viewBox={`0 0 ${width} ${height}`}
			style={{ width: "100%", height: "auto", maxHeight: height }}
		>
			<path d={pathD} fill="none" stroke="#53d769" strokeWidth="2" strokeLinecap="round" />
			<path
				d={`${pathD} L${width},${height} L0,${height} Z`}
				fill="url(#volGradient)"
				opacity="0.15"
			/>
			<defs>
				<linearGradient id="volGradient" x1="0" x2="0" y1="0" y2="1">
					<stop offset="0%" stopColor="#53d769" />
					<stop offset="100%" stopColor="#53d769" stopOpacity="0" />
				</linearGradient>
			</defs>
		</svg>
	);
}

export function TokenDetailPage({ ticker }: { ticker: string }) {
	const { navigate } = useRouter();
	const [detail, setDetail] = useState<LoadState<TokenDetail>>({ loading: true });
	const [chartData, setChartData] = useState<ChartPoint[]>([]);
	const [chartPeriod, setChartPeriod] = useState("7d");

	useEffect(() => {
		api
			.token(ticker)
			.then((d: TokenDetail) => setDetail({ data: d, loading: false }))
			.catch((e: Error) => setDetail({ error: e.message, loading: false }));
	}, [ticker]);

	useEffect(() => {
		api
			.tokenChart(ticker, chartPeriod)
			.then((d: any) => setChartData(d.points || []))
			.catch(() => setChartData([]));
	}, [ticker, chartPeriod]);

	if (detail.loading) return <SkeletonTable rows={5} />;
	if (detail.error) {
		return (
			<div>
				<div className="page-header">
					<h1>Token Not Found</h1>
					<p>{detail.error}</p>
				</div>
				<button className="button" onClick={() => navigate("/tokens")}>
					← Back to Tokens
				</button>
			</div>
		);
	}

	const d = detail.data!;

	return (
		<>
			<Helmet>
				<title>{d.ticker} — DagLock Tokens</title>
				<meta
					name="description"
					content={`${d.ticker} token price, volume, and trades on DagLock escrow.`}
				/>
			</Helmet>
			<div>
				<div className="page-header">
					<button
						className="button"
						onClick={() => navigate("/tokens")}
						style={{ marginBottom: "8px" }}
					>
						← Tokens
					</button>
					<h1 style={{ margin: 0 }}>{d.ticker}</h1>
					<p>KRC-20 token traded on DagLock escrow</p>
				</div>

				{/* Stats grid */}
				<div className="stats-grid" style={{ marginBottom: "20px" }}>
					<div className="stat-card">
						<div className="stat-card-label">Price</div>
						<div className="stat-card-value">{formatPrice(d.price_kas)} KAS</div>
					</div>
					<div className="stat-card">
						<div className="stat-card-label">Volume (24h)</div>
						<div className="stat-card-value">{money(d.volume_24h_sompi)}</div>
					</div>
					<div className="stat-card">
						<div className="stat-card-label">Trades (24h)</div>
						<div className="stat-card-value">{d.trades_24h}</div>
					</div>
					<div className="stat-card">
						<div className="stat-card-label">Active Offers</div>
						<div className="stat-card-value">{d.active_offers}</div>
					</div>
				</div>

				{/* Price chart */}
				{chartData.length > 1 && (
					<div className="panel" style={{ marginBottom: "20px" }}>
						<div
							style={{
								display: "flex",
								justifyContent: "space-between",
								alignItems: "center",
								marginBottom: "12px",
							}}
						>
							<h3 style={{ margin: 0 }}>Price Chart</h3>
							<div style={{ display: "flex", gap: "4px" }}>
								{["7d", "30d", "all"].map((p) => (
									<button
										key={p}
										className={`button ${chartPeriod === p ? "primary" : ""}`}
										onClick={() => setChartPeriod(p)}
										style={{ padding: "4px 12px", fontSize: "12px" }}
									>
										{p}
									</button>
								))}
							</div>
						</div>
						<SimpleVolumeChart points={chartData} />
						<p className="muted" style={{ fontSize: "11px", marginTop: "4px" }}>
							KAS trade volume over time (token price requires per-trade token amount data)
						</p>
					</div>
				)}

				{/* Quick actions */}
				<div style={{ display: "flex", gap: "12px", marginBottom: "20px" }}>
					<button className="button primary" onClick={() => navigate("/escrows" as any)}>
						Buy {d.ticker}
					</button>
					<button className="button" onClick={() => navigate("/offers" as any)}>
						Sell {d.ticker}
					</button>
				</div>

				{/* Recent trades */}
				<div className="panel">
					<h3 style={{ margin: "0 0 12px" }}>Recent Trades</h3>
					{d.trades.length === 0 ? (
						<p className="muted">No trades yet. Be the first!</p>
					) : (
						<div className="stack">
							{d.trades.slice(0, 10).map((t) => (
								<div key={t.escrow_id} className="trade-row">
									<div className="trade-row-main">
										<strong>{money(t.amount_sompi)}</strong>
										<span className={`pill pill-${t.status}`}>{t.status}</span>
									</div>
									<div className="trade-row-meta">
										<span className="muted">{relativeTime(t.created_at)}</span>
										<ExplorerEscrowLink escrowId={t.escrow_id} />
									</div>
								</div>
							))}
						</div>
					)}
				</div>
			</div>
		</>
	);
}
