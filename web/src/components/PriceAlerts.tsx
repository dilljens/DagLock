import { useState, useEffect } from "react";
import { api, type PriceAlert } from "../api";
import { useToast } from "../layout/Toast";

export function PriceAlertsSettings({ address }: { address: string }) {
	const { notify } = useToast();
	const [alerts, setAlerts] = useState<PriceAlert[]>([]);
	const [loading, setLoading] = useState(false);
	const [targetPrice, setTargetPrice] = useState("");
	const [direction, setDirection] = useState<"above" | "below">("above");

	useEffect(() => {
		if (!address) return;
		setLoading(true);
		api
			.listPriceAlerts(address)
			.then((d) => setAlerts(d.alerts))
			.catch(() => {})
			.finally(() => setLoading(false));
	}, [address]);

	async function handleCreate() {
		const price = Number.parseFloat(targetPrice);
		if (!price || price <= 0) return;
		try {
			await api.createPriceAlert({ address, target_price: price, direction });
			notify("success", `Price alert created: KAS ${direction} $${price}`);
			setTargetPrice("");
			const d = await api.listPriceAlerts(address);
			setAlerts(d.alerts);
		} catch (e) {
			notify("error", "Failed to create alert", (e as Error).message);
		}
	}

	async function handleDelete(id: string) {
		try {
			await api.deletePriceAlert(id);
			setAlerts((prev) => prev.filter((a) => a.id !== id));
			notify("success", "Alert deleted");
		} catch (e) {
			notify("error", "Failed to delete", (e as Error).message);
		}
	}

	return (
		<div className="panel" style={{ marginBottom: "16px" }}>
			<h3 style={{ margin: "0 0 4px" }}>Price Alerts</h3>
			<p className="muted" style={{ margin: "0 0 12px", fontSize: "13px" }}>
				Get notified when KAS reaches your target price.
			</p>

			<div style={{ display: "flex", gap: "8px", marginBottom: "12px", alignItems: "flex-end" }}>
				<select
					value={direction}
					onChange={(e) => setDirection(e.target.value as "above" | "below")}
					style={{ width: "100px", padding: "8px" }}
				>
					<option value="above">Above</option>
					<option value="below">Below</option>
				</select>
				<input
					type="number"
					step="0.001"
					value={targetPrice}
					onChange={(e) => setTargetPrice(e.target.value)}
					placeholder="0.05"
					style={{ flex: 1 }}
				/>
				<button className="button primary" onClick={handleCreate} disabled={!targetPrice}>
					Create Alert
				</button>
			</div>

			{loading && (
				<p className="muted" style={{ fontSize: "13px" }}>
					Loading alerts…
				</p>
			)}

			{alerts.length === 0 && !loading && (
				<p className="muted" style={{ fontSize: "13px" }}>
					No price alerts yet.
				</p>
			)}

			{alerts.map((alert) => (
				<div
					key={alert.id}
					style={{
						display: "flex",
						alignItems: "center",
						justifyContent: "space-between",
						padding: "8px 0",
						borderBottom: "1px solid var(--color-border)",
					}}
				>
					<div style={{ fontSize: "14px" }}>
						<span
							style={{
								fontWeight: 600,
								color: alert.triggered ? "#888" : "var(--color-text)",
							}}
						>
							{alert.triggered ? "🔔 " : "⏰ "}
							{alert.direction === "above" ? "Above" : "Below"} ${alert.target_price.toFixed(4)}
						</span>
						{alert.triggered && alert.triggered_at && (
							<span className="muted" style={{ marginLeft: "8px", fontSize: "12px" }}>
								Triggered {new Date(alert.triggered_at * 1000).toLocaleDateString()}
							</span>
						)}
					</div>
					{!alert.triggered && (
						<button
							className="button"
							style={{ padding: "2px 8px", fontSize: "11px" }}
							onClick={() => handleDelete(alert.id)}
						>
							Delete
						</button>
					)}
				</div>
			))}
		</div>
	);
}
