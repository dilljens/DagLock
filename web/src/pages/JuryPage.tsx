import { useState, useCallback, useEffect } from "react";
import { api, type AuthHeaders, type JuryCase, type JurorRegistration } from "../api";
import { badge, type LoadState } from "../helpers";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonTable } from "../ui";
import { EmptyState } from "../components/empty-state";

type Tab = "my-cases" | "register" | "candidates";

export function JuryPage() {
	const [tab, setTab] = useState<Tab>("my-cases");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<div>
			<div className="page-header">
				<h1><h1>⚖ Jury</h1></h1>
				<p>Community dispute resolution. Register as a juror, vote on cases.</p>
			</div>
			<div className="tab-bar">
				{wallet.connected && (
					<button className={`tab-btn ${tab === "my-cases" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("my-cases")}>My Cases</button>
				)}
				<button className={`tab-btn ${tab === "register" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("register")}>Register</button>
				<button className={`tab-btn ${tab === "candidates" ? "tab-btn--active" : ""}`}
					onClick={() => setTab("candidates")}>Candidates</button>
			</div>
			{tab === "my-cases" && (wallet.connected ? <MyCases address={address!} /> : <ConnectPrompt />)}
			{tab === "register" && (wallet.connected ? <RegisterSection address={address!} /> : <ConnectPrompt />)}
			{tab === "candidates" && <CandidatesSection />}
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
		api.juryActiveCases(address)
			.then((d) => setCases({ data: d.cases, loading: false }))
			.catch((e) => setCases({ error: e.message, loading: false }));
	}, [address]);

	useEffect(() => { load(); }, [load]);

	if (cases.loading) return <SkeletonTable rows={5} />;
	if (cases.error) return <p className="muted error-text">{cases.error}</p>;
	if (!cases.data?.length) return (
		<EmptyState
			icon="⚖"
			title="No active cases"
			description="You're not assigned to any jury cases right now."
		/>
	);

	return (
		<div>
			{cases.data.map((c) => (
				<article key={c.id} className="offer" style={{ cursor: "pointer", marginBottom: "8px" }}
					onClick={() => setSelectedId(selectedId === c.id ? null : c.id)}>
					<div className="offer-top">
						<strong>Case {c.id.slice(0, 16)}…</strong>
						<span className={badge(c.status)}>{c.status}</span>
					</div>
					<p>Escrow: {c.escrow_id} · Votes: {c.votes_for_seller + c.votes_for_buyer}/{c.juror_count} · Threshold: {c.threshold}</p>
					<code>{c.id}</code>
					{selectedId === c.id && c.status === "voting" && (
						<VotePanel case={c} address={address} onVoted={load} />
					)}
				</article>
			))}
		</div>
	);
}

function VotePanel({ case: c, address, onVoted }: { case: JuryCase; address: string; onVoted: () => void }) {
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
				<input value={reasoning} onChange={(e) => setReasoning(e.target.value)}
					placeholder="Why?" />
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

	if (status === "done") return (
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
					Jurors resolve disputes by voting. You need 10+ trades and a 3.0+ reputation score.
					When assigned, you have 72 hours to vote.
				</p>
				<div style={{ fontSize: "13px", color: "#88b888", marginBottom: "16px" }}>
					Address: <code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 24)}…</code>
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

	if (candidates.loading) {
		api.juryCandidates()
			.then((d) => setCandidates({ data: d.candidates, loading: false }))
			.catch((e) => setCandidates({ error: e.message, loading: false }));
	}

	if (candidates.loading) return <SkeletonTable rows={5} />;
	if (candidates.error) return <p className="muted error-text">{candidates.error}</p>;
	if (!candidates.data?.length) return (
		<EmptyState
			icon="👤"
			title="No jurors registered"
			description="Be the first to register!"
		/>
	);

	return (
		<div>
			<p className="muted" style={{ marginBottom: "12px" }}>{candidates.data.length} registered jurors</p>
			{candidates.data.map((c) => (
				<article key={c.address} className="offer" style={{ cursor: "default", marginBottom: "8px" }}>
					<div className="offer-top">
						<strong>{c.address.slice(0, 24)}…</strong>
						<span className="pill">{c.reliability_score.toFixed(1)}/5</span>
					</div>
					<p>Cases: {c.total_cases_assigned} assigned · {c.total_cases_voted} voted</p>
					<small className="muted">Registered {new Date(c.registered_at * 1000).toLocaleDateString()}</small>
				</article>
			))}
		</div>
	);
}
