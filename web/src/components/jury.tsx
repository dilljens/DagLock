import { useState } from "react";
import { api, type AuthHeaders, type JuryCase } from "../api";
import { badge, type LoadState } from "../helpers";
import { FormField } from "../ui";

/* ─── Jury Panel ─── */
export function JuryPanel() {
	const [regStatus, setRegStatus] = useState<"idle" | "loading" | "registered" | "error">("idle");
	const [regError, setRegError] = useState("");
	const [authAddr, setAuthAddr] = useState("");
	const [authSig, setAuthSig] = useState("");
	const [cases, setCases] = useState<LoadState<JuryCase[]>>({ loading: false });
	const [selectedCase, setSelectedCase] = useState<JuryCase | null>(null);
	const [vote, setVote] = useState("");
	const [reasoning, setReasoning] = useState("");
	const [voteResult, setVoteResult] = useState("");

	function makeAuth() {
		if (!authAddr || !authSig) return undefined;
		return {
			address: authAddr,
			signature: authSig,
			message: "jury:auth",
		} as AuthHeaders;
	}

	async function handleRegister() {
		const a = makeAuth();
		if (!a) return;
		setRegStatus("loading");
		try {
			await api.juryRegister(a);
			setRegStatus("registered");
		} catch (err) {
			setRegStatus("error");
			setRegError((err as Error).message);
		}
	}

	async function handleUnregister() {
		const a = makeAuth();
		if (!a) return;
		setRegStatus("loading");
		try {
			await api.juryUnregister(a);
			setRegStatus("idle");
		} catch (err) {
			setRegStatus("error");
			setRegError((err as Error).message);
		}
	}

	async function loadCases() {
		const a = makeAuth();
		if (!a) return;
		setCases({ loading: true });
		try {
			const r = await api.juryCases(a);
			setCases({ data: r.cases, loading: false });
		} catch (err) {
			setCases({ error: (err as Error).message, loading: false });
		}
	}

	async function handleVote() {
		if (!selectedCase || !vote) return;
		const a = makeAuth();
		if (!a) return;
		try {
			const r = await api.juryVote(selectedCase.id, vote, reasoning || undefined, a);
			setVoteResult(r.verdict ? `Verdict: ${r.vote} (case decided)` : `Voted: ${r.vote}`);
			loadCases();
		} catch (err) {
			setVoteResult(`Error: ${(err as Error).message}`);
		}
	}

	return (
		<div className="stack">
			<FormField label="Your address">
				<input
					value={authAddr}
					onChange={(e) => setAuthAddr(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Signature (hex)">
				<input
					value={authSig}
					onChange={(e) => setAuthSig(e.target.value)}
					placeholder="hex signature"
				/>
			</FormField>
			<div className="action-tabs">
				<button
					type="button"
					className="button primary"
					onClick={handleRegister}
					disabled={regStatus === "loading"}
				>
					{regStatus === "loading" ? "Registering…" : "Register as juror"}
				</button>
				<button type="button" className="button" onClick={handleUnregister}>
					Unregister
				</button>
				<button type="button" className="button" onClick={loadCases}>
					Load my cases
				</button>
			</div>
			{regStatus === "registered" && <p className="muted success-text">Registered as juror!</p>}
			{regError && <p className="muted error-text">{regError}</p>}
			{voteResult && <p className="muted">{voteResult}</p>}

			{cases.loading && <p className="muted">Loading cases…</p>}
			{cases.data && cases.data.length === 0 && (
				<p className="muted">No active cases assigned to you.</p>
			)}
			{cases.data?.map((c) => (
				<article key={c.id} className="offer" onClick={() => setSelectedCase(c)}>
					<div className="offer-top">
						<strong>Case: {c.id.slice(0, 16)}…</strong>
						<span className={badge(c.status)}>{c.status}</span>
					</div>
					<p>
						Escrow: {c.escrow_id} | Votes: {c.votes_for_seller + c.votes_for_buyer}/{c.juror_count}{" "}
						| Threshold: {c.threshold}
					</p>
				</article>
			))}

			{selectedCase && selectedCase.status === "voting" && (
				<div className="panel">
					<h4>Cast vote for {selectedCase.id.slice(0, 16)}…</h4>
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
					<button type="button" className="button primary" onClick={handleVote}>
						Submit vote
					</button>
				</div>
			)}
		</div>
	);
}

/* ─── Resolve Dispute Panel ─── */
export function ResolveDisputePanel({ onDone }: { onDone: () => void }) {
	const [escrowId, setEscrowId] = useState("");
	const [outcome, setOutcome] = useState("expunge");
	const [authAddress, setAuthAddress] = useState("");
	const [authSig, setAuthSig] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!escrowId || !authAddress || !authSig) {
			setError("Escrow ID and auth required");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const auth: AuthHeaders = {
				address: authAddress,
				signature: authSig,
				message: `resolve:${escrowId}`,
			};
			await api.resolveDispute(escrowId, outcome, authAddress, auth);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") return <p className="muted success-text">Dispute resolved!</p>;

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Escrow ID">
				<input
					value={escrowId}
					onChange={(e) => setEscrowId(e.target.value)}
					placeholder="esc_..."
				/>
			</FormField>
			<FormField label="Outcome">
				<select value={outcome} onChange={(e) => setOutcome(e.target.value)}>
					<option value="expunge">Expunge (dismiss dispute)</option>
					<option value="uphold">Uphold (dispute valid)</option>
				</select>
			</FormField>
			<FormField label="Your address">
				<input
					value={authAddress}
					onChange={(e) => setAuthAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Signature">
				<input value={authSig} onChange={(e) => setAuthSig(e.target.value)} placeholder="hex" />
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Resolving..." : "Resolve dispute"}
			</button>
		</form>
	);
}

/* ─── Vouch Panel ─── */
export function VouchPanel({ onDone }: { onDone: () => void }) {
	const [subjectAddress, setSubjectAddress] = useState("");
	const [note, setNote] = useState("");
	const [authAddress, setAuthAddress] = useState("");
	const [authSig, setAuthSig] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!subjectAddress.startsWith("kaspa:") || !authAddress || !authSig) {
			setError("Valid Kaspa address and auth signature required");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const auth: AuthHeaders = {
				address: authAddress,
				signature: authSig,
				message: `vouch:${subjectAddress}`,
			};
			await api.vouch(subjectAddress, auth, undefined, note || undefined);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") return <p className="muted success-text">Vouch created!</p>;

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Subject address">
				<input
					value={subjectAddress}
					onChange={(e) => setSubjectAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Note (optional)">
				<input
					value={note}
					onChange={(e) => setNote(e.target.value)}
					placeholder="Why do you vouch for them?"
				/>
			</FormField>
			<FormField label="Your address">
				<input
					value={authAddress}
					onChange={(e) => setAuthAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Signature">
				<input value={authSig} onChange={(e) => setAuthSig(e.target.value)} placeholder="hex" />
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating..." : "Create vouch"}
			</button>
		</form>
	);
}
