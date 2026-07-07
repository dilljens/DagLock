import { useState, useEffect, useCallback } from "react";
import { api, type AuthHeaders, type MediationResult, type MediationStatus } from "../api";
import { useAddress, useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";

type Props = {
	escrowId: string;
	disputeMode?: string | null;
};

export function MediationPanel({ escrowId, disputeMode }: Props) {
	const address = useAddress();
	const { sign, connect } = useWallet();
	const { notify } = useToast();
	const [status, setStatus] = useState<MediationStatus | null>(null);
	const [loading, setLoading] = useState(false);
	const [buyerClaim, setBuyerClaim] = useState("");
	const [sellerClaim, setSellerClaim] = useState("");
	const [showForm, setShowForm] = useState(false);
	const [hasVoted, setHasVoted] = useState<"buyer" | "seller" | null>(null);

	const loadMediation = useCallback(async () => {
		try {
			const s = await api.getMediation(escrowId);
			setStatus(s);
		} catch {
			// No mediation exists yet
		}
	}, [escrowId]);

	useEffect(() => {
		loadMediation();
	}, [loadMediation]);

	if (!address) {
		return (
			<div className="panel" style={{ marginBottom: "16px", padding: "12px 16px" }}>
				<p className="muted">
					<a
						href="#"
						onClick={(e) => {
							e.preventDefault();
							connect();
						}}
						style={{ cursor: "pointer" }}
					>
						Connect your wallet
					</a>{" "}
					to use AI mediation.
				</p>
			</div>
		);
	}

	// Mediation exists — show status
	if (status && status.mediation_status) {
		const isCompleted = status.mediation_status === "completed";
		const isPending = status.mediation_status === "pending";
		const isEscalated = status.mediation_status === "escalated";

		const remaining = status.expires_at ? Math.max(0, status.expires_at * 1000 - Date.now()) : 0;
		const remainingHrs = Math.ceil(remaining / 3600000);

		return (
			<div className="panel" style={{ marginBottom: "16px", padding: "12px 16px" }}>
				<h4 style={{ margin: "0 0 8px" }}>
					🤖 AI Mediation
					<span className="pill" style={{ marginLeft: "8px", fontSize: "11px" }}>
						{status.mediation_status}
					</span>
				</h4>

				{isPending && (
					<p className="muted" style={{ fontSize: "13px" }}>
						AI is analyzing the dispute... Check back shortly.
					</p>
				)}

				{isCompleted && status.recommendation && (
					<>
						<div
							style={{
								background: "var(--color-bg-secondary)",
								borderRadius: "8px",
								padding: "12px",
								marginBottom: "12px",
							}}
						>
							<p style={{ margin: "0 0 8px", fontWeight: 600, fontSize: "14px" }}>
								Proposed Outcome:{" "}
								<span
									style={{
										color:
											status.recommendation.outcome === "refund"
												? "#e88"
												: status.recommendation.outcome === "payout"
													? "#8e8"
													: "#ee8",
									}}
								>
									{status.recommendation.outcome.toUpperCase()}
									{status.recommendation.outcome === "split" &&
										` (${(status.recommendation.buyer_share_basis / 100).toFixed(1)}% buyer / ${(100 - status.recommendation.buyer_share_basis / 100).toFixed(1)}% seller)`}
								</span>
							</p>
							<p style={{ margin: 0, fontSize: "13px", whiteSpace: "pre-wrap" }}>
								{status.recommendation.reasoning}
							</p>
						</div>

						<div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
							<AcceptButton
								party="buyer"
								escrowId={escrowId}
								accepted={status.buyer_accepted}
								disabled={hasVoted !== null}
								onDone={() => {
									setHasVoted("buyer");
									loadMediation();
								}}
							/>
							<AcceptButton
								party="seller"
								escrowId={escrowId}
								accepted={status.seller_accepted}
								disabled={hasVoted !== null}
								onDone={() => {
									setHasVoted("seller");
									loadMediation();
								}}
							/>
						</div>

						<p className="muted" style={{ fontSize: "12px", marginTop: "8px" }}>
							{status.both_accepted
								? "✅ Both parties accepted! Outcome is being executed."
								: remaining > 0
									? `⏰ Escalates to jury in ~${remainingHrs}h if not both accepted`
									: "⏰ Escalation deadline passed — dispute moving to jury."}
						</p>
					</>
				)}

				{isEscalated && (
					<p className="muted" style={{ fontSize: "13px" }}>
						⏰ Mediation period has ended. This dispute has been escalated to the jury system.
					</p>
				)}
			</div>
		);
	}

	// No mediation yet — show prompt
	return (
		<div className="panel" style={{ marginBottom: "16px", padding: "12px 16px" }}>
			<h4 style={{ margin: "0 0 8px" }}>🤖 AI Mediation Available</h4>
			<p className="muted" style={{ fontSize: "13px", marginBottom: "12px" }}>
				Before going to a jury vote, try AI mediation. It's <strong>free and non-binding</strong> —
				the AI analyzes the evidence and proposes a fair outcome. Takes about 2 minutes. If either
				party disagrees, it escalates to jury automatically.
			</p>

			{!showForm ? (
				<button className="button primary" onClick={() => setShowForm(true)}>
					Start AI Mediation
				</button>
			) : (
				<div>
					<FormField label="Your claim — what happened and what you want">
						<textarea
							value={buyerClaim}
							onChange={(e) => setBuyerClaim(e.target.value)}
							placeholder="Describe the dispute from your perspective. Include dates, amounts, and any relevant details..."
							rows={4}
							style={{ width: "100%", resize: "vertical" }}
						/>
					</FormField>
					<button
						className="button primary"
						disabled={!buyerClaim.trim() || loading}
						onClick={async () => {
							setLoading(true);
							try {
								const auth: AuthHeaders = {
									address,
									signature: await sign("mediation:submit"),
									message: "mediation:submit",
								};
								await api.mediateEscrow(
									escrowId,
									{
										buyer_claim: buyerClaim,
										seller_claim: "", // will be filled by other party
									},
									auth,
								);
								notify("success", "Mediation started! AI analyzing dispute...");
								loadMediation();
							} catch (e) {
								notify("error", "Failed to start mediation", (e as Error).message);
							} finally {
								setLoading(false);
							}
						}}
					>
						{loading ? "Analyzing..." : "Start Mediation"}
					</button>
				</div>
			)}
		</div>
	);
}

function AcceptButton({
	party,
	escrowId,
	accepted,
	disabled,
	onDone,
}: {
	party: string;
	escrowId: string;
	accepted: boolean;
	disabled: boolean;
	onDone: () => void;
}) {
	const { sign } = useWallet();
	const { notify } = useToast();
	const address = useAddress();
	const [loading, setLoading] = useState(false);

	if (accepted) {
		return (
			<span className="pill" style={{ background: "#282" }}>
				✅ {party === "buyer" ? "Buyer" : "Seller"} accepted
			</span>
		);
	}

	return (
		<button
			className="button secondary"
			disabled={disabled || loading}
			style={{ fontSize: "12px", padding: "4px 12px" }}
			onClick={async () => {
				setLoading(true);
				try {
					const auth: AuthHeaders = {
						address: address!,
						signature: await sign(`mediation:accept:${escrowId}`),
						message: `mediation:accept:${escrowId}`,
					};
					const r = await api.acceptMediation(escrowId, party, true, auth);
					if (r.outcome_executed) {
						notify("success", "Both accepted! Outcome executed.");
					} else {
						notify("success", `Accepted! Waiting for other party.`);
					}
					onDone();
				} catch (e) {
					notify("error", "Failed to accept", (e as Error).message);
				} finally {
					setLoading(false);
				}
			}}
		>
			{loading ? "..." : `${party === "buyer" ? "Buyer" : "Seller"}: Accept Outcome`}
		</button>
	);
}
