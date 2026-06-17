import { useState } from "react";
import { api, type Reputation } from "../api";
import { money, type LoadState } from "../helpers";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonStats } from "../ui";
import { EmptyState } from "../components/empty-state";

type Tab = "lookup" | "my-reputation" | "vouch" | "identity";

export function ReputationPage() {
	const [tab, setTab] = useState<Tab>("my-reputation");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<div>
			<div className="page-header">
				<h1><h1> Reputation</h1></h1>
				<p>On-chain trading history, vouches, and identity verification.</p>
			</div>
			<div className="tab-bar">
				{wallet.connected && (
					<button className={`tab-btn ${tab === "my-reputation" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("my-reputation")}>My Reputation</button>
				)}
				<button className={`tab-btn ${tab === "lookup" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("lookup")}>Lookup</button>
				{wallet.connected && (
					<>
						<button className={`tab-btn ${tab === "vouch" ? "tab-btn--active" : ""}`}
							onClick={() => setTab("vouch")}>Vouch</button>
						<button className={`tab-btn ${tab === "identity" ? "tab-btn--active" : ""}`}
							onClick={() => setTab("identity")}>Link Telegram</button>
					</>
				)}
			</div>
			{tab === "my-reputation" && address && <ReputationDisplay address={address} />}
			{tab === "lookup" && <ReputationLookup />}
			{tab === "vouch" && (wallet.connected ? <VouchSection /> : <ConnectPrompt />)}
			{tab === "identity" && (wallet.connected ? <IdentitySection /> : <ConnectPrompt />)}
		</div>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<EmptyState
			icon="👛"
			title="Connect your wallet"
			description="Connect KasWare to manage reputation and vouches."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

/* ─── Reputation Display ─── */
function ReputationDisplay({ address }: { address: string }) {
	const [state, setState] = useState<LoadState<Reputation>>({ loading: true });

	if (state.loading) {
		api.reputation(address)
			.then((d) => setState({ data: d, loading: false }))
			.catch((e) => setState({ error: e.message, loading: false }));
	}

	if (state.loading) return <SkeletonStats />;
	if (state.error) return <p className="muted error-text">{state.error}</p>;
	if (!state.data) return <p className="muted">No data</p>;

	const d = state.data;
	return (
		<div>
			<div className="stats-grid">
				<div className="stat-card">
					<div className="stat-card-label">Score</div>
					<div className="stat-card-value">{d.score.toFixed(2)}/5</div>
				</div>
				<div className="stat-card">
					<div className="stat-card-label">Trades</div>
					<div className="stat-card-value">{d.trade_count}</div>
				</div>
				<div className="stat-card">
					<div className="stat-card-label">Volume</div>
					<div className="stat-card-value">{money(d.total_volume_sompi)}</div>
				</div>
				<div className="stat-card">
					<div className="stat-card-label">Account Age</div>
					<div className="stat-card-value">{d.age_days}d</div>
				</div>
			</div>
			<div className="panel">
				<div className="stack">
					<div className="row"><span>Recent trades (90d)</span><strong>{d.recent_trade_count}</strong></div>
					<div className="row"><span>Settled</span><strong>{d.settled_count}</strong></div>
					<div className="row"><span>Refunded</span><strong>{d.refunded_count}</strong></div>
					<div className="row"><span>Disputed</span><strong>{d.disputed_count}</strong></div>
					<div className="row"><span>Refund rate</span><strong>{(d.refund_rate * 100).toFixed(1)}%</strong></div>
					<div className="row"><span>Dispute rate</span><strong>{(d.dispute_rate * 100).toFixed(1)}%</strong></div>
					<div className="row"><span>Vouches received</span><strong>{d.vouches_received}</strong></div>
					<div className="row"><span>Vouches given</span><strong>{d.vouches_given}</strong></div>
					{d.vouch_score != null && (
						<div className="row"><span>Vouch score</span><strong>{d.vouch_score.toFixed(2)}/5</strong></div>
					)}
					{d.telegram_handle && (
						<div className="row"><span>Telegram</span><strong>{d.telegram_handle}</strong></div>
					)}
					{d.trading_concentration > 0.9 && (
						<div className="row">
							<span> Wash trading signal</span>
							<strong style={{ color: "#ff7b7b" }}>{(d.trading_concentration * 100).toFixed(0)}% with one counterparty</strong>
						</div>
					)}
					{d.mediator_stats && (
						<div className="row">
							<span>Mediator score</span>
							<strong>{d.mediator_stats.score.toFixed(2)}/5 ({d.mediator_stats.disputes_mediated} cases)</strong>
						</div>
					)}
				</div>
			</div>
		</div>
	);
}

/* ─── Lookup (any address) ─── */
function ReputationLookup() {
	const [addr, setAddr] = useState("");
	const [data, setData] = useState<LoadState<Reputation>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!addr.trim()) return;
		setData({ loading: true });
		try {
			setData({ data: await api.reputation(addr.trim()), loading: false });
		} catch (err) {
			setData({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<div>
			<form className="form" onSubmit={handleSubmit} style={{ marginBottom: "16px" }}>
				<input value={addr} onChange={(e) => setAddr(e.target.value)}
					placeholder="kaspa address" />
				<button className="button primary" type="submit" disabled={data.loading}>
					{data.loading ? "Loading…" : "Check"}
				</button>
			</form>
			{data.error && <p className="muted error-text">{data.error}</p>}
			{data.data && <ReputationDisplay address={addr} />}
		</div>
	);
}

/* ─── Vouch ─── */
function VouchSection() {
	const address = useAddress()!;
	const { sign } = useWallet();
	const { notify } = useToast();
	const [subject, setSubject] = useState("");
	const [note, setNote] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!subject.startsWith("kaspa:")) return;
		setStatus("loading");
		try {
			const auth = { address, signature: await sign(`vouch:${subject}`), message: `vouch:${subject}` };
			await api.vouch(subject, auth, undefined, note || undefined);
			setStatus("done");
			notify("success", "Vouch created!");
		} catch (e) {
			notify("error", "Failed to vouch", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done") return (
		<EmptyState
			icon="✅"
			title="Vouch created!"
			description={`You vouched for ${subject.slice(0, 24)}…`}
		/>
	);

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted" style={{ fontSize: "13px" }}>
				Vouch for someone's reliability. Your own reputation score is attached.
			</p>
			<FormField label="Subject address">
				<input value={subject} onChange={(e) => setSubject(e.target.value)}
					placeholder="kaspa:..." />
			</FormField>
			<FormField label="Note (optional)">
				<input value={note} onChange={(e) => setNote(e.target.value)}
					placeholder="Why do you vouch for them?" />
			</FormField>
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create Vouch"}
			</button>
		</form>
	);
}

/* ─── Identity / Link Telegram ─── */
function IdentitySection() {
	const address = useAddress()!;
	const { sign } = useWallet();
	const { notify } = useToast();
	const [handle, setHandle] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!handle.trim()) return;
		setStatus("loading");
		try {
			const msg = `daglock.io:verify:telegram:${handle.trim()}`;
			const sig = await sign(msg);
			await api.createIdentity("telegram", handle.trim(), msg, sig, {
				address, signature: sig, message: msg,
			});
			setStatus("done");
			notify("success", "Telegram linked!");
		} catch (e) {
			notify("error", "Failed to link", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done") return (
		<EmptyState
			icon="✅"
			title="Telegram linked!"
			description={`@${handle} is now associated with your address.`}
		/>
	);

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted" style={{ fontSize: "13px" }}>
				Link your Telegram handle to your Kaspa address. This shows in your reputation profile.
			</p>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				Address: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>
			<FormField label="Telegram handle">
				<input value={handle} onChange={(e) => setHandle(e.target.value)}
					placeholder="yourusername (without @)" />
			</FormField>
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Linking…" : "Link Telegram"}
			</button>
		</form>
	);
}
