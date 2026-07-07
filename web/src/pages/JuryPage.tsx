import { useState, useCallback, useEffect } from "react";
import {
	api,
	type AuthHeaders,
	type JuryCase,
	type JurorRegistration,
	type EvidenceMessage,
} from "../api";
import { badge, type LoadState } from "../helpers";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonTable } from "../ui";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";
import { MediationPanel } from "../components/MediationPanel";

type Tab = "my-cases" | "register" | "candidates";

export function JuryPage() {
	const [tab, setTab] = useState<Tab>("my-cases");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<>
			<Helmet>
				<title>Jury — DagLock</title>
				<meta
					name="description"
					content="Decentralized dispute resolution for Kaspa escrow trades via community jury."
				/>
				<link rel="canonical" href="https://daglock.com/jury" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>⚖ Jury</h1>
					<p>Community dispute resolution. Register as a juror, vote on cases.</p>
				</div>

				{/* How jury works — collapsible */}
				<details
					className="panel"
					style={{ marginBottom: "16px", padding: "12px 16px", cursor: "pointer" }}
				>
					<summary style={{ fontWeight: 600, fontSize: "14px", color: "var(--color-text)" }}>
						⚖️ How the Jury System Works
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
							When an escrow is disputed with <strong>"jury"</strong> dispute mode, randomly
							selected community members vote to decide the outcome.
						</p>
						<p style={{ margin: "0 0 8px" }}>
							<strong>How it works:</strong>
						</p>
						<ul style={{ margin: "0 0 8px", paddingLeft: "20px" }}>
							<li>
								<strong>Registering:</strong> Anyone with 10+ trades and a 3.0+ reputation score can
								register as a juror. Registration costs nothing.
							</li>
							<li>
								<strong>Selection:</strong> For each case, the top candidates by reliability score
								are pooled and then randomly selected. Threshold varies by escrow value: 2/3 for
								small, 3/5 for medium, 5/9 for large.
							</li>
							<li>
								<strong>Voting:</strong> Selected jurors vote "seller wins" or "buyer wins" and
								provide reasoning. Votes are visible to both parties.
							</li>
							<li>
								<strong>Timeout:</strong> If no verdict is reached within 72 hours, seller wins by
								default.
							</li>
							<li>
								<strong>Escalation Tiers:</strong> Disputes auto-escalate through levels if
								unresolved: mediation (2 days) → jury vote (5 days) → admin override (10 days).
							</li>
						</ul>
						<p style={{ margin: 0 }}>
							Jury decisions are recorded on-chain via the arbiter covenant. The winning party must
							broadcast the transaction to release funds.
						</p>
					</div>
				</details>

				{/* Escrow dispute lookup — show mediation panel for any escrow */}
				<DisputeMediationLookup />

				<div className="tab-bar">
					{wallet.connected && (
						<button
							className={`tab-btn ${tab === "my-cases" ? "tab-btn--active" : ""}`}
							onClick={() => setTab("my-cases")}
						>
							My Cases
						</button>
					)}
					<button
						className={`tab-btn ${tab === "register" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("register")}
					>
						Register
					</button>
					<button
						className={`tab-btn ${tab === "candidates" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("candidates")}
					>
						Candidates
					</button>
				</div>
				{tab === "my-cases" &&
					(wallet.connected ? <MyCases address={address!} /> : <ConnectPrompt />)}
				{tab === "register" &&
					(wallet.connected ? <RegisterSection address={address!} /> : <ConnectPrompt />)}
				{tab === "candidates" && <CandidatesSection />}
			</div>
		</>
	);
}

