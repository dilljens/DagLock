import { useQuery } from "@tanstack/react-query";
import { Helmet } from "react-helmet-async";
import { api, type DailyStat, type LiveSummary } from "../api";
import { moneyCompact } from "../helpers";
import { PriceChart } from "../components/PriceChart";

function LineChart({
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

	const n = padded.length;
	const values = padded.map(getValue);
	const max = Math.max(...values, 1);
	const labelEvery = Math.max(1, Math.floor(n / 7));

	// SVG dimensions — use a wide viewBox matching chart ratio so circles stay circular
	const svgW = 200;
	const svgH = 50;
	const padL = 2;
	const padR = 2;
	const padT = 2;
	const padB = 2;
	const plotW = svgW - padL - padR;
	const plotH = svgH - padT - padB;

	// Build line path only — no circle markers (they stretched into ovals in the SVG)
	let pathD = "";
	values.forEach((v, i) => {
		const x = padL + (i / Math.max(n - 1, 1)) * plotW;
		const y = padT + (1 - v / max) * plotH;
		const xS = x.toFixed(1);
		const yS = y.toFixed(1);
		pathD += `${i === 0 ? "M" : "L"}${xS},${yS}`;
	});

	// Y-axis labels: show min, mid, and max values
	const yLabels = [
		{ value: 0, label: "0" },
		{ value: Math.round(max / 2), label: Math.round(max / 2).toLocaleString() },
		{ value: max, label: max.toLocaleString() },
	];

	return (
		<div style={{ display: "flex", width: "100%" }}>
			{/* Y-axis labels */}
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					justifyContent: "space-between",
					padding: "0 6px 0 0",
					fontSize: "9px",
					color: "#888",
					textAlign: "right",
					minWidth: "36px",
					height: `${height}px`,
					paddingTop: "2px",
					paddingBottom: "2px",
					boxSizing: "border-box",
				}}
			>
				<span>{yLabels[2].label}</span>
				<span>{yLabels[1].label}</span>
				<span>{yLabels[0].label}</span>
			</div>
			<div style={{ position: "relative", width: "100%" }}>
				<div style={{ position: "relative", width: "100%", height: `${height}px` }}>
					<svg
						viewBox={`0 0 ${svgW} ${svgH}`}
						style={{ width: "100%", height: "100%" }}
					>
						{/* Area fill under line */}
						<path
							d={`${pathD} L${padL + plotW},${svgH - padB} L${padL},${svgH - padB} Z`}
							fill={color}
							fillOpacity="0.12"
						/>
						{/* Line */}
						<path
							d={pathD}
							fill="none"
							stroke={color}
							strokeWidth="2"
							vectorEffect="non-scaling-stroke"
						/>
						{/* No circle markers — removed to avoid oval stretching in the SVG viewBox */}
					</svg>
				</div>
			</div>
			{/* Date labels */}
			<div
				style={{
					display: "flex",
					justifyContent: "space-between",
					padding: "0 4px",
					marginTop: "2px",
				}}
			>
				{padded
					.filter((_, i) => i % labelEvery === 0)
					.map((d) => (
						<span
							key={d.date}
							style={{ fontSize: "9px", color: "#888", whiteSpace: "nowrap" }}
						>
							{getLabel(d)}
						</span>
					))}
			</div>
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
						<h2>Escrows per Day (Last 30 days)</h2>
						<div style={{ display: "flex", gap: "24px", marginTop: "12px" }}>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Created</div>
								<LineChart
									data={stats}
									getValue={(d) => d.escrows_created}
									getLabel={(d) => formatDate(d.date)}
									color="var(--color-primary)"
								/>
							</div>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Settled</div>
								<LineChart
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
							<LineChart
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
								<LineChart
									data={stats}
									getValue={(d) => d.active_escrows}
									getLabel={(d) => formatDate(d.date)}
									color="#ff9800"
								/>
							</div>
							<div style={{ flex: 1 }}>
								<div className="stat-card-label">Open Offers</div>
								<LineChart
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
							<LineChart
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
 

