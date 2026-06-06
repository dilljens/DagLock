import { useState } from "react";
import { api, type Vault, type VaultType, type VaultStatus } from "../api";
import { money, sompi, time, type LoadState } from "../helpers";
import { FormField } from "../ui";

/* ─── Create Vault Form ─── */
export function CreateVaultForm({ onDone }: { onDone: () => void }) {
	const [ownerAddress, setOwnerAddress] = useState("");
	const [amount, setAmount] = useState("");
	const [timeoutDays, setTimeoutDays] = useState("30");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<{
		script: string;
		template_hash: string;
	} | null>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const trimmedAddress = ownerAddress.trim();
		if (!trimmedAddress || !trimmedAddress.startsWith("kaspa:")) {
			setError("Enter a valid Kaspa address starting with 'kaspa:'");
			return;
		}
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) {
			setError("Amount must be a positive number");
			return;
		}

		const timeoutSec = Math.floor(Date.now() / 1000) + (Number.parseInt(timeoutDays) || 30) * 86400;
		setStatus("loading");
		setError("");

		try {
			// Create vault entry in database
			const vault = await api.createVault({
				owner_address: trimmedAddress,
				vault_type: "time",
				amount_sompi: sompi(amountNum),
				timeout: timeoutSec,
			});

			setResult({
				script: "Vault created",
				template_hash: vault.id,
			});
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done" && result) {
		return (
			<div className="result stack">
				<p className="muted success-text">Vault created!</p>
				<div className="row">
					<span>Vault ID</span>
					<code>{result.template_hash}</code>
				</div>
				<p className="muted">
					Your vault is now locked. You can withdraw after the timeout expires.
				</p>
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">
				Create a time-locked KAS vault. Only the owner can withdraw after the timeout.
			</p>
			<FormField label="Owner address">
				<input
					value={ownerAddress}
					onChange={(e) => setOwnerAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Amount (KAS)">
				<input
					type="number"
					step="any"
					value={amount}
					onChange={(e) => setAmount(e.target.value)}
					placeholder="100"
				/>
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
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create vault"}
			</button>
		</form>
	);
}

/* ─── Vault Lookup Panel ─── */
export function VaultLookup() {
	const [vaultId, setVaultId] = useState("");
	const [vault, setVault] = useState<Vault | null>(null);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!vaultId.trim()) return;
		setLoading(true);
		setError("");
		try {
			const data = await api.vault(vaultId.trim());
			setVault(data);
		} catch (err) {
			setError((err as Error).message);
			setVault(null);
		} finally {
			setLoading(false);
		}
	}

	return (
		<div className="panel">
			<div className="panel-head">
				<h3>Vault lookup</h3>
			</div>
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={vaultId}
					onChange={(e) => setVaultId(e.target.value)}
					placeholder="vault id (vault_...)"
				/>
				<button className="button" type="submit" disabled={loading}>
					{loading ? "Loading…" : "Fetch"}
				</button>
			</form>
			{error && <p className="muted error-text">{error}</p>}
			{vault && (
				<VaultStatusPanel
					vault={vault}
					onWithdraw={() => {
						setVault(null);
						setVaultId("");
					}}
				/>
			)}
		</div>
	);
}

