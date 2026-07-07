import { useState, useEffect } from "react";
import { api } from "../api";
import { money } from "../helpers";

/**
 * Fee calculator component.
 * Shows 0.5% protocol fee for any KAS amount with optional USD estimate.
 * No API call needed for fee math — it's a constant 1/200.
 */
export function FeeCalculator() {
	const [amountKas, setAmountKas] = useState("");
	const [price, setPrice] = useState<number | null>(null);

	useEffect(() => {
		api
			.networkPrice()
			.then((d) => setPrice(d.kas_usd))
			.catch(() => {
				/* price fetch is optional */
			});
	}, []);

	const amount = Number.parseFloat(amountKas) || 0;
	const fee = amount / 200;
	const net = amount - fee;
	const feeSompi = Math.round(fee * 100_000_000);
	const netSompi = Math.round(net * 100_000_000);

	return (
		<div className="panel fee-calculator">
			<h3 style={{ margin: "0 0 8px" }}>Fee Calculator</h3>
			<p className="muted" style={{ fontSize: "13px", margin: "0 0 12px" }}>
				DagLock charges a <strong>0.5% protocol fee</strong> (1/200) on settlement, enforced by the
				covenant. No hidden fees.
			</p>
			<div className="form form-stacked">
				<div className="form-field">
					<label>Amount (KAS)</label>
					<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
						<input
							type="number"
							step="any"
							min="0"
							value={amountKas}
							onChange={(e) => setAmountKas(e.target.value)}
							placeholder="e.g. 1000"
							style={{ flex: 1 }}
						/>
						{price != null && amount > 0 && (
							<span className="muted" style={{ fontSize: "13px", whiteSpace: "nowrap" }}>
								~${(amount * price).toLocaleString()}
							</span>
						)}
					</div>
				</div>
			</div>

			{amount > 0 && (
				<div className="fee-breakdown" style={{ marginTop: "12px" }}>
					<div className="fee-row">
						<span>Protocol fee (0.5%)</span>
						<strong style={{ color: "var(--color-warning, #ff9800)" }}>{money(feeSompi)}</strong>
					</div>
					<div className="fee-row">
						<span>Net to seller</span>
						<strong>{money(netSompi)}</strong>
					</div>
					<div className="fee-row">
						<span>Treasury receives</span>
						<strong>{money(feeSompi)}</strong>
					</div>
					{price != null && (
						<div
							className="fee-row"
							style={{ borderTop: "1px solid #333", paddingTop: "8px", marginTop: "8px" }}
						>
							<span>USD value</span>
							<strong>${(amount * price).toLocaleString()}</strong>
						</div>
					)}
				</div>
			)}
		</div>
	);
}
