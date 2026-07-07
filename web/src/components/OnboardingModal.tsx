import { useState, useEffect } from "react";
import { useWallet } from "../context/WalletContext";

const STORAGE_KEY = "daglock_onboarded";

const slides = [
	{
		title: "Welcome to DagLock",
		subtitle: "Trustless Escrow on Kaspa",
		body: "DagLock lets you trade KAS and KRC-20 tokens with anyone, anywhere — without trusting them. A SilverScript covenant holds the funds and enforces the rules. No admin keys, no backdoors, no banks.",
		icon: "🤝",
	},
	{
		title: "How It Works",
		subtitle: "4 simple steps",
		body: [
			"1. Propose — Create an escrow or accept an offer",
			"2. Lock — Buyer sends KAS to a covenant address (enforced by code)",
			"3. Confirm — Both parties agree the terms are met",
			"4. Settle — Funds release to seller. Or refund to buyer after timeout.",
		].join("\n"),
		icon: "🔄",
	},
	{
		title: "Get Started",
		subtitle: "Connect your wallet",
		body: "Connect KasWare browser extension to get started. No KasWare? Use manual mode with any Kaspa wallet. All your trades, vaults, and reputation are on-chain and verifiable.",
		icon: "🚀",
	},
];

export function OnboardingModal() {
	const [visible, setVisible] = useState(false);
	const [slide, setSlide] = useState(0);
	const { state, connect } = useWallet();

	useEffect(() => {
		const dismissed = localStorage.getItem(STORAGE_KEY);
		if (!dismissed && !state.connected) {
			setVisible(true);
		}
	}, [state.connected]);

	function dismiss() {
		localStorage.setItem(STORAGE_KEY, "1");
		setVisible(false);
	}

	function next() {
		if (slide < slides.length - 1) {
			setSlide(slide + 1);
		} else {
			dismiss();
		}
	}

	function prev() {
		if (slide > 0) setSlide(slide - 1);
	}

	if (!visible) return null;

	const s = slides[slide];

	return (
		<div className="onboarding-overlay">
			<div className="onboarding-modal">
				<button type="button" className="onboarding-skip" onClick={dismiss} aria-label="Skip">
					Skip tour
				</button>

				<div className="onboarding-icon">{s.icon}</div>
				<h2 className="onboarding-title">{s.title}</h2>
				<p className="onboarding-subtitle">{s.subtitle}</p>
				<p className="onboarding-body">{s.body}</p>

				<div className="onboarding-dots">
					{slides.map((_, i) => (
						<span
							key={i}
							className={`onboarding-dot ${i === slide ? "onboarding-dot--active" : ""}`}
						/>
					))}
				</div>

				<div className="onboarding-actions">
					{slide > 0 && (
						<button type="button" className="button" onClick={prev}>
							Back
						</button>
					)}
					{slide === 0 && <div />}
					{slide < slides.length - 1 ? (
						<button type="button" className="button primary" onClick={next}>
							Next
						</button>
					) : (
						<button
							type="button"
							className="button primary"
							onClick={() => {
								dismiss();
								if (!state.connected) connect();
							}}
						>
							Connect Wallet
						</button>
					)}
				</div>
			</div>
		</div>
	);
}
