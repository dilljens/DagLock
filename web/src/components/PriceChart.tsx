import { useQuery } from "@tanstack/react-query";
import { api, type PriceHistoryPoint } from "../api";

function formatDateShort(ts: number): string {
	const d = new Date(ts * 1000);
	return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

export function PriceChart({ days = 30 }: { days?: number }) {
	const { data, isLoading, error } = useQuery({
		queryKey: ["price-history", days],
		queryFn: () => api.getPriceHistory(days),
		staleTime: 60_000,
	});

	if (isLoading) {
		return (
			<div
				style={{
					height: "160px",
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					color: "#888",
					fontSize: "13px",
				}}
			>
				Loading price chart…
			</div>
		);
	}

	if (error || !data?.points?.length) {
		return (
			<p className="muted" style={{ fontSize: "13px", margin: "8px 0" }}>
				Price history not available yet.
			</p>
		);
	}

	const points = data.points;
	const max = Math.max(...points.map((p) => p.price_usd), 0.0001);
	const min = Math.min(...points.map((p) => p.price_usd), 0);
	const range = max - min || 1;

	const width = 100;
	const height = 160;

	const pathD = points
		.map((p, i) => {
			const x = (i / (points.length - 1)) * width;
			const y = height - ((p.price_usd - min) / range) * height;
			return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
		})
		.join(" ");

	return (
		<div>
			<div style={{ position: "relative", width: "100%", height: `${height}px` }}>
				<svg
					viewBox={`0 0 ${width} ${height}`}
					style={{ width: "100%", height: "100%" }}
					preserveAspectRatio="none"
				>
					<defs>
						<linearGradient id="price-grad" x1="0" y1="0" x2="0" y2="1">
							<stop offset="0%" stopColor="var(--color-primary)" stopOpacity="0.3" />
							<stop offset="100%" stopColor="var(--color-primary)" stopOpacity="0" />
						</linearGradient>
					</defs>
					{/* Area fill */}
					<path d={`${pathD} L${width},${height} L0,${height} Z`} fill="url(#price-grad)" />
					{/* Line */}
					<path
						d={pathD}
						fill="none"
						stroke="var(--color-primary)"
						strokeWidth="2"
						vectorEffect="non-scaling-stroke"
					/>
				</svg>
			</div>
			<div
				style={{
					display: "flex",
					justifyContent: "space-between",
					marginTop: "4px",
					fontSize: "11px",
					color: "#888",
				}}
			>
				<span>${min.toFixed(4)}</span>
				<span>${max.toFixed(4)}</span>
			</div>
			<div
				style={{
					display: "flex",
					justifyContent: "space-between",
					fontSize: "10px",
					color: "#666",
				}}
			>
				<span>{formatDateShort(points[0].timestamp)}</span>
				<span>{formatDateShort(points[points.length - 1].timestamp)}</span>
			</div>
		</div>
	);
}
