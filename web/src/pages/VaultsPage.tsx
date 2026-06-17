import { useState, useEffect, useCallback } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { api, type Vault, type VaultType, type VaultStatus } from "../api";
import { money, time, sompi, type LoadState } from "../helpers";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonTable } from "../ui";
import { EmptyState } from "../components/empty-state";
import { z } from "zod";
import { CreateVaultSchema } from "../validation";

type Tab = "my-vaults" | "create" | "lookup";

export function VaultsPage() {
	const [tab, setTab] = useState<Tab>("my-vaults");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<div>
			<div className="page-header">
				<h1><h1> Vaults</h1></h1>
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
		<EmptyState
			icon="🔐"
			title="Connect your wallet"
			description="Connect KasWare to manage vaults."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

const VAULT_TYPE_INFO: Record<string, { label: string; desc: string }> = {
	time: { label: "Time-locked", desc: "Funds locked until timeout, then anyone can withdraw. Best for cold storage or inheritance." },
	beneficiary: { label: "Beneficiary", desc: "Time-locked with a beneficiary address. Beneficiary can withdraw after timeout without a password." },
	deadman: { label: "Deadman switch", desc: "Recurring timeout — must be refreshed periodically or funds are released to a beneficiary." },
	inheritance: { label: "Inheritance", desc: "Two-party vault with beneficiary timeout. Primary owner can withdraw anytime; beneficiary waits." },
	multisig: { label: "Multi-sig", desc: "Requires 2-of-3 signatures to withdraw. Best for team treasuries or shared accounts." },
};

function formatVaultType(type: VaultType): string {
	return VAULT_TYPE_INFO[type]?.label || type;
}

function vaultTypeBadge(type: string): string {
	const colors: Record<string, string> = {
		time: "#53d769", beneficiary: "#4fc3f7", deadman: "#ff9800",
		inheritance: "#ce93d8", multisig: "#ff7b7b",
	};
	const bg = colors[type] || "rgba(255,255,255,0.1)";
	return bg;
}

function formatVaultStatus(status: VaultStatus): string {
	const map: Record<VaultStatus, string> = {
		locked: "Locked", unlocked: "Unlocked", expired: "Expired", transferred: "Transferred",
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

	if (vaults.loading) return <SkeletonTable rows={5} />;
	if (vaults.error) return <p className="muted error-text">{vaults.error}</p>;
	if (!vaults.data?.length) return (
		<EmptyState
			icon="🏦"
			title="No vaults yet"
			description="Create your first time-locked vault."
		/>
	);

	return (
		<div>
			{vaults.data.map((v) => {
				const now = Math.floor(Date.now() / 1000);
				const canWithdraw = v.status === "locked" && now >= v.timeout;
				return (
					<article key={v.id} className="offer" style={{ cursor: "default", marginBottom: "8px" }}>
						<div className="offer-top">
							<strong>
								<span className="pill" style={{
									background: `${vaultTypeBadge(v.vault_type)}22`,
									color: vaultTypeBadge(v.vault_type),
									border: `1px solid ${vaultTypeBadge(v.vault_type)}44`,
								}}>
									{formatVaultType(v.vault_type)}
								</span>
								{money(v.amount_sompi)} KAS
							</strong>
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
			{loading ? "Withdrawing…" : " Withdraw"}
		</button>
	);
}

const VAULT_TYPES = ["time", "beneficiary", "multisig"] as const;

function CreateVault({ address }: { address: string }) {
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [vaultId, setVaultId] = useState("");
	const [vaultType, setVaultType] = useState<(typeof VAULT_TYPES)[number]>("time");
	const { notify } = useToast();

	const {
		register,
		handleSubmit,
		formState: { errors, isSubmitting },
	} = useForm<z.infer<typeof CreateVaultSchema>>({
		resolver: zodResolver(CreateVaultSchema),
		defaultValues: {
			owner_address: address,
			amount_sompi: 0,
			timeout_days: 30,
		},
	});

	async function onSubmit(data: z.infer<typeof CreateVaultSchema>) {
		setStatus("loading");
		try {
			const timeoutSec = Math.floor(Date.now() / 1000) + data.timeout_days * 86400;
			const vault = await api.createVault({
				owner_address: data.owner_address,
				vault_type: vaultType,
				amount_sompi: data.amount_sompi,
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
		<EmptyState
			icon="✅"
			title="Vault created!"
			description={`ID: ${vaultId} — Locked until timeout expires.`}
		/>
	);

	return (
		<form className="form form-stacked" onSubmit={handleSubmit(onSubmit)}>
			<div style={{ fontSize: "13px", color: "#88b888", padding: "8px 0" }}>
				Owner: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
			</div>

			{/* Vault type selector */}
			<div className="field" style={{ marginBottom: "16px" }}>
				<span style={{ fontSize: "13px", fontWeight: 600, display: "block", marginBottom: "8px" }}>Vault Type</span>
				<div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
					{VAULT_TYPES.map((t) => {
						const info = VAULT_TYPE_INFO[t];
						const active = vaultType === t;
						return (
							<label
								key={t}
								onClick={() => setVaultType(t)}
								style={{
									display: "flex", gap: "12px", padding: "12px", borderRadius: "12px",
									border: `1px solid ${active ? vaultTypeBadge(t) : "var(--color-border)"}`,
									background: active ? `${vaultTypeBadge(t)}11` : "transparent",
									cursor: "pointer", transition: "all 0.15s ease",
								}}
							>
								<input type="radio" name="vaultType" checked={active} readOnly
									style={{ accentColor: vaultTypeBadge(t), marginTop: "2px" }} />
								<div>
									<div style={{ fontWeight: 700, fontSize: "14px", color: active ? vaultTypeBadge(t) : "var(--color-text)" }}>
										{info.label}
									</div>
									<div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "2px" }}>
										{info.desc}
									</div>
								</div>
							</label>
						);
					})}
				</div>
			</div>

			<FormField label="Amount (KAS)">
				<input
					type="number"
					step="any"
					placeholder="100"
					{...register("amount_sompi", { valueAsNumber: true })}
				/>
				{errors.amount_sompi && <span className="input-feedback error">{errors.amount_sompi.message}</span>}
			</FormField>
			<FormField label="Lock duration">
				<select {...register("timeout_days", { valueAsNumber: true })}>
					<option value={1}>1 day</option>
					<option value={7}>7 days</option>
					<option value={30}>30 days</option>
					<option value={90}>90 days</option>
					<option value={365}>1 year</option>
				</select>
				{errors.timeout_days && <span className="input-feedback error">{errors.timeout_days.message}</span>}
			</FormField>
			<input type="hidden" {...register("owner_address")} />
			<button className="button primary" type="submit" disabled={isSubmitting || status === "loading"}>
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
