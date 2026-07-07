import { useState, useEffect, useCallback } from "react";
import { api, type Escrow } from "../api";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { money, sompi } from "../helpers";
import { FormField, Panel } from "../ui";
import { FeeCalculator } from "./FeeCalculator";
import { ExplorerEscrowLink } from "./ExplorerLink";

type SwapStep = "init" | "secrets" | "create" | "waiting" | "claim" | "done";

interface SwapState {
	amount: string;
	assetType: string;
	counterpartyAddress: string;
	secret: string;
	tradeHash: string;
	escrowId: string | null;
	escrow: Escrow | null;
	timeout: number;
	preimage: string;
}

const STEP_LABELS: Record<SwapStep, string> = {
	init: "Terms",
	secrets: "Generate Secret",
	create: "Create Escrow",
	waiting: "Waiting",
	claim: "Claim",
	done: "Complete",
};

export function AtomicSwapWizard() {
	const address = useAddress();
	const { state: wallet } = useWallet();
	const { notify } = useToast();
	const [step, setStep] = useState<SwapStep>("init");
	const [s, setS] = useState<SwapState>({
		amount: "",
		assetType: "KAS",
		counterpartyAddress: "",
		secret: "",
		tradeHash: "",
		escrowId: null,
		escrow: null,
		timeout: 86400,
		preimage: "",
	});
	const [loading, setLoading] = useState("");
	const [error, setError] = useState("");

	function updateField<K extends keyof SwapState>(key: K, value: SwapState[K]) {
		setS((prev) => ({ ...prev, [key]: value }));
	}

	// ── Step 1: Init ──
	function handleInitSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(s.amount);
		if (!amountNum || amountNum <= 0) {
			setError("Enter a valid amount");
			return;
		}
		if (!s.counterpartyAddress.startsWith("kaspa:")) {
			setError("Enter a valid counterparty Kaspa address");
			return;
		}
		setError("");
		setStep("secrets");
	}

	// ── Step 2: Generate Secret ──
	async function handleGenerateSecret() {
		setLoading("generating");
		setError("");
		try {
			const res = await api.generateSwap();
			updateField("secret", res.secret);
			updateField("tradeHash", res.hash);
			setStep("create");
		} catch (err) {
			setError((err as Error).message);
		} finally {
			setLoading("");
		}
	}

	// ── Step 3: Create Escrow ──
	async function handleCreateEscrow() {
		if (!address) {
			setError("Connect your wallet first");
			return;
		}
		setLoading("creating");
		setError("");
		try {
			let lockTxId = "";
			if (window.kasware?.sendKaspa) {
				lockTxId = await window.kasware.sendKaspa(
					s.counterpartyAddress,
					sompi(Number.parseFloat(s.amount)),
				);
			} else {
				lockTxId = prompt("Enter tx_id from wallet (or hex for testnet):") || "";
				if (!lockTxId) throw new Error("Tx ID required");
			}

			const escrow = await api.createEscrow({
				lock_tx_id: lockTxId,
				lock_tx_output_index: 0,
				buyer_address: address,
				seller_address: s.counterpartyAddress,
				amount_sompi: sompi(Number.parseFloat(s.amount)),
				asset_type: s.assetType,
				trade_hash: s.tradeHash,
			});
			updateField("escrowId", escrow.id);
			updateField("escrow", escrow);
			notify("success", "Swap escrow created!");
			setStep("waiting");
		} catch (err) {
			setError((err as Error).message);
		} finally {
			setLoading("");
		}
	}

	// ── Step 4: Waiting (poll for status) ──
	useEffect(() => {
		if (step !== "waiting" || !s.escrowId) return;
		const interval = setInterval(async () => {
			try {
				const data = await api.escrow(s.escrowId!);
				updateField("escrow", data);
				if (data.status === "settled") {
					notify("success", "Escrow settled by counterparty!");
					setStep("done");
					clearInterval(interval);
				}
				if (data.status === "refunded" || data.status === "cancelled") {
					setStep("done");
					clearInterval(interval);
				}
			} catch {
				// retry on next interval
			}
		}, 10_000);
		return () => clearInterval(interval);
	}, [step, s.escrowId]); // eslint-disable-line react-hooks/exhaustive-deps

	// ── Step 5: Claim ──
	async function handleClaim(e: React.FormEvent) {
		e.preventDefault();
		if (!s.escrowId || !s.preimage.trim()) return;
		setLoading("claiming");
		setError("");
		try {
			const res = await api.swapEscrow(s.escrowId, s.preimage.trim());
			notify("success", "Swap settled!");
			setStep("done");
		} catch (err) {
			setError((err as Error).message);
		} finally {
			setLoading("");
		}
	}

	const escrowLink = s.escrowId
		? `${window.location.origin}/swap/${s.escrowId}`
		: "";
	const telegramLink = s.escrowId
		? `https://t.me/DagLock_bot?start=swap_${s.escrowId}`
		: "";

	async function copyToClipboard(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			notify("success", "Copied to clipboard!");
		} catch {
			// fallback
		}
	}

	if (!wallet.connected) {
		return (
			<Panel title="Atomic Swap Wizard">
				<p className="muted">Connect your wallet to start an atomic swap.</p>
			</Panel>
		);
	}

	return (
		<div className="swap-wizard">
			{/* Step indicator */}
			<div className="swap-wizard-steps">
				{(["init", "secrets", "create", "waiting", "claim", "done"] as SwapStep[]).map(
					(st, i) => {
						const stepIdx = ["init", "secrets", "create", "waiting", "claim", "done"].indexOf(step);
						const thisIdx = ["init", "secrets", "create", "waiting", "claim", "done"].indexOf(st);
						const isActive = st === step;
						const isPast = thisIdx < stepIdx;
						return (
							<div
								key={st}
								className={`swap-wizard-step ${isActive ? "active" : ""} ${isPast ? "past" : ""}`}
							>
								<div className="swap-wizard-step-num">{isPast ? "✓" : i + 1}</div>
								<span className="swap-wizard-step-label">{STEP_LABELS[st]}</span>
							</div>
						);
					},
				)}
			</div>

			<div className="swap-wizard-body">
				{/* Step: Init */}
				{step === "init" && (
					<form onSubmit={handleInitSubmit} className="form form-stacked">
						<h3 style={{ margin: "0 0 4px" }}>Start an Atomic Swap</h3>
						<p className="muted" style={{ margin: "0 0 16px" }}>
							Set the terms for your swap. You'll generate a secret in the next step.
						</p>

						<FormField label="Asset type">
							<select
								value={s.assetType}
								onChange={(e) => updateField("assetType", e.target.value)}
							>
								<option value="KAS">KAS</option>
								<option value="KRC20">KRC-20</option>
							</select>
						</FormField>

						<FormField label="Amount">
							<input
								type="number"
								step="any"
								value={s.amount}
								onChange={(e) => updateField("amount", e.target.value)}
								placeholder="100"
							/>
						</FormField>

						{!(isNaN(Number(s.amount)) || Number(s.amount) <= 0) && (
							<div style={{ marginBottom: "12px" }}>
								<FeeCalculator />
							</div>
						)}

						<FormField label="Counterparty address">
							<input
								value={s.counterpartyAddress}
								onChange={(e) => updateField("counterpartyAddress", e.target.value)}
								placeholder="kaspa:..."
							/>
						</FormField>

						<FormField label="Timeout">
							<select
								value={s.timeout}
								onChange={(e) => updateField("timeout", Number(e.target.value))}
							>
								<option value={3600}>1 hour</option>
								<option value={86400}>24 hours</option>
								<option value={259200}>3 days</option>
								<option value={604800}>7 days</option>
							</select>
						</FormField>

						{error && <p className="muted error-text">{error}</p>}
						<button className="button primary" type="submit">
							Next: Generate Secret
						</button>
					</form>
				)}

				{/* Step: Secrets (handled via generate button click) */}
				{step === "secrets" && (
					<div>
						<h3 style={{ margin: "0 0 4px" }}>Generate Secret</h3>
						<p className="muted" style={{ margin: "0 0 16px" }}>
							Generate a random secret and its SHA-256 hash. The hash goes into the covenant.
						</p>

						<div
							style={{
								background: "#332200",
								border: "1px solid #ff9800",
								borderRadius: "8px",
								padding: "16px",
								marginBottom: "16px",
							}}
						>
							<strong style={{ color: "#ff9800" }}>⚠ Important</strong>
							<p style={{ fontSize: "13px", margin: "8px 0 0", color: "#ccc" }}>
								Save the secret before continuing. It's shown only once and never stored on the
								server. If you lose it, the funds will be locked until the timeout expires.
							</p>
						</div>

						{error && <p className="muted error-text">{error}</p>}
						<button
							className="button primary"
							onClick={handleGenerateSecret}
							disabled={loading === "generating"}
						>
							{loading === "generating" ? "Generating…" : "Generate Secret & Continue"}
						</button>
					</div>
				)}

				{/* Step: Create */}
				{step === "create" && (
					<div>
						<h3 style={{ margin: "0 0 4px" }}>Your Secret</h3>
						<div
							style={{
								background: "#3a1a1a",
								border: "1px solid #ff4444",
								borderRadius: "8px",
								padding: "16px",
								marginBottom: "16px",
							}}
						>
							<strong style={{ color: "#ff4444" }}>🔑 Secret (save this!)</strong>
							<div style={{ display: "flex", gap: "8px", marginTop: "8px", alignItems: "center" }}>
								<code
									style={{
										flex: 1,
										padding: "8px",
										background: "#1a1a1a",
										borderRadius: "4px",
										fontSize: "12px",
										wordBreak: "break-all",
									}}
								>
									{s.secret}
								</code>
								<button className="button" onClick={() => copyToClipboard(s.secret)}>
									Copy
								</button>
							</div>
						</div>

						<div
							style={{
								background: "#1a2a1a",
								border: "1px solid #53d769",
								borderRadius: "8px",
								padding: "12px",
								marginBottom: "16px",
							}}
						>
							<strong style={{ color: "#53d769" }}>🔗 Hash (share with counterparty)</strong>
							<code
								style={{
									display: "block",
									marginTop: "8px",
									padding: "8px",
									background: "#0a1a0a",
									borderRadius: "4px",
									fontSize: "12px",
									wordBreak: "break-all",
								}}
							>
								{s.tradeHash}
							</code>
						</div>

						<h3 style={{ margin: "16px 0 4px" }}>Create Escrow</h3>
						<p className="muted" style={{ margin: "0 0 12px" }}>
							Lock funds in the covenant with the trade hash. Your counterparty will need the
							secret to claim.
						</p>

						{error && <p className="muted error-text">{error}</p>}
						<button
							className="button primary"
							onClick={handleCreateEscrow}
							disabled={loading === "creating"}
						>
							{loading === "creating"
								? "Creating…"
								: `Create Escrow — ${money(sompi(Number.parseFloat(s.amount) || 0))}`}
						</button>
					</div>
				)}

				{/* Step: Waiting */}
				{step === "waiting" && (
					<div>
						<h3 style={{ margin: "0 0 4px" }}>Waiting for Counterparty</h3>
						<p className="muted" style={{ margin: "0 0 16px" }}>
							Share the link below with your counterparty so they can claim the swap.
						</p>

						<div className="panel" style={{ marginBottom: "16px" }}>
							<div
								style={{
									display: "flex",
									alignItems: "center",
									gap: "12px",
									marginBottom: "12px",
								}}
							>
								<div
									className="loading-spinner"
									style={{
										width: 20,
										height: 20,
										border: "2px solid #333",
										borderTop: "2px solid #53d769",
										borderRadius: "50%",
										animation: "spin 1s linear infinite",
									}}
								/>
								<span>Waiting for counterparty to claim…</span>
							</div>

							{s.escrow?.created_at && (
								<p style={{ fontSize: "13px", color: "#888" }}>
									Timeout:{" "}
									{new Date(
										(s.escrow.created_at + s.timeout) * 1000,
									).toLocaleString()}
								</p>
							)}
						</div>

						<FormField label="Share this link">
							<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
								<input value={escrowLink} readOnly style={{ flex: 1, fontSize: "12px" }} />
								<button className="button" onClick={() => copyToClipboard(escrowLink)}>
									Copy
								</button>
							</div>
						</FormField>

						<FormField label="Telegram link">
							<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
								<input
									value={telegramLink}
									readOnly
									style={{ flex: 1, fontSize: "12px" }}
								/>
								<button className="button" onClick={() => copyToClipboard(telegramLink)}>
									Copy
								</button>
							</div>
						</FormField>

						{s.escrowId && (
							<div style={{ marginTop: "8px" }}>
								<ExplorerEscrowLink escrowId={s.escrowId} />
							</div>
						)}
					</div>
				)}

				{/* Step: Claim (counterparty view) */}
				{step === "claim" && (
					<form onSubmit={handleClaim} className="form form-stacked">
						<h3 style={{ margin: "0 0 4px" }}>Claim Atomic Swap</h3>
						<p className="muted" style={{ margin: "0 0 16px" }}>
							Enter the secret preimage to claim this swap. The covenant verifies that
							SHA-256(preimage) matches the trade hash.
						</p>

						{s.escrow && (
							<div className="result stack" style={{ marginBottom: "16px" }}>
								<div className="row">
									<span>Escrow</span>
									<code>{s.escrow.id}</code>
								</div>
								<div className="row">
									<span>Amount</span>
									<strong>{money(s.escrow.amount_sompi)}</strong>
								</div>
								{s.escrow.trade_hash && (
									<div className="row">
										<span>Expected hash</span>
										<code style={{ fontSize: "11px", wordBreak: "break-all" }}>
											{s.escrow.trade_hash}
										</code>
									</div>
								)}
							</div>
						)}

						<FormField label="Preimage (secret)">
							<input
								value={s.preimage}
								onChange={(e) => updateField("preimage", e.target.value)}
								placeholder="Paste the secret here"
							/>
						</FormField>

						{error && <p className="muted error-text">{error}</p>}
						<button
							className="button primary"
							type="submit"
							disabled={loading === "claiming" || !s.preimage.trim()}
						>
							{loading === "claiming" ? "Claiming…" : "Claim Swap"}
						</button>
					</form>
				)}

				{/* Step: Done */}
				{step === "done" && (
					<div style={{ textAlign: "center", padding: "24px 0" }}>
						<div style={{ fontSize: "48px", marginBottom: "16px" }}>✅</div>
						<h3 style={{ margin: "0 0 8px" }}>Swap Complete</h3>
						<p className="muted" style={{ margin: "0 0 20px" }}>
							The atomic swap has been settled successfully.
						</p>

						{s.escrowId && (
							<div style={{ marginBottom: "16px" }}>
								<ExplorerEscrowLink escrowId={s.escrowId} />
							</div>
						)}

						{s.escrow?.id && (
							<button
								className="button primary"
								onClick={async () => {
									try {
										const receipt = await api.receipt(s.escrow!.id);
										notify("success", "Receipt fetched!");
									} catch {
										notify("error", "Could not fetch receipt");
									}
								}}
							>
								View Receipt
							</button>
						)}

						<button
							className="button"
							onClick={() => {
								setS({
									amount: "",
									assetType: "KAS",
									counterpartyAddress: "",
									secret: "",
									tradeHash: "",
									escrowId: null,
									escrow: null,
									timeout: 86400,
									preimage: "",
								});
								setStep("init");
								setError("");
							}}
							style={{ marginLeft: "8px" }}
						>
							Start Another Swap
						</button>
					</div>
				)}
			</div>
		</div>
	);
}
