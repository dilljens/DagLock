import { useState, useEffect, useCallback } from "react";
import { api, type Vault, type VaultType, type VaultStatus } from "../api";
import { money, time, sompi, type LoadState } from "../helpers";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";

type Tab = "my-vaults" | "create" | "lookup";

export function VaultsPage() {
	const [tab, setTab] = useState<Tab>("my-vaults");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<div>
			<div className="page-header">
				<h1>🏦 Vaults</h1>
				<p>Time-locked KAS storage. Only you can withdraw after the timeout.</p>
			</div>
			<div className="tab-bar">
				<button className={`tab-btn ${tab === "my-vaults" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("my-vaults")}>My Vaults</button>
				<button className={`tab-btn ${tab === "create" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("create")}>Create</button>
				<button className={`tab-btn ${tab === "lookup" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("lookup")}>Lookup</button>
			</div>
			{tab === "my-vaults" && (wallet.connected ? <MyVaults address={address!} /> : <ConnectPrompt />)}
			{tab === "create" && (wallet.connected ? <CreateVault address={address!} /> : <ConnectPrompt />)}
			{tab === "lookup" && <VaultLookup />}
		</div>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<div className="empty-state">
			<div className="empty-state-icon">🔗</div>
			<h3>Connect your wallet</h3>
			<p>Connect KasWare to manage vaults.</p>
			<button className="button primary" onClick={connect}>Connect Wallet</button>
		</div>
	);
}

function formatVaultType(type: VaultType): string {
	const map: Record<VaultType, string> = {
		time: "Time-locked", beneficiary: "Beneficiary", deadman: "Deadman switch",
		inheritance: "Inheritance", multisig: "Multi-sig",
	};
	return map[type] || type;
}

function formatVaultStatus(status: VaultStatus): string {
	const map: Record<VaultStatus, string> = {
		locked: "🔒 Locked", unlocked: "🔓 Unlocked", expired: "⏰ Expired", transferred: "↗️ Transferred",
	};
	return map[status] || status;
}

function timeRemaining(untilTs: number): string {
	const secs = untilTs - Math.floor(Date.now() / 1000);
	if (secs <= 0) return "Ready to withdraw";
	const d = Math.floor(secs / 86400);
	const h = Math.floor((secs % 86400) / 3600);
	if (d > 0) return `${d}d ${h}h remaining`;
	return `${h}h ${Math.floor((secs % 3600) / 60)}m remaining`;
}

function MyVaults({ address }: { address: string }) {
	const [vaults, setVaults] = useState<LoadState<Vault[]>>({ loading: true });
	const { notify } = useToast();
	const { sign } = useWallet();

	const load = useCallback(() => {
		setVaults({ loading: true });
		api.vaults(address)
			.then((d) => setVaults({ data: d.vaults, loading: false }))
			.catch((e) => setVaults({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => { load(); }, [load]);

	if (vaults.loading) return <p className="muted">Loading vaults…</p>;
	if (vaults.error) return <p className="muted error-text">{vaults.error}</p>;
	if (!vaults.data?.length) return (
		<div className="empty-state">
			<div className="empty-state-icon">🏦</div>
			<h3>No vaults yet</h3>
			<p>Create your first time-locked vault.</p>
		</div>
	);

	return (
		<div>
			{vaults.data.map((v) => {
				const now = Math.floor(Date.now() / 1000);
				const canWithdraw = v.status === "locked" && now >= v.timeout;
				return (
					<article key={v.id} className="offer" style={{ cursor: "default", marginBottom: "8px" }}>
						<div className="offer-top">
							<strong>{formatVaultType(v.vault_type)}</strong>
							<span className="pill">{formatVaultStatus(v.status)}</span>
						</div>
						<p>{money(v.amount_sompi)} KAS</p>
						<small className="muted">Timeout: {time(v.timeout)} · {timeRemaining(v.timeout)}</small>
						<code>{v.id}</code>
						{canWithdraw && <WithdrawButton vault={v} address={address} />}
					</article>
				);
			})}
		</div>
	);
}

function WithdrawButton({ vault, address }: { vault: Vault; address: string }) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const [loading, setLoading] = useState(false);

	async function handleWithdraw() {
		setLoading(true);
		try {
			const sig = await sign(`withdraw:${vault.id}`);
			await api.withdrawVault(vault.id, address, sig);
			notify("success", "Vault unlocked!");
		} catch (e) {
			notify("error", "Withdraw failed", (e as Error).message);
		} finally {
			setLoading(false);
		}
	}

	return (
		<button className="button primary" disabled={loading} onClick={handleWithdraw}
			style={{ marginTop: "12px" }}>
			{loading ? "Withdrawing…" : "🔓 Withdraw"}
		</button>
	);
}

function CreateVault({ address }: { address: string }) {
	const [amount, setAmount] = useState("");
	const [timeoutDays, setTimeoutDays] = useState("30");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [vaultId, setVaultId] = useState("");
	const { notify } = useToast();

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		setStatus("loading");
		try {
			const timeoutSec = Math.floor(Date.now() / 1000) + (Number.parseInt(timeoutDays) || 30) * 86400;
			const vault = await api.createVault({
				owner_address: address,
				vault_type: "time",
				amount_sompi: sompi(amountNum),
				timeout: timeoutSec,
			});
			setVaultId(vault.id);
			setStatus("done");
			notify("success", "Vault created!");
		} catch (e) {
			notify("error", "Failed to create vault", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done") return (
		<div className="empty-state">
			<div className="empty-state-icon">✅</div>
			<h3>Vault created!</h3>
			<p>ID: <code>{vaultId}</code><br />Locked until timeout expires.</p>
		</div>
	);

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				Owner: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>
			<FormField label="Amount (KAS)">
				<input type="number" step="any" value={amount}
					onChange={(e) => setAmount(e.target.value)} placeholder="100" />
			</FormField>
			<FormField label="Lock duration">
				<select value={timeoutDays} onChange={(e) => setTimeoutDays(e.target.value)}>
					<option value="1">1 day</option>
					<option value="7">7 days</option>
					<option value="30">30 days</option>
					<option value="90">90 days</option>
					<option value="365">1 year</option>
				</select>
			</FormField>
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create Vault"}
			</button>
		</form>
	);
}

function VaultLookup() {
	const [id, setId] = useState("");
	const [vault, setVault] = useState<LoadState<Vault>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!id.trim()) return;
		setVault({ loading: true });
		try {
			setVault({ data: await api.vault(id.trim()), loading: false });
		} catch (err) {
			setVault({ error: (err as Error).message, loading: false });
		}
	}

	return (
		<div>
			<form className="form" onSubmit={handleSubmit} style={{ marginBottom: "16px" }}>
				<input value={id} onChange={(e) => setId(e.target.value)}
					placeholder="vault id (vault_...)" />
				<button className="button primary" type="submit" disabled={vault.loading}>
					{vault.loading ? "Loading…" : "Fetch"}
				</button>
			</form>
			{vault.error && <p className="muted error-text">{vault.error}</p>}
			{vault.data && (
				<div className="panel">
					<div className="stack">
						<div className="row"><span>Type</span><strong>{formatVaultType(vault.data.vault_type)}</strong></div>
						<div className="row"><span>Amount</span><strong>{money(vault.data.amount_sompi)}</strong></div>
						<div className="row"><span>Status</span><strong>{formatVaultStatus(vault.data.status)}</strong></div>
						<div className="row"><span>Timeout</span><strong>{time(vault.data.timeout)}</strong></div>
						<div className="row"><span>Time</span><strong>{timeRemaining(vault.data.timeout)}</strong></div>
						{vault.data.beneficiary_address && (
							<div className="row"><span>Beneficiary</span><strong className="addr">{vault.data.beneficiary_address}</strong></div>
						)}
						<div className="row"><span>Owner</span><strong className="addr">{vault.data.owner_address}</strong></div>
						<div className="row"><span>Created</span><strong>{time(vault.data.created_at)}</strong></div>
					</div>
				</div>
			)}
		</div>
	);
}
