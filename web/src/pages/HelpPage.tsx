import { useState } from "react";
import { Helmet } from "react-helmet-async";

const faqs = [
	{
		q: "What is an escrow?",
		a: "An escrow is a financial arrangement where a neutral third party holds funds until both parties fulfill their obligations. DagLock uses SilverScript covenants on Kaspa L1 instead of a human third party — the code itself enforces the rules. Funds only release when the agreed conditions are met.",
	},
	{
		q: "How are fees calculated?",
		a: "DagLock charges a flat 0.5% protocol fee (1/200) on settlement. This fee is enforced by the covenant — it cannot be bypassed. For example, on a 1000 KAS escrow, the fee is 5 KAS and the seller receives 995 KAS. The fee goes to the DagLock treasury. Timeout refunds have no fee.",
	},
	{
		q: "What if the other party doesn't respond?",
		a: "Every escrow has a configurable timeout. If the buyer doesn't settle before the timeout, the seller can refund the funds back to the buyer. If both parties are unresponsive, the escrow will expire and funds are returned to the buyer. You can also cancel an escrow in the 'proposed' state before funds are locked.",
	},
	{
		q: "How do disputes work?",
		a: "Any party can dispute an escrow if something goes wrong. DagLock offers three dispute modes: Standard (timeout-based refund), Mediator (a trusted third party resolves), and Jury (random community members vote). During a dispute, funds remain locked in the covenant — no one can move them until resolution.",
	},
	{
		q: "What is the jury system?",
		a: "The jury system allows community members to vote on escrow disputes. Jurors are randomly selected from registered candidates. Each case needs a threshold of votes to reach a decision. Jurors with high reliability scores are weighted more heavily. The jury outcome determines who receives the funds.",
	},
	{
		q: "What assets are supported?",
		a: "DagLock supports native KAS and all KRC-20 tokens on Kaspa. KAS escrows use the standard daglock.sil covenant. KRC-20 escrows use daglock_krc20.sil with ICC (Inter-Covenant Communication) for token ownership enforcement. Atomic swaps between different assets are supported via hash preimage.",
	},
	{
		q: "How long does an escrow take?",
		a: "The escrow lifecycle has no hard time limit — it depends on the timeout you set at creation (configurable from 1 hour to 1 year). Settlement is instant once both parties agree and sign. Disputes follow a structured timeline: negotiation period + voting period.",
	},
	{
		q: "Is DagLock safe?",
		a: "DagLock's smart contracts (covenants) enforce all rules at the protocol level. DagLock the company never holds your funds — they sit in a covenant UTXO on Kaspa L1. The code is open source and has undergone an internal security review. Key properties: no admin keys, no backdoors, no upgrade mechanisms that could change the rules.",
	},
	{
		q: "What's the difference between KasWare, Kaspium, and manual mode?",
		a: "KasWare is a browser extension wallet (Chrome/Firefox) that integrates directly with the DagLock web UI for signing. Kaspium is a mobile wallet (Android/iOS). Manual mode lets you use any wallet by copying addresses and signing messages externally. We recommend KasWare for desktop users and Kaspium for mobile.",
	},
	{
		q: "How do I get testnet KAS?",
		a: "Visit the Kaspa Testnet Faucet at https://faucet-testnet.kaspanet.io/ to request test KAS tokens. These have no real value and are only for testing on testnet-12.",
	},
];

const quickStart = [
	{
		step: "1",
		title: "Connect a wallet",
		desc: "Install KasWare browser extension or use manual mode with any Kaspa wallet.",
	},
	{
		step: "2",
		title: "Browse or create an offer",
		desc: "Check the Offer Board for existing trades, or create your own with your terms.",
	},
	{
		step: "3",
		title: "Lock funds in a covenant",
		desc: "The buyer sends KAS to a covenant address. Only the covenant code controls these funds now — nobody can steal them.",
	},
	{
		step: "4",
		title: "Settle the escrow",
		desc: "Both parties sign to release funds to the seller. Or wait for timeout to refund the buyer. The 0.5% fee is deducted automatically.",
	},
];

export function HelpPage() {
	const [openFaq, setOpenFaq] = useState<number | null>(null);

	return (
		<>
			<Helmet>
				<title>Help & FAQ — DagLock</title>
				<meta
					name="description"
					content="Learn how DagLock escrow works, how fees are calculated, dispute resolution, and more."
				/>
			</Helmet>
			<div className="help-page">
				<div className="page-header">
					<h1>Help & FAQ</h1>
					<p>Everything you need to know about DagLock escrow on Kaspa.</p>
				</div>

				{/* Quick Start */}
				<section className="panel" style={{ marginBottom: "24px" }}>
					<h3 style={{ margin: "0 0 16px" }}>Quick Start</h3>
					<p className="muted" style={{ marginTop: 0 }}>
						From zero to your first escrow in 4 steps.
					</p>
					<div className="quick-start-steps">
						{quickStart.map((s) => (
							<div key={s.step} className="quick-start-step">
								<div className="quick-start-number">{s.step}</div>
								<div>
									<strong>{s.title}</strong>
									<p className="muted" style={{ margin: "4px 0 0", fontSize: "13px" }}>
										{s.desc}
									</p>
								</div>
							</div>
						))}
					</div>
				</section>

				{/* FAQ */}
				<section className="panel">
					<h3 style={{ margin: "0 0 16px" }}>Frequently Asked Questions</h3>
					<div className="faq-list">
						{faqs.map((faq, i) => (
							<div key={i} className="faq-item">
								<button
									type="button"
									className={`faq-question ${openFaq === i ? "faq-question--open" : ""}`}
									onClick={() => setOpenFaq(openFaq === i ? null : i)}
								>
									<span>{faq.q}</span>
									<span className="faq-chevron">{openFaq === i ? "▲" : "▼"}</span>
								</button>
								{openFaq === i && (
									<div className="faq-answer">
										<p>{faq.a}</p>
									</div>
								)}
							</div>
						))}
					</div>
				</section>

				{/* Additional resources */}
				<section className="panel" style={{ marginTop: "24px" }}>
					<h3 style={{ margin: "0 0 8px" }}>More Resources</h3>
					<ul className="help-links">
						<li>
							<a href="/docs" target="_blank" rel="noopener noreferrer">
								Developer Documentation — API reference, CLI, bot
							</a>
						</li>
						<li>
							<a
								href="https://github.com/dilljens/DagLock"
								target="_blank"
								rel="noopener noreferrer"
							>
								GitHub — Source code, issues, contribute
							</a>
						</li>
						<li>
							<a href="https://kas.fyi" target="_blank" rel="noopener noreferrer">
								Kaspa Block Explorer — View transactions on-chain
							</a>
						</li>
					</ul>
				</section>
			</div>
		</>
	);
}