/* ─── Dispute Mediation Lookup ─── */
function DisputeMediationLookup() {
	const [escrowId, setEscrowId] = useState("");
	const [searched, setSearched] = useState(false);
	const [mediation, setMediation] = useState<any>(null);
	const [error, setError] = useState("");

	async function handleLookup() {
		if (!escrowId.trim()) return;
		setSearched(false);
		setError("");
		setMediation(null);
		try {
			const s = await api.getMediation(escrowId.trim());
			setMediation(s);
		} catch {
			setError("No mediation found for this escrow.");
		}
		setSearched(true);
	}

	return (
		<div className="panel" style={{ marginBottom: "16px", padding: "12px 16px" }}>
			<h4 style={{ margin: "0 0 8px" }}>🔍 Check Mediation Status</h4>
			<p className="muted" style={{ fontSize: "13px", marginBottom: "8px" }}>
				Enter an escrow ID to view its AI mediation status and accept/reject the outcome.
			</p>
			<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
				<input
					value={escrowId}
					onChange={(e) => setEscrowId(e.target.value)}
					placeholder="esc_abc123..."
					style={{
						flex: 1,
						padding: "8px",
						borderRadius: "6px",
						border: "1px solid var(--color-border)",
					}}
				/>
				<button className="button secondary" onClick={handleLookup}>
					Check
				</button>
			</div>
			{searched && mediation && (
				<div style={{ marginTop: "12px" }}>
					<MediationPanel escrowId={escrowId.trim()} />
				</div>
			)}
			{error && (
				<p className="muted error-text" style={{ marginTop: "8px" }}>
					{error}
				</p>
			)}
		</div>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<EmptyState
			icon="👛"
			title="Connect your wallet"
			description="Connect KasWare to participate in the jury system."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

/* ─── My Cases ─── */
function MyCases({ address }: { address: string }) {
	const [cases, setCases] = useState<LoadState<JuryCase[]>>({ loading: true });
	const [selectedId, setSelectedId] = useState<string | null>(null);
	const { sign } = useWallet();
	const { notify } = useToast();

	const load = useCallback(() => {
		setCases({ loading: true });
		api
			.juryActiveCases(address)
			.then((d) => setCases({ data: d.cases, loading: false }))
			.catch((e) => setCases({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => {
		load();
	}, [load]);

	if (cases.loading) return <SkeletonTable rows={5} />;
	if (cases.error) return <p className="muted error-text">{cases.error}</p>;
	if (!cases.data?.length)
		return (
			<EmptyState
				icon="⚖"
				title="No active cases"
				description="You're not assigned to any jury cases right now."
			/>
		);

	return (
		<div>
			{cases.data.map((c) => {
				const escalationLabels = ["Mediation", "Jury Vote", "Admin Override"];
				const deadlineMs = c.escalation_deadline ? c.escalation_deadline * 1000 : null;
				const remaining = deadlineMs ? deadlineMs - Date.now() : 0;
				const remainingDays = remaining > 0 ? Math.ceil(remaining / 86400000) : 0;
				return (
					<article
						key={c.id}
						className="offer"
						style={{ cursor: "pointer", marginBottom: "8px" }}
						onClick={() => setSelectedId(selectedId === c.id ? null : c.id)}
					>
						<div className="offer-top">
							<strong>Case {c.id.slice(0, 16)}…</strong>
							<span>
								<span className={badge(c.status)}>{c.status}</span>
								<span className="pill" style={{ marginLeft: "6px" }}>
									{escalationLabels[c.escalation_level] || "Unknown"}
								</span>
							</span>
						</div>
						<p>
							Escrow: {c.escrow_id} · Votes: {c.votes_for_seller + c.votes_for_buyer}/
							{c.juror_count} · Threshold: {c.threshold}
							{remaining > 0 && ` · Escalates in ${remainingDays}d`}
						</p>
						<code>{c.id}</code>
						{selectedId === c.id && (
							<>
								{c.escalation_level <= 1 && (
									<MediationPanel escrowId={c.escrow_id} disputeMode="jury" />
								)}
								{c.status === "voting" && <VotePanel case={c} address={address} onVoted={load} />}
								<EvidenceSection caseId={c.id} escrowId={c.escrow_id} status={c.status} />
							</>
						)}
					</article>
				);
			})}
		</div>
	);
}

function VotePanel({
	case: c,
	address,
	onVoted,
}: { case: JuryCase; address: string; onVoted: () => void }) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const [vote, setVote] = useState("");
	const [reasoning, setReasoning] = useState("");
	const [loading, setLoading] = useState(false);

	async function handleVote() {
		if (!vote) return;
		setLoading(true);
		try {
			const auth: AuthHeaders = {
				address,
				signature: await sign(`vote:${c.id}`),
				message: `vote:${c.id}`,
			};
			const r = await api.juryVote(c.id, vote, reasoning || undefined, auth);
			notify("success", r.verdict ? `Verdict: ${r.vote}` : `Voted: ${r.vote}`);
			onVoted();
		} catch (e) {
			notify("error", "Vote failed", (e as Error).message);
		} finally {
			setLoading(false);
		}
	}

	return (
		<div className="panel" style={{ marginTop: "12px" }}>
			<h4 style={{ margin: "0 0 12px" }}>Cast your vote</h4>
			<FormField label="Vote">
				<select value={vote} onChange={(e) => setVote(e.target.value)}>
					<option value="">— select —</option>
					<option value="seller_wins">Seller wins</option>
					<option value="buyer_wins">Buyer wins</option>
				</select>
			</FormField>
			<FormField label="Reasoning (optional)">
				<input
					value={reasoning}
					onChange={(e) => setReasoning(e.target.value)}
					placeholder="Why?"
				/>
			</FormField>
			<button className="button primary" disabled={!vote || loading} onClick={handleVote}>
				{loading ? "Submitting…" : "Submit Vote"}
			</button>
		</div>
	);
}

/* ─── Register ─── */
function RegisterSection({ address }: { address: string }) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");

	async function handleRegister() {
		setStatus("loading");
		try {
			const auth: AuthHeaders = {
				address,
				signature: await sign("jury:register"),
				message: "jury:register",
			};
			await api.juryRegister(auth);
			setStatus("done");
			notify("success", "Registered as juror!");
		} catch (e) {
			notify("error", "Registration failed", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done")
		return (
			<EmptyState
				icon="✅"
				title="Vote submitted!"
				description="You'll be assigned to dispute cases when they arise. Stay responsive."
			/>
		);

	return (
		<div>
			<div className="panel">
				<h3 style={{ margin: "0 0 12px" }}>Become a Juror</h3>
				<p className="muted" style={{ fontSize: "13px", marginBottom: "16px" }}>
					Jurors resolve disputes by voting. You need 10+ trades and a 3.0+ reputation score. When
					assigned, you have 72 hours to vote.
				</p>
				<div style={{ fontSize: "13px", color: "#88b888", marginBottom: "16px" }}>
					Address:{" "}
					<code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
				</div>
				<button className="button primary" onClick={handleRegister} disabled={status === "loading"}>
					{status === "loading" ? "Registering…" : "Register as Juror"}
				</button>
			</div>
		</div>
	);
}

/* ─── Candidates ─── */
function CandidatesSection() {
	const [candidates, setCandidates] = useState<LoadState<JurorRegistration[]>>({ loading: true });

	useEffect(() => {
		api
			.juryCandidates()
			.then((d) => setCandidates({ data: d.candidates, loading: false }))
			.catch((e) => setCandidates({ error: e.message, loading: false }));
	}, []);

	if (candidates.loading) return <SkeletonTable rows={5} />;
	if (candidates.error) return <p className="muted error-text">{candidates.error}</p>;
	if (!candidates.data?.length)
		return (
			<EmptyState icon="👤" title="No jurors registered" description="Be the first to register!" />
		);

	return (
		<div>
			<p className="muted" style={{ marginBottom: "12px" }}>
				{candidates.data.length} registered jurors
			</p>
			{candidates.data.map((c) => (
				<article
					key={c.address}
					className="offer"
					style={{ cursor: "default", marginBottom: "8px" }}
				>
					<div className="offer-top">
						<strong>{c.address.slice(0, 24)}…</strong>
						<span className="pill">{c.reliability_score.toFixed(1)}/5</span>
					</div>
					<p>
						Cases: {c.total_cases_assigned} assigned · {c.total_cases_voted} voted
					</p>
					<small className="muted">
						Registered {new Date(c.registered_at * 1000).toLocaleDateString()}
					</small>
				</article>
			))}
		</div>
	);
}

/* ─── Chat Evidence ─── */
function EvidenceSection({
	caseId,
	escrowId,
	status,
}: { caseId: string; escrowId: string; status: string }) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const address = useAddress();
	const [evidence, setEvidence] = useState<
		LoadState<{
			evidence: EvidenceMessage[];
			chat_pubkey_buyer: string | null;
			chat_pubkey_seller: string | null;
		} | null>
	>({ loading: true });
	const [showRevealModal, setShowRevealModal] = useState(false);
	const [revealing, setRevealing] = useState(false);

	const loadEvidence = useCallback(() => {
		setEvidence({ loading: true });
		(async () => {
			try {
				const auth: AuthHeaders = {
					address: address!,
					signature: await sign(`evidence:${caseId}`),
					message: `evidence:${caseId}`,
				};
				const data = await api.getEvidence(caseId, auth);
				setEvidence({ data, loading: false });
			} catch {
				setEvidence({ data: null, loading: false });
			}
		})();
	}, [caseId, address, sign]);

	useEffect(() => {
		if (address) loadEvidence();
		else setEvidence({ data: null, loading: false });
	}, [address, loadEvidence]);

	async function handleReveal() {
		setRevealing(true);
		try {
			const auth: AuthHeaders = {
				address: address!,
				signature: await sign(`reveal:${escrowId}`),
				message: `reveal:${escrowId}`,
			};
			const result = await api.revealChatKey(escrowId, "placeholder_chat_key", auth);
			notify("success", `Chat revealed! ${result.evidence_count} messages decrypted.`);
			setShowRevealModal(false);
			loadEvidence();
		} catch (e) {
			notify("error", "Reveal failed", (e as Error).message);
		} finally {
			setRevealing(false);
		}
	}

	return (
		<div className="panel" style={{ marginTop: "12px" }}>
			<h4 style={{ margin: "0 0 12px" }}>Chat Evidence</h4>
			{evidence.loading && <p className="muted">Loading evidence status...</p>}
			{evidence.error && <p className="muted">Could not load evidence</p>}
			{!evidence.loading && !evidence.error && evidence.data === null && status === "decided" && (
				<p className="muted">Evidence cleared</p>
			)}
			{!evidence.loading && evidence.data && evidence.data.evidence.length === 0 && status !== "decided" && (
				<>
					<p className="muted">No party has revealed the chat yet.</p>
					<button
						className="button secondary"
						style={{ marginTop: "8px" }}
						onClick={() => setShowRevealModal(true)}
					>
						Reveal my chat key
					</button>
				</>
			)}
			{!evidence.loading && evidence.data && evidence.data.evidence.length > 0 && (
				<>
					<p className="muted" style={{ marginBottom: "8px" }}>
						Chat revealed — {evidence.data.evidence.length} messages
					</p>
					{evidence.data.chat_pubkey_buyer && (
						<p className="muted" style={{ fontSize: "12px" }}>
							Buyer pubkey: <code>{evidence.data.chat_pubkey_buyer.slice(0, 16)}...</code>
						</p>
					)}
					{evidence.data.chat_pubkey_seller && (
						<p className="muted" style={{ fontSize: "12px" }}>
							Seller pubkey: <code>{evidence.data.chat_pubkey_seller.slice(0, 16)}...</code>
						</p>
					)}
					<div style={{ maxHeight: "300px", overflowY: "auto", marginTop: "8px" }}>
						{evidence.data.evidence.map((m) => (
							<div
								key={m.id}
								style={{
									padding: "8px",
									borderBottom: "1px solid var(--color-border)",
									fontSize: "13px",
								}}
							>
								<div
									style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}
								>
									<code style={{ fontSize: "11px" }}>{m.sender_address.slice(0, 16)}...</code>
									<span className="muted" style={{ fontSize: "11px" }}>
										{new Date(m.created_at * 1000).toLocaleString()}
									</span>
								</div>
								<div>{m.decrypted_content}</div>
								{m.anchor_tx_id && (
									<a
										href={`https://kas.fyi/transaction/${m.anchor_tx_id}`}
										target="_blank"
										rel="noopener noreferrer"
										style={{ fontSize: "11px" }}
									>
										Anchored
									</a>
								)}
							</div>
						))}
					</div>
				</>
			)}
			{!evidence.loading && evidence.data && evidence.data.evidence.length === 0 && status === "decided" && (
				<p className="muted">Evidence has been cleared after case resolution.</p>
			)}

			{showRevealModal && (
				<div className="modal-overlay" onClick={() => setShowRevealModal(false)}>
					<div className="modal" onClick={(e) => e.stopPropagation()}>
						<h3 style={{ margin: "0 0 12px" }}>Reveal chat to jury?</h3>
						<p
							style={{
								fontSize: "13px",
								color: "var(--color-text-secondary)",
								marginBottom: "16px",
							}}
						>
							This gives the jury read-only access to ALL your messages on this escrow. The jury
							will be able to read the decrypted message thread.
						</p>
						<p style={{ fontSize: "13px", color: "#cc6666", marginBottom: "16px" }}>
							This action cannot be undone.
						</p>
						<div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
							<button className="button secondary" onClick={() => setShowRevealModal(false)}>
								Cancel
							</button>
							<button className="button primary" onClick={handleReveal} disabled={revealing}>
								{revealing ? "Revealing..." : "Confirm Reveal"}
							</button>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}