/* ─── Vault List Panel ─── */
export function VaultListPanel() {
	const [address, setAddress] = useState("");
	const [list, setList] = useState<LoadState<Vault[]>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address.trim()) return;
		setList({ loading: true });
		try {
			const data = await api.vaults(address.trim());
			setList({ data: data.vaults, loading: false });
		} catch (err) {
			setList({ error: (err as Error).message, loading: false });
		}
	}

	function formatVaultType(type: VaultType): string {
		switch (type) {
			case "time":
				return "Time-locked";
			case "beneficiary":
				return "Beneficiary";
			case "deadman":
				return "Deadman switch";
			case "inheritance":
				return "Inheritance";
			case "multisig":
				return "Multi-sig";
			default:
				return type;
		}
	}

	function formatVaultStatus(status: VaultStatus): string {
		switch (status) {
			case "locked":
				return "🔒 Locked";
			case "unlocked":
				return "🔓 Unlocked";
			case "expired":
				return "⏰ Expired";
			case "transferred":
				return "↗️ Transferred";
			default:
				return status;
		}
	}

	return (
		<div className="stack">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="your kaspa address"
				/>
				<button className="button" type="submit">
					List my vaults
				</button>
			</form>
			{list.loading && <p className="muted">Loading vaults…</p>}
			{list.error && <p className="muted error-text">{list.error}</p>}
			{list.data?.length === 0 && <p className="muted">No vaults found for this address.</p>}
			{list.data?.map((v) => (
				<article key={v.id} className="offer" style={{ cursor: "default" }}>
					<div className="offer-top">
						<strong>{formatVaultType(v.vault_type)}</strong>
						<span className={`pill pill-${v.status}`}>{formatVaultStatus(v.status)}</span>
					</div>
					<p>{money(v.amount_sompi)} KAS</p>
					<small className="muted">Expires: {time(v.timeout)}</small>
					<code>{v.id}</code>
				</article>
			))}
		</div>
	);
}

/* ─── Vault Status Panel ─── */
export function VaultStatusPanel({
	vault,
	onWithdraw,
}: {
	vault: Vault;
	onWithdraw: () => void;
}) {
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");

	const now = Math.floor(Date.now() / 1000);
	const isLocked = vault.status === "locked";
	const canWithdraw = isLocked && now >= vault.timeout;
	const timeRemaining = vault.timeout - now;

	function formatTimeRemaining(seconds: number): string {
		if (seconds <= 0) return "Ready to withdraw";
		const days = Math.floor(seconds / 86400);
		const hours = Math.floor((seconds % 86400) / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (days > 0) return `${days}d ${hours}h remaining`;
		if (hours > 0) return `${hours}h ${minutes}m remaining`;
		return `${minutes}m remaining`;
	}

	async function handleWithdraw() {
		const address = prompt("Enter your Kaspa address:");
		if (!address) return;
		const signature = prompt("Enter your signature (hex):");
		if (!signature) return;

		setStatus("loading");
		try {
			await api.withdrawVault(vault.id, address, signature);
			setStatus("done");
			onWithdraw();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	return (
		<div className="panel">
			<div className="panel-head">
				<h3>Vault Status</h3>
			</div>
			<div className="stack">
				<div className="row">
					<span>Type</span>
					<strong>{vault.vault_type}</strong>
				</div>
				<div className="row">
					<span>Amount</span>
					<strong>{money(vault.amount_sompi)} KAS</strong>
				</div>
				<div className="row">
					<span>Status</span>
					<strong className={isLocked ? "error-text" : "success-text"}>
						{isLocked ? "🔒 Locked" : "🔓 Unlocked"}
					</strong>
				</div>
				<div className="row">
					<span>Timeout</span>
					<strong>{time(vault.timeout)}</strong>
				</div>
				<div className="row">
					<span>Time</span>
					<strong className={canWithdraw ? "success-text" : ""}>
						{formatTimeRemaining(timeRemaining)}
					</strong>
				</div>
				{vault.beneficiary_address && (
					<div className="row">
						<span>Beneficiary</span>
						<strong className="addr">{vault.beneficiary_address}</strong>
					</div>
				)}
				<div className="row">
					<span>Created</span>
					<strong>{time(vault.created_at)}</strong>
				</div>
				{status === "done" && <p className="muted success-text">Vault unlocked successfully!</p>}
				{error && <p className="muted error-text">{error}</p>}
				{canWithdraw && (
					<button
						className="button primary"
						onClick={handleWithdraw}
						disabled={status === "loading"}
					>
						{status === "loading" ? "Withdrawing…" : "Withdraw"}
					</button>
				)}
			</div>
		</div>
	);
}
