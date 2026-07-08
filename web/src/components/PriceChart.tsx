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
	const values = points.map((p) => p.price_usd);
	const max = values.length > 0 ? Math.max(...values) : 0.0001;
	const min = values.length > 0 ? Math.min(...values) : 0;
	const range = max - min || 1;

	const width = 200;
	const height = 80;

	let pathD = "";
	let markersD = "";
	points.forEach((p, i) => {
		const x = (i / (points.length - 1)) * width;
		const y = height - ((p.price_usd - min) / range) * height;
		const xS = x.toFixed(1);
		const yS = y.toFixed(1);
		pathD += `${i === 0 ? "M" : "L"}${xS},${yS}`;
		// Circle markers as paths (not <circle>) to avoid oval stretching
		const r = 1.5;
		markersD += `M${(x - r).toFixed(1)},${yS} A${r},${r} 0 1,0 ${(x + r).toFixed(1)},${yS} `;
	});

	// Y-axis labels
	const yLabels = [
		{ value: min, label: `$${min.toFixed(4)}` },
		{ value: (min + max) / 2, label: `$${((min + max) / 2).toFixed(4)}` },
		{ value: max, label: `$${max.toFixed(4)}` },
	];

	return (
		<div>
			<div style={{ display: "flex", width: "100%" }}>
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						justifyContent: "space-between",
						padding: "0 6px 0 0",
						fontSize: "9px",
						color: "#888",
						textAlign: "right",
						minWidth: "42px",
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
						{/* Circle markers rendered as paths to avoid oval stretching */}
						<path d={markersD} fill="var(--color-primary)" stroke="none" />
					</svg>
				</div>
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
