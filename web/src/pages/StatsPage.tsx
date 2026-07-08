import { useQuery } from "@tanstack/react-query";
import { Helmet } from "react-helmet-async";
import { api, type DailyStat, type LiveSummary } from "../api";
import { moneyCompact } from "../helpers";
import { PriceChart } from "../components/PriceChart";

function BarChart({
	data,
	getValue,
	getLabel,
	color,
	height = 120,
	minDays = 7,
}: {
	data: DailyStat[];
	getValue: (d: DailyStat) => number;
	getLabel: (d: DailyStat) => string;
	color: string;
	height?: number;
	minDays?: number;
}) {
	// Pad data to at least minDays by prepending zero-value days
	const padded = [...data];
	if (padded.length > 0) {
		padded.sort((a, b) => a.date.localeCompare(b.date));
		const earliest = new Date(padded[0].date + "T00:00:00");
		while (padded.length < minDays) {
			earliest.setDate(earliest.getDate() - 1);
			const dateStr = earliest.toISOString().split("T")[0];
			padded.unshift({
				date: dateStr,
				escrows_created: 0,
				escrows_settled: 0,
				volume_sompi: 0,
				fees_sompi: 0,
				active_escrows: 0,
				open_offers: 0,
				total_users: 0,
				kas_usd_price: null,
			});
		}
	}

	const values = padded.map(getValue);
	const max = Math.max(...values, 1);
	// Prefer showing bars that are at least 3px tall for visibility.
	// Scale: the tallest bar is 100%, smaller bars are proportional,
	// but we use a minimum bar height of 4px so single-day data doesn't dominate.
	const barMinPx = 4;
	const scaleMax = max <= 1 ? max : max;
	const labelEvery = Math.max(1, Math.floor(padded.length / 7));

	return (
		<div
			style={{
				display: "flex",
				alignItems: "flex-end",
				gap: "3px",
				height: `${height}px`,
				padding: "0 4px",
			}}
		>
			{padded.map((d, i) => {
				const v = getValue(d);
				const pct = max > 0 ? (v / scaleMax) * 100 : 0;
				const barHeight = v > 0 ? Math.max(pct, (barMinPx / height) * 100) : 0;
				return (
					<div
						key={d.date}
						title={`${d.date}: ${v.toLocaleString()}`}
						style={{
							flex: "1",
							height: `${barHeight}%`,
							background: color,
							borderRadius: "3px 3px 0 0",
							minWidth: "6px",
							position: "relative",
							transition: "height 0.3s ease",
							opacity: v > 0 ? 1 : 0.3,
						}}
					>
						{i % labelEvery === 0 && (
							<span
								style={{
									position: "absolute",
									bottom: "-18px",
									left: "50%",
									transform: "translateX(-50%)",
									fontSize: "9px",
									color: "#888",
									whiteSpace: "nowrap",
								}}
							>
								{getLabel(d)}
							</span>
						)}
					</div>
				);
			})}
		</div>
	);
}

