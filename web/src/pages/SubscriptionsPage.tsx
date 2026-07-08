import { useState, useEffect, useCallback, useRef } from "react";
import { useForm } from "react-hook-form";
import { api, type Subscription, type CreateSubscriptionRequest, type AuthHeaders } from "../api";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonTable } from "../ui";
import { EmptyState } from "../components/empty-state";
import { money, badge, time } from "../helpers";
import type { LoadState } from "../helpers";
import { Helmet } from "react-helmet-async";

type Tab = "my-subscriptions" | "create" | "lookup";

const INTERVAL_PRESETS: { label: string; seconds: number }[] = [
	{ label: "Daily", seconds: 86400 },
	{ label: "Weekly", seconds: 604800 },
	{ label: "Monthly (30d)", seconds: 2592000 },
	{ label: "Quarterly", seconds: 7776000 },
	{ label: "Yearly", seconds: 31536000 },
];

function formatInterval(seconds: number): string {
	const p = INTERVAL_PRESETS.find((i) => i.seconds === seconds);
	if (p) return p.label;
	if (seconds % 86400 === 0) return `${seconds / 86400} days`;
	return `${seconds}s`;
}

function subscriptionStatusBadge(status: string): string {
	return badge(status);
}

function statusLabel(status: string): string {
	const map: Record<string, string> = {
		active: "Active",
		cancelled: "Cancelled",
		completed: "Completed",
	};
	return map[status] || status;
}

