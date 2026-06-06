import { useState } from "react";
import { api, type Reputation, type Receipt } from "../api";
import { money, time, type LoadState } from "../helpers";
import { Panel, LookupResult } from "../ui";

/* ─── Reputation Lookup ─── */
export function ReputationLookup() {
	const [address, setAddress] = useState("");
	const [state, setState] = useState<LoadState<Reputation>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address) return;
		setState({ loading: true });
		try {
			setState({ data: await api.reputation(address.trim()), loading: false });
		} catch (err) {
			setState({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<Panel title="Reputation">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="kaspa address"
				/>
				<button className="button" type="submit">
					Check
				</button>
			</form>
			<LookupResult
				loading={state.loading}
				error={state.error}
				data={state.data}
				render={(data) => (
					<div className="stack">
						<div className="row">
							<span>Score</span>
							<strong>{data.score.toFixed(2)}/5</strong>
						</div>
						<div className="row">
							<span>Trades</span>
							<strong>
								{data.trade_count} ({data.recent_trade_count} in last 90d)
							</strong>
						</div>
						<div className="row">
							<span>Volume</span>
							<strong>{money(data.total_volume_sompi)}</strong>
						</div>
						<div className="row">
							<span>Refund rate</span>
							<strong>{(data.refund_rate * 100).toFixed(1)}%</strong>
						</div>
						<div className="row">
							<span>Dispute rate</span>
							<strong>{(data.dispute_rate * 100).toFixed(1)}%</strong>
						</div>
						<div className="row">
							<span>Age</span>
							<strong>{data.age_days} days</strong>
						</div>
						{data.telegram_handle && (
							<div className="row">
								<span>Telegram</span>
								<strong>{data.telegram_handle}</strong>
							</div>
						)}
						<div className="row">
							<span>Vouches</span>
							<strong>
								{data.vouches_received} received / {data.vouches_given} given
							</strong>
						</div>
						{data.vouch_score != null && (
							<div className="row">
								<span>Vouch score</span>
								<strong>{data.vouch_score.toFixed(2)}/5</strong>
							</div>
						)}
						{data.trading_concentration > 0.9 && (
							<div className="row">
								<span>Wash trading</span>
								<strong className="error-text">
									Warning: {(data.trading_concentration * 100).toFixed(0)}% volume with one
									counterparty
								</strong>
							</div>
						)}
						{data.mediator_stats && (
							<div className="row">
								<span>Mediator</span>
								<strong>
									{data.mediator_stats.score.toFixed(2)}/5 ({data.mediator_stats.disputes_mediated}{" "}
									cases)
								</strong>
							</div>
						)}
					</div>
				)}
			/>
		</Panel>
	);
}

/* ─── Receipt Lookup ─── */
export function ReceiptLookup() {
	const [id, setId] = useState("");
	const [state, setState] = useState<LoadState<Receipt>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!id) return;
		setState({ loading: true });
		try {
			setState({ data: await api.receipt(id.trim()), loading: false });
		} catch (err) {
			setState({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<Panel title="Receipt lookup">
			<form className="form" onSubmit={handleSubmit}>
				<input value={id} onChange={(e) => setId(e.target.value)} placeholder="escrow id" />
				<button className="button" type="submit">
					Fetch
				</button>
			</form>
			<LookupResult
				loading={state.loading}
				error={state.error}
				data={state.data}
				render={(data) => (
					<div className="stack">
						<div className="row">
							<span>ID</span>
							<strong>{data.receipt_id}</strong>
						</div>
						<div className="row">
							<span>Status</span>
							<strong>{data.status}</strong>
						</div>
						<div className="row">
							<span>Amount</span>
							<strong>{money(data.amount_sompi)}</strong>
						</div>
					</div>
				)}
			/>
		</Panel>
	);
}