function formatDate(dateStr: string): string {
	const d = new Date(dateStr + "T00:00:00");
	return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

function Card({ label, value, sub }: { label: string; value: string; sub?: string }) {
	return (
		<div className="stat-card">
			<div className="stat-card-label">{label}</div>
			<div className="stat-card-value">{value}</div>
			{sub && <div className="stat-card-sub">{sub}</div>}
		</div>
	);
}

export function StatsPage() {
	const { data: dailyData, isLoading: dailyLoading } = useQuery({
		queryKey: ["daily-stats"],
		queryFn: () => api.getDailyStats(30),
		staleTime: 60_000,
	});

	const { data: summary, isLoading: summaryLoading } = useQuery({
		queryKey: ["live-summary"],
		queryFn: () => api.getLiveSummary(),
		staleTime: 30_000,
	});

	const stats = dailyData?.stats ?? [];
	const live = summary;
	const loading = dailyLoading || summaryLoading;

	if (loading) {
		return (
			<div>
				<Helmet>
					<title>Analytics — DagLock</title>
				</Helmet>
				<div className="page-header">
					<h1>Analytics</h1>
					<p>Loading stats…</p>
				</div>
				<div className="loading" style={{ padding: "3rem", textAlign: "center", color: "#888" }}>
					Loading…
				</div>
			</div>
		);
	}

	const sompiToKas = (v: number) =>
		(v / 100_000_000).toLocaleString(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		});

	return (
		<div>
			<Helmet>
				<title>Analytics — DagLock</title>
				<meta
					name="description"
					content="DagLock platform analytics: escrow volume, fees, user growth, and network health."
				/>
			</Helmet>

			<div className="page-header">
				<h1>Analytics</h1>
				<p>Platform-wide statistics and growth trends</p>
			</div>

			{/* Live summary hero */}
			{live && (
				<div className="stats-grid">
					<Card
						label="Total Escrows"
						value={live.total_escrows.toLocaleString()}
						sub={`${live.active_escrows.toLocaleString()} active`}
					/>
					<Card
						label="Total Volume"
						value={moneyCompact(live.total_volume_sompi)}
						sub={`${sompiToKas(live.total_volume_sompi)} KAS`}
					/>
					<Card
						label="Total Fees"
						value={moneyCompact(live.total_fees_sompi)}
						sub={`${sompiToKas(live.total_fees_sompi)} KAS`}
					/>
					<Card label="Unique Users" value={live.total_users.toLocaleString()} />
					<Card label="Open Offers" value={live.open_offers.toLocaleString()} />
					<Card
						label="Uptime"
						value={(() => {
							const d = Math.floor(live.uptime_seconds / 86400);
							const h = Math.floor((live.uptime_seconds % 86400) / 3600);
							return d > 0 ? `${d}d ${h}h` : `${h}h`;
						})()}
					/>
				</div>
			)}

			{/* KAS/USD Price Chart */}
			<section style={{ marginTop: "32px" }}>
				<h2>KAS/USD Price (30d)</h2>
				<div style={{ marginTop: "12px" }}>
					<PriceChart days={30} />
				</div>
			</section>

			{/* Daily charts */}
			{stats.length > 0 && (
				<>
					{/* Escrows created / settled */}
					<section style={{ marginTop: "32px" }}>
						<h2>Escrows per Day (Last 30d)</h2>
						<div style={{ display: "flex", gap: "24px", marginTop: "12px" }}>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Created</div>
								<BarChart
									data={stats}
									getValue={(d) => d.escrows_created}
									getLabel={(d) => formatDate(d.date)}
									color="var(--color-primary)"
								/>
							</div>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Settled</div>
								<BarChart
									data={stats}
									getValue={(d) => d.escrows_settled}
									getLabel={(d) => formatDate(d.date)}
									color="#4caf50"
								/>
							</div>
						</div>
					</section>

					{/* Volume per day */}
					<section style={{ marginTop: "48px" }}>
						<h2>Daily Volume (KAS)</h2>
						{stats[0]?.kas_usd_price != null && (
							<p className="muted" style={{ marginTop: 0 }}>
								KAS/USD: ${stats[0].kas_usd_price.toFixed(4)}
							</p>
						)}
						<div style={{ marginTop: "12px" }}>
							<BarChart
								data={stats}
								getValue={(d) => Math.round(d.volume_sompi / 100_000_000)}
								getLabel={(d) => formatDate(d.date)}
								color="#2196f3"
								height={160}
							/>
						</div>
					</section>

					{/* Active escrows / open offers */}
					<section style={{ marginTop: "48px" }}>
						<h2>Active Escrows vs Open Offers</h2>
						<div style={{ display: "flex", gap: "24px", marginTop: "12px" }}>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Active Escrows</div>
								<BarChart
									data={stats}
									getValue={(d) => d.active_escrows}
									getLabel={(d) => formatDate(d.date)}
									color="#ff9800"
								/>
							</div>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Open Offers</div>
								<BarChart
									data={stats}
									getValue={(d) => d.open_offers}
									getLabel={(d) => formatDate(d.date)}
									color="#9c27b0"
								/>
							</div>
						</div>
					</section>

					{/* Users */}
					<section style={{ marginTop: "48px" }}>
						<h2>Total Users</h2>
						<div style={{ marginTop: "12px" }}>
							<BarChart
								data={stats}
								getValue={(d) => d.total_users}
								getLabel={(d) => formatDate(d.date)}
								color="#00bcd4"
								height={100}
							/>
						</div>
					</section>
				</>
			)}

			{!stats.length && !loading && (
				<div className="empty-state" style={{ marginTop: "32px" }}>
					<p className="muted" style={{ textAlign: "center" }}>
						No daily stats yet. Stats will appear after the first background computation run.
					</p>
				</div>
			)}
		</div>
	);
}