export function SubscriptionsPage() {
	const [tab, setTab] = useState<Tab>("my-subscriptions");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<>
			<Helmet>
				<title>Subscriptions — DagLock</title>
				<meta
					name="description"
					content="Create and manage recurring payment subscriptions on Kaspa. Set up periodic KAS payments with SilverScript covenants."
				/>
				<link rel="canonical" href="https://daglock.com/subscriptions" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1> Subscriptions</h1>
					<p>Recurring KAS payments. Set up periodic installments with covenant-enforced terms.</p>
				</div>

				<details
					className="panel"
					style={{ marginBottom: "16px", padding: "12px 16px", cursor: "pointer" }}
				>
					<summary style={{ fontWeight: 600, fontSize: "14px", color: "var(--color-text)" }}>
						 How Subscriptions Work
					</summary>
					<div
						style={{
							marginTop: "12px",
							fontSize: "13px",
							color: "var(--color-text-secondary)",
							lineHeight: 1.7,
						}}
					>
						<p style={{ margin: "0 0 8px" }}>
							Subscriptions let you set up recurring KAS payments on-chain. The payer locks the
							full <strong>total_amount</strong> upfront in a covenant. The recipient can{" "}
							<strong>draw</strong> each <strong>installment_amount</strong> once per interval.
						</p>
						<p style={{ margin: "0 0 8px" }}>
							<strong>Key rules:</strong>
						</p>
						<ul style={{ margin: "0 0 8px", paddingLeft: "20px" }}>
							<li>The payer can <strong>cancel</strong> anytime — remaining funds are returned.</li>
							<li>The recipient can <strong>draw</strong> once per interval up to <strong>max_periods</strong>.</li>
							<li>Funds are secured by a SilverScript covenant — trustless by construction.</li>
						</ul>
						<p style={{ margin: 0 }}>
							<strong>Fee:</strong> 0.5% protocol fee on each draw, paid to the DagLock treasury.
						</p>
					</div>
				</details>

				<div className="tab-bar">
					<button
						className={`tab-btn ${tab === "my-subscriptions" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("my-subscriptions")}
					>
						My Subscriptions
					</button>
					{wallet.connected && (
						<button
							className={`tab-btn ${tab === "create" ? "tab-btn--active" : ""}`}
							onClick={() => setTab("create")}
						>
							Create
						</button>
					)}
					<button
						className={`tab-btn ${tab === "lookup" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("lookup")}
					>
						Lookup
					</button>
				</div>
				{tab === "my-subscriptions" &&
					(wallet.connected ? <MySubscriptions address={address!} /> : <ConnectPrompt />)}
				{tab === "create" &&
					(wallet.connected ? <CreateSubscription address={address!} /> : <ConnectPrompt />)}
				{tab === "lookup" && <SubscriptionLookup />}
			</div>
		</>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<EmptyState
			icon="👛"
			title="Connect your wallet"
			description="Connect KasWare to manage subscriptions."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

function MySubscriptions({ address }: { address: string }) {
	const [subscriptions, setSubscriptions] = useState<LoadState<Subscription[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const loaded = useRef(false);

	const load = useCallback(() => {
		setSubscriptions({ loading: true });
		api
			.subscriptions(address)
			.then((d) => setSubscriptions({ data: d.subscriptions, loading: false }))
			.catch((e) => setSubscriptions({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => {
		if (loaded.current) return;
		loaded.current = true;
		load();
	}, [load]);

	if (subscriptions.loading) return <SkeletonTable rows={5} />;
	if (subscriptions.error) return <p className="muted error-text">{subscriptions.error}</p>;
	if (!subscriptions.data?.length)
		return (
			<EmptyState
				icon="🔄"
				title="No subscriptions"
				description="Create a subscription to set up recurring KAS payments."
			/>
		);

	return (
		<div>
			{subscriptions.data.map((s) => (
				<article
					key={s.id}
					className="offer"
					style={{ cursor: "pointer", marginBottom: "8px" }}
					onClick={() => setSelectedId(selectedId === s.id ? null : s.id)}
				>
					<div className="offer-top">
						<strong>{money(s.installment_amount)} / {formatInterval(s.interval_seconds)}</strong>
						<span className={subscriptionStatusBadge(s.status)}>{statusLabel(s.status)}</span>
					</div>
					<p>
						{money(s.total_amount)} total · {s.payer_address.slice(0, 16)}… →{" "}
						{s.recipient_address.slice(0, 16)}…
					</p>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							gap: "8px",
							margin: "4px 0",
							fontSize: "12px",
						}}
					>
						<span style={{ color: "#888" }}>
							Progress: {s.current_period} / {s.max_periods}
						</span>
						<div
							style={{
								flex: 1,
								height: "6px",
								background: "#333",
								borderRadius: "3px",
								overflow: "hidden",
								maxWidth: "200px",
							}}
						>
							<div
								style={{
									width: `${Math.min(100, (s.current_period / Math.max(s.max_periods, 1)) * 100)}%`,
									height: "100%",
									background: s.status === "active" ? "#4caf50" : "#888",
									borderRadius: "3px",
									transition: "width 0.3s",
								}}
							/>
						</div>
					</div>
					<code>{s.id}</code>
					{selectedId === s.id && (
						<SubscriptionActions subscription={s} onMutated={load} currentAddress={address} />
					)}
				</article>
			))}
		</div>
	);
}

function SubscriptionActions({
	subscription,
	onMutated,
	currentAddress,
}: {
	subscription: Subscription;
	onMutated: () => void;
	currentAddress: string;
}) {
	const { notify } = useToast();
	const [loading, setLoading] = useState("");

	const isFinal = subscription.status !== "active";
	const isPayer = subscription.payer_address === currentAddress;
	const isRecipient = subscription.recipient_address === currentAddress;

	async function doCancel() {
		setLoading("cancel");
		try {
			await api.cancelSubscription(subscription.id);
			notify("success", "Subscription cancelled");
			onMutated();
		} catch (e) {
			notify("error", "Failed to cancel", (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	async function doDraw() {
		setLoading("draw");
		try {
			const result = await api.drawSubscription(subscription.id);
			notify(
				"success",
				`Drawn! Period ${result.current_period}/${result.max_periods}`,
			);
			onMutated();
		} catch (e) {
			notify("error", "Failed to draw", (e as Error).message);
		} finally {
			setLoading("");
		}
	}

	if (isFinal) return <p className="muted"> Finalized — {statusLabel(subscription.status)}</p>;

	return (
		<div className="offer-actions" style={{ marginTop: "12px" }}>
			{isRecipient && (
				<button
					className="button primary"
					disabled={!!loading}
					onClick={doDraw}
					style={{ marginRight: "8px" }}
				>
					{loading === "draw" ? "Drawing…" : " Draw"}
				</button>
			)}
			{isPayer && (
				<button
					className="button"
					disabled={!!loading}
					onClick={doCancel}
				>
					{loading === "cancel" ? "Cancelling…" : " Cancel"}
				</button>
			)}
		</div>
	);
}

function CreateSubscription({ address }: { address: string }) {
	const { sign, state } = useWallet();
	const { notify } = useToast();
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [result, setResult] = useState<Subscription | null>(null);
	const [intervalSecs, setIntervalSecs] = useState(604800);
	const [totalAmount, setTotalAmount] = useState("");
	const [installmentAmount, setInstallmentAmount] = useState("");
	const [maxPeriods, setMaxPeriods] = useState("12");
	const [recipientAddress, setRecipientAddress] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const totalNum = Number.parseFloat(totalAmount);
		const installmentNum = Number.parseFloat(installmentAmount);
		const maxPeriodsNum = Number.parseInt(maxPeriods, 10);
		if (!totalNum || totalNum <= 0) return;
		if (!installmentNum || installmentNum <= 0) return;
		if (!maxPeriodsNum || maxPeriodsNum < 1) return;
		if (!recipientAddress.startsWith("kaspa:")) return;

		if (installmentNum > totalNum) {
			notify("error", "Installment cannot exceed total amount");
			return;
		}

		setStatus("loading");
		try {
			const sompiTotal = Math.round(totalNum * 100_000_000);
			const sompiInstallment = Math.round(installmentNum * 100_000_000);

			const auth: AuthHeaders = {
				address,
				signature: await sign(`subscribe:${recipientAddress}:${sompiInstallment}`),
				message: `subscribe:${recipientAddress}:${sompiInstallment}`,
			};

			const sub = await api.createSubscription(
				{
					payer_address: address,
					recipient_address: recipientAddress,
					total_amount: sompiTotal,
					installment_amount: sompiInstallment,
					interval_seconds: intervalSecs,
					max_periods: maxPeriodsNum,
					start_time: Math.floor(Date.now() / 1000),
				},
				auth,
			);
			setResult(sub);
			setStatus("done");
			notify("success", "Subscription created!");
		} catch (e) {
			notify("error", "Failed to create subscription", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done" && result) {
		return (
			<EmptyState
				icon="✅"
				title="Subscription created!"
				description={`ID: ${result.id} · ${money(result.installment_amount)} / ${formatInterval(result.interval_seconds)} · ${result.max_periods} periods`}
			/>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				You (payer): <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>

			<FormField label="Recipient address">
				<input
					value={recipientAddress}
					onChange={(e) => setRecipientAddress(e.target.value)}
					placeholder="kaspa:..."
					required
				/>
			</FormField>

			<div style={{ display: "flex", gap: "12px" }}>
				<div style={{ flex: 1 }}>
					<FormField label="Total amount (KAS)">
						<input
							type="number"
							step="any"
							value={totalAmount}
							onChange={(e) => setTotalAmount(e.target.value)}
							placeholder="1000"
							required
						/>
					</FormField>
				</div>
				<div style={{ flex: 1 }}>
					<FormField label="Installment (KAS)">
						<input
							type="number"
							step="any"
							value={installmentAmount}
							onChange={(e) => setInstallmentAmount(e.target.value)}
							placeholder="100"
							required
						/>
					</FormField>
				</div>
			</div>

			<FormField label="Interval">
				<select
					value={intervalSecs}
					onChange={(e) => setIntervalSecs(Number(e.target.value))}
				>
					{INTERVAL_PRESETS.map((p) => (
						<option key={p.seconds} value={p.seconds}>
							{p.label}
						</option>
					))}
				</select>
			</FormField>

			<FormField label="Max periods">
				<input
					type="number"
					min={1}
					max={365}
					value={maxPeriods}
					onChange={(e) => setMaxPeriods(e.target.value)}
					placeholder="12"
					required
				/>
				<span style={{ fontSize: "11px", color: "#888", marginTop: "4px", display: "block" }}>
					Number of installments (e.g. 12 for monthly over 1 year)
				</span>
			</FormField>

			{totalAmount && installmentAmount && maxPeriods && (
				<div
					style={{
						padding: "12px",
						background: "#1a2a1a",
						borderRadius: "8px",
						fontSize: "13px",
						color: "#aaa",
						marginTop: "8px",
					}}
				>
					<div>
						<strong style={{ color: "#4caf50" }}>Summary</strong>
					</div>
					<div style={{ marginTop: "4px" }}>
						{Number(installmentAmount) * Number(maxPeriods)} KAS total drawn ·{" "}
						{formatInterval(intervalSecs)} intervals
					</div>
					{Number(installmentAmount) * Number(maxPeriods) > Number(totalAmount) && (
						<div style={{ color: "#ff9800", marginTop: "4px" }}>
							⚠️ Total drawn exceeds total locked — adjust amounts
						</div>
					)}
				</div>
			)}

			<button
				className="button primary"
				type="submit"
				disabled={status === "loading"}
				style={{ marginTop: "12px" }}
			>
				{status === "loading" ? "Creating…" : "Create Subscription"}
			</button>
		</form>
	);
}

function SubscriptionLookup() {
	const [id, setId] = useState("");
	const [sub, setSub] = useState<LoadState<Subscription>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!id.trim()) return;
		setSub({ loading: true });
		try {
			setSub({ data: await api.getSubscription(id.trim()), loading: false });
		} catch (err) {
			setSub({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<div>
			<form className="form" onSubmit={handleSubmit} style={{ marginBottom: "16px" }}>
				<input
					value={id}
					onChange={(e) => setId(e.target.value)}
					placeholder="subscription id (sub_...)"
				/>
				<button className="button primary" type="submit" disabled={sub.loading}>
					{sub.loading ? "Loading…" : "Fetch"}
				</button>
			</form>
			{sub.error && <p className="muted error-text">{sub.error}</p>}
			{sub.data && (
				<div className="panel">
					<div className="stack">
						<div className="row">
							<span>ID</span>
							<code>{sub.data.id}</code>
						</div>
						<div className="row">
							<span>Status</span>
							<strong>
								<span className={subscriptionStatusBadge(sub.data.status)}>
									{statusLabel(sub.data.status)}
								</span>
							</strong>
						</div>
						<div className="row">
							<span>Total Amount</span>
							<strong>{money(sub.data.total_amount)}</strong>
						</div>
						<div className="row">
							<span>Installment</span>
							<strong>
								{money(sub.data.installment_amount)} / {formatInterval(sub.data.interval_seconds)}
							</strong>
						</div>
						<div className="row">
							<span>Progress</span>
							<strong>
								{sub.data.current_period} / {sub.data.max_periods}
							</strong>
						</div>
						<div className="row">
							<span>Payer</span>
							<strong className="addr">{sub.data.payer_address}</strong>
						</div>
						<div className="row">
							<span>Recipient</span>
							<strong className="addr">{sub.data.recipient_address}</strong>
						</div>
						<div className="row">
							<span>Created</span>
							<strong>{time(sub.data.created_at)}</strong>
						</div>
						{sub.data.cancelled_at && (
							<div className="row">
								<span>Cancelled</span>
								<strong>{time(sub.data.cancelled_at)}</strong>
							</div>
						)}
						{sub.data.completed_at && (
							<div className="row">
								<span>Completed</span>
								<strong>{time(sub.data.completed_at)}</strong>
							</div>
						)}
					</div>
				</div>
			)}
		</div>
	);
}
